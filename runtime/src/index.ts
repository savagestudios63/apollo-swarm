import { AnchorProvider, Program, type Idl } from "@coral-xyz/anchor";
import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";

import { echoHandler } from "./handlers";
import { Orchestrator } from "./orchestrator";

/**
 * Minimal runnable example. Usage:
 *   ANCHOR_WALLET=~/.config/solana/id.json \
 *   APOLLO_SWARM=<swarm pubkey> \
 *   APOLLO_ROLE=worker \
 *   ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
 *   ts-node runtime/src/index.ts
 */
async function main() {
  const swarmKey = process.env.APOLLO_SWARM;
  if (!swarmKey) throw new Error("APOLLO_SWARM env var is required");
  const role = process.env.APOLLO_ROLE ?? "worker";

  const connection = new Connection(
    process.env.ANCHOR_PROVIDER_URL ?? clusterApiUrl("devnet"),
    "confirmed",
  );
  const provider = AnchorProvider.env();

  // The IDL is emitted by `anchor build` to target/idl/apollo_swarm.json
  // and mirrored into sdk/ at publish time. For runtime use, load it from
  // disk or an npm package.
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const idl = require("../../target/idl/apollo_swarm.json") as Idl;
  const program = new Program(idl, provider) as Program<Idl>;

  const orchestrator = new Orchestrator(program, {
    agentRole: role,
    swarm: new PublicKey(swarmKey),
    handler: echoHandler,
  });

  await orchestrator.start();

  process.on("SIGINT", async () => {
    await orchestrator.stop();
    process.exit(0);
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
