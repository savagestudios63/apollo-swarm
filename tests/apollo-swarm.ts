import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, BN, Program, type Idl } from "@coral-xyz/anchor";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";

import {
  ApolloClient,
  CoordModel,
  TaskState,
  findAgentPda,
  findMembershipPda,
  findSwarmPda,
  findTaskPda,
  findTreasuryPda,
} from "../sdk/src";

describe("apollo-swarm", () => {
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  // The workspace loader discovers the program via Anchor.toml.
  const program = anchor.workspace.ApolloSwarm as Program<Idl>;
  const client = new ApolloClient(program);

  const payer = (provider.wallet as anchor.Wallet).payer;

  // Three keypairs: the swarm creator + two worker agents.
  const creator = payer;
  const worker = Keypair.generate();
  const reviewer = Keypair.generate();

  const SWARM_ID = new BN(1);
  const TASK_ID = new BN(0);

  const [swarmPda] = findSwarmPda(creator.publicKey, SWARM_ID);
  const [treasuryPda] = findTreasuryPda(swarmPda);
  const [taskPda] = findTaskPda(swarmPda, TASK_ID);

  before(async () => {
    // Fund worker and reviewer wallets.
    for (const kp of [worker, reviewer]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        2 * LAMPORTS_PER_SOL,
      );
      await provider.connection.confirmTransaction(sig, "confirmed");
    }
  });

  const workerClient = () =>
    new ApolloClient(
      new Program(
        program.idl,
        new AnchorProvider(provider.connection, new anchor.Wallet(worker), {}),
      ) as Program<Idl>,
    );

  const reviewerClient = () =>
    new ApolloClient(
      new Program(
        program.idl,
        new AnchorProvider(provider.connection, new anchor.Wallet(reviewer), {}),
      ) as Program<Idl>,
    );

  it("registers agents", async () => {
    await client.registerAgent("coordinator", "ipfs://creator");
    await workerClient().registerAgent("worker", "ipfs://worker");
    await reviewerClient().registerAgent("reviewer", "ipfs://reviewer");

    const a = await client.fetchAgent(creator.publicKey);
    assert.ok(a);
    assert.equal(a!.role, "coordinator");
    assert.equal(a!.status, 0);
  });

  it("creates a swarm with threshold-approval coordination", async () => {
    await client.createSwarm({
      swarmId: SWARM_ID,
      name: "apollo-test",
      metadataUri: "ipfs://swarm",
      coordModel: CoordModel.ThresholdApproval,
      approvalThreshold: 1,
    });

    const s = await client.fetchSwarm(creator.publicKey, SWARM_ID);
    assert.ok(s);
    assert.equal(s!.coordModel, CoordModel.ThresholdApproval);
    assert.equal(s!.approvalThreshold, 1);
  });

  it("adds members with share bps", async () => {
    await client.addMember({
      swarm: swarmPda,
      agentAuthority: creator.publicKey,
      role: "coordinator",
      shareBps: 2000,
    });
    await client.addMember({
      swarm: swarmPda,
      agentAuthority: worker.publicKey,
      role: "worker",
      shareBps: 6000,
    });
    await client.addMember({
      swarm: swarmPda,
      agentAuthority: reviewer.publicKey,
      role: "reviewer",
      shareBps: 2000,
    });

    const s = await client.fetchSwarm(creator.publicKey, SWARM_ID);
    assert.equal(s!.memberCount, 3);
    assert.equal(s!.totalShareBps, 10000);
  });

  it("rejects share overflow", async () => {
    const extra = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      extra.publicKey,
      LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig, "confirmed");
    const extraClient = new ApolloClient(
      new Program(
        program.idl,
        new AnchorProvider(provider.connection, new anchor.Wallet(extra), {}),
      ) as Program<Idl>,
    );
    await extraClient.registerAgent("worker", "ipfs://extra");

    let threw = false;
    try {
      await client.addMember({
        swarm: swarmPda,
        agentAuthority: extra.publicKey,
        role: "worker",
        shareBps: 1,
      });
    } catch (e: any) {
      threw = true;
      assert.match(e.toString(), /ShareOverflow/);
    }
    assert.isTrue(threw, "expected ShareOverflow");
  });

  it("delegates, accepts, completes, approves, verifies a task", async () => {
    await client.depositTreasury({
      swarm: swarmPda,
      amount: new BN(LAMPORTS_PER_SOL),
    });

    await client.delegateTask({
      swarm: swarmPda,
      taskId: TASK_ID,
      requiredRole: "worker",
      payloadUri: "ipfs://payload",
      reward: new BN(LAMPORTS_PER_SOL / 2),
      workBased: false,
    });

    await workerClient().acceptTask({ swarm: swarmPda, taskId: TASK_ID });
    await workerClient().completeTask({
      swarm: swarmPda,
      taskId: TASK_ID,
      resultUri: "ipfs://result",
    });

    // Reviewer approves — threshold = 1.
    await reviewerClient().approveTask({ swarm: swarmPda, taskId: TASK_ID });

    await client.verifyTask({
      swarm: swarmPda,
      taskId: TASK_ID,
      executorAuthority: worker.publicKey,
    });

    const t = await client.fetchTask(swarmPda, TASK_ID);
    assert.equal(t!.state, TaskState.Verified);
  });

  it("settles rewards proportionally", async () => {
    const [creatorMembership] = findMembershipPda(
      swarmPda,
      findAgentPda(creator.publicKey)[0],
    );
    const [workerMembership] = findMembershipPda(
      swarmPda,
      findAgentPda(worker.publicKey)[0],
    );
    const [reviewerMembership] = findMembershipPda(
      swarmPda,
      findAgentPda(reviewer.publicKey)[0],
    );

    const beforeWorker = await provider.connection.getBalance(worker.publicKey);

    await client.settleRewards({
      swarm: swarmPda,
      taskId: TASK_ID,
      recipients: [
        { membership: creatorMembership, recipient: creator.publicKey },
        { membership: workerMembership, recipient: worker.publicKey },
        { membership: reviewerMembership, recipient: reviewer.publicKey },
      ],
    });

    const afterWorker = await provider.connection.getBalance(worker.publicKey);
    // worker share = 60% of reward
    const expected = (LAMPORTS_PER_SOL / 2) * 0.6;
    assert.approximately(afterWorker - beforeWorker, expected, 10_000);

    const t = await client.fetchTask(swarmPda, TASK_ID);
    assert.equal(t!.state, TaskState.Settled);
  });
});
