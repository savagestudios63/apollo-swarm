<p align="center">
  <img src="https://github.com/user-attachments/assets/ec5856b7-0aab-41fe-8ffd-ed74fb4c7649" width="140" />
</p>

# Apollo Swarm

Sovereign multi-agent AI coordination on Solana.

Apollo Swarm is an on-chain coordination layer. Multiple specialized AI agents
register verifiable identities, join swarms, delegate tasks to each other, and
share rewards transparently — all enforced by an Anchor program instead of a
trusted operator.

---

## Architecture

```
┌───────────────────────────────┐
│          Anchor program       │   ← source of truth (Agents, Swarms,
│   (programs/apollo-swarm)     │     Memberships, Tasks, Approvals, Treasury)
└──────────────┬────────────────┘
               │ IDL / events
┌──────────────▼────────────────┐
│      TypeScript SDK           │   ← typed client + PDA helpers (sdk/)
└──────────────┬────────────────┘
               │ subscribe / submit
┌──────────────▼────────────────┐
│   Runtime / Orchestrator      │   ← listens to events, runs agent logic,
│        (runtime/)             │     submits completion transactions
└───────────────────────────────┘
```

### PDAs

| Account      | Seeds                                          |
|--------------|------------------------------------------------|
| `Agent`      | `["agent", authority]`                         |
| `Swarm`      | `["swarm", creator, swarm_id_u64]`             |
| `Membership` | `["member", swarm, agent]`                     |
| `Task`       | `["task", swarm, task_id_u64]`                 |
| `Approval`   | `["approval", task, agent]`                    |
| `Treasury`   | `["treasury", swarm]` (system-owned PDA)       |

### Coordination models

- **RoleBased** (0) — tasks carry a required role; verification is gated by
  the swarm authority.
- **ThresholdApproval** (1) — members other than the executor submit on-chain
  `Approval` PDAs; verification succeeds once `approvals ≥ threshold`.
- **RoundRobin** (2) — identical on-chain semantics as RoleBased; executor
  rotation is handled off-chain by the orchestrator.
- **Voting** (3) — same on-chain mechanism as ThresholdApproval; weighting and
  tallying happen off-chain before surfacing an approval.

### Task lifecycle

```
Created → Accepted → Completed ──▶ Verified ──▶ Settled
                          └──▶ Failed
```

Every transition emits an event. Invalid transitions are rejected with
`InvalidTaskState`.

### Profit sharing

Each `Membership` stores a `share_bps` (basis points). `Swarm.total_share_bps`
is the sum; `add_member` enforces `total_share_bps ≤ 10_000`.

- `work_based = false`: on `settle_task`, the caller supplies every member +
  recipient wallet. Each member is paid
  `reward * share_bps / swarm.total_share_bps`.
- `work_based = true`: the full reward goes to the executor's recipient wallet.

The treasury PDA is a plain system account; deposits use `SystemProgram.transfer`
and payouts are issued via `CpiContext::new_with_signer` using the treasury's
seeds.

---

## Repository layout

```
apollo-swarm/
├── programs/apollo-swarm/    # Anchor program (Rust)
│   └── src/
│       ├── lib.rs            # #[program] entry
│       ├── constants.rs
│       ├── errors.rs
│       ├── events.rs
│       ├── state/            # Agent, Swarm, Membership, Task, Approval
│       └── instructions/
│           ├── agent/
│           ├── swarm/
│           ├── task/
│           └── treasury/
├── sdk/                      # TypeScript SDK (typed client + PDAs)
├── runtime/                  # Example orchestrator
├── tests/                    # Anchor integration tests
├── Anchor.toml
└── Cargo.toml
```

---

## Getting started

### Requirements

- Rust (stable) + Solana CLI
- Anchor 0.30.x (`avm install 0.30.1 && avm use 0.30.1`)
- Node.js 18+ and Yarn

### Build & test

```bash
yarn install
anchor build
anchor test
```

### Run the example orchestrator

```bash
# 1. Deploy the program to devnet and note the program id.
anchor deploy --provider.cluster devnet

# 2. Point the runtime at a swarm you have already created.
ANCHOR_WALLET=~/.config/solana/id.json \
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
APOLLO_SWARM=<swarm-pubkey> \
APOLLO_ROLE=worker \
  yarn --cwd runtime start
```

The default handler in `runtime/src/handlers.ts` is a no-op echo. Replace it
with a call into your LLM or tooling of choice.

---

## SDK usage

```ts
import { ApolloClient, CoordModel } from "@apollo-swarm/sdk";

const client = new ApolloClient(program);

await client.registerAgent("worker", "ipfs://agent.json");

const swarmId = new BN(1);
await client.createSwarm({
  swarmId,
  name: "apollo-demo",
  metadataUri: "ipfs://swarm.json",
  coordModel: CoordModel.ThresholdApproval,
  approvalThreshold: 2,
});

const [swarm] = client.swarmPda(creator.publicKey, swarmId);
await client.addMember({ swarm, agentAuthority: worker, role: "worker", shareBps: 8000 });

await client.delegateTask({
  swarm,
  taskId: 0,
  requiredRole: "worker",
  payloadUri: "ipfs://payload.json",
  reward: new BN(1_000_000),
  workBased: true,
});
```

---

## Instruction reference

| Instruction        | Description                                        |
|--------------------|----------------------------------------------------|
| `register_agent`   | Create a PDA-based agent identity.                 |
| `update_agent`     | Change status or metadata.                         |
| `create_swarm`     | Open a new swarm + treasury PDA.                   |
| `add_member`       | Join an agent to the swarm with role + share bps.  |
| `remove_member`    | Close a membership, freeing share bps.             |
| `create_task`      | Creator delegates work; `task_id == swarm.task_count`. |
| `accept_task`      | Member with matching role takes ownership.         |
| `complete_task`    | Executor posts a result URI.                       |
| `fail_task`        | Executor or swarm authority aborts the task.       |
| `approve_task`     | Non-executor member records on-chain approval.     |
| `verify_task`      | Finalize — gated by coord model / threshold.       |
| `deposit_treasury` | Top up the swarm treasury.                         |
| `settle_task`      | Pay out a verified task (work-based or by shares). |

---

## License

MIT.
