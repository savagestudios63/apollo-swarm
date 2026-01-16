import {
  AnchorProvider,
  BN,
  Program,
  type Idl,
} from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  type TransactionSignature,
} from "@solana/web3.js";

import {
  findAgentPda,
  findApprovalPda,
  findMembershipPda,
  findSwarmPda,
  findTaskPda,
  findTreasuryPda,
} from "./pdas";
import {
  type AgentAccount,
  type MembershipAccount,
  PROGRAM_ID,
  type SettlementRecipient,
  type SwarmAccount,
  type TaskAccount,
} from "./types";

type AnyProgram = Program<Idl>;

/**
 * Thin typed client around the Apollo Swarm Anchor program.
 * Callers provide the Anchor Program (loaded from the generated IDL).
 */
export class ApolloClient {
  readonly program: AnyProgram;
  readonly provider: AnchorProvider;

  constructor(program: AnyProgram) {
    if (!program.programId.equals(PROGRAM_ID)) {
      throw new Error(
        `Program ID mismatch: expected ${PROGRAM_ID.toBase58()}, got ${program.programId.toBase58()}`,
      );
    }
    this.program = program;
    this.provider = program.provider as AnchorProvider;
  }

  get wallet(): PublicKey {
    return this.provider.wallet.publicKey;
  }

  // ---------- PDA helpers (re-exported for convenience) ----------

  agentPda = findAgentPda;
  swarmPda = findSwarmPda;
  membershipPda = findMembershipPda;
  taskPda = findTaskPda;
  approvalPda = findApprovalPda;
  treasuryPda = findTreasuryPda;

  // ---------- Agent ----------

  async registerAgent(
    role: string,
    metadataUri: string,
    authority: PublicKey = this.wallet,
  ): Promise<TransactionSignature> {
    const [agent] = findAgentPda(authority);
    return this.program.methods
      .registerAgent(role, metadataUri)
      .accounts({
        authority,
        agent,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async updateAgent(
    status: number | null,
    metadataUri: string | null,
    authority: PublicKey = this.wallet,
  ): Promise<TransactionSignature> {
    const [agent] = findAgentPda(authority);
    return this.program.methods
      .updateAgent(status, metadataUri)
      .accounts({ authority, agent })
      .rpc();
  }

  // ---------- Swarm ----------

  async createSwarm(params: {
    swarmId: BN | number;
    name: string;
    metadataUri: string;
    coordModel: number;
    approvalThreshold: number;
    authority?: PublicKey;
  }): Promise<TransactionSignature> {
    const authority = params.authority ?? this.wallet;
    const swarmId = BN.isBN(params.swarmId) ? params.swarmId : new BN(params.swarmId);
    const [swarm] = findSwarmPda(authority, swarmId);
    const [treasury] = findTreasuryPda(swarm);
    return this.program.methods
      .createSwarm(
        swarmId,
        params.name,
        params.metadataUri,
        params.coordModel,
        params.approvalThreshold,
      )
      .accounts({
        authority,
        swarm,
        treasury,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async addMember(params: {
    swarm: PublicKey;
    agentAuthority: PublicKey;
    role: string;
    shareBps: number;
    authority?: PublicKey;
  }): Promise<TransactionSignature> {
    const authority = params.authority ?? this.wallet;
    const [agent] = findAgentPda(params.agentAuthority);
    const [membership] = findMembershipPda(params.swarm, agent);
    return this.program.methods
      .addMember(params.role, params.shareBps)
      .accounts({
        authority,
        swarm: params.swarm,
        agent,
        membership,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async removeMember(params: {
    swarm: PublicKey;
    agentAuthority: PublicKey;
    authority?: PublicKey;
  }): Promise<TransactionSignature> {
    const authority = params.authority ?? this.wallet;
    const [agent] = findAgentPda(params.agentAuthority);
    const [membership] = findMembershipPda(params.swarm, agent);
    return this.program.methods
      .removeMember()
      .accounts({
        authority,
        swarm: params.swarm,
        membership,
      })
      .rpc();
  }

  // ---------- Task ----------

  async delegateTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    requiredRole: string;
    payloadUri: string;
    reward: BN | number;
    workBased: boolean;
    creatorAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const creator = params.creatorAuthority ?? this.wallet;
    const [creatorAgent] = findAgentPda(creator);
    const [creatorMembership] = findMembershipPda(params.swarm, creatorAgent);
    const taskId = BN.isBN(params.taskId) ? params.taskId : new BN(params.taskId);
    const reward = BN.isBN(params.reward) ? params.reward : new BN(params.reward);
    const [task] = findTaskPda(params.swarm, taskId);

    return this.program.methods
      .createTask(taskId, params.requiredRole, params.payloadUri, reward, params.workBased)
      .accounts({
        creator,
        creatorAgent,
        authority: creator,
        swarm: params.swarm,
        creatorMembership,
        task,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async acceptTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    executorAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const executor = params.executorAuthority ?? this.wallet;
    const [executorAgent] = findAgentPda(executor);
    const [membership] = findMembershipPda(params.swarm, executorAgent);
    const [task] = findTaskPda(params.swarm, params.taskId);
    return this.program.methods
      .acceptTask()
      .accounts({
        executor,
        executorAgent,
        authority: executor,
        swarm: params.swarm,
        membership,
        task,
      })
      .rpc();
  }

  async completeTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    resultUri: string;
    executorAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const executor = params.executorAuthority ?? this.wallet;
    const [executorAgent] = findAgentPda(executor);
    const [task] = findTaskPda(params.swarm, params.taskId);
    return this.program.methods
      .completeTask(params.resultUri)
      .accounts({
        executor,
        executorAgent,
        authority: executor,
        swarm: params.swarm,
        task,
      })
      .rpc();
  }

  async failTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    reason: string;
    signerAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const signer = params.signerAuthority ?? this.wallet;
    const [agent] = findAgentPda(signer);
    const [task] = findTaskPda(params.swarm, params.taskId);
    return this.program.methods
      .failTask(params.reason)
      .accounts({
        signer,
        agent,
        authority: signer,
        swarm: params.swarm,
        task,
      })
      .rpc();
  }

  async approveTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    approverAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const approver = params.approverAuthority ?? this.wallet;
    const [approverAgent] = findAgentPda(approver);
    const [membership] = findMembershipPda(params.swarm, approverAgent);
    const [task] = findTaskPda(params.swarm, params.taskId);
    const [approval] = findApprovalPda(task, approverAgent);
    return this.program.methods
      .approveTask()
      .accounts({
        approver,
        approverAgent,
        authority: approver,
        swarm: params.swarm,
        membership,
        task,
        approval,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async verifyTask(params: {
    swarm: PublicKey;
    taskId: BN | number;
    executorAuthority: PublicKey;
    signerAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const signer = params.signerAuthority ?? this.wallet;
    const [executorAgent] = findAgentPda(params.executorAuthority);
    const [task] = findTaskPda(params.swarm, params.taskId);
    return this.program.methods
      .verifyTask()
      .accounts({
        signer,
        executorAgent,
        swarm: params.swarm,
        task,
      })
      .rpc();
  }

  // ---------- Treasury ----------

  async depositTreasury(params: {
    swarm: PublicKey;
    amount: BN | number;
    payer?: PublicKey;
  }): Promise<TransactionSignature> {
    const payer = params.payer ?? this.wallet;
    const [treasury] = findTreasuryPda(params.swarm);
    const amount = BN.isBN(params.amount) ? params.amount : new BN(params.amount);
    return this.program.methods
      .depositTreasury(amount)
      .accounts({
        payer,
        swarm: params.swarm,
        treasury,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async settleRewards(params: {
    swarm: PublicKey;
    taskId: BN | number;
    recipients: SettlementRecipient[];
    signerAuthority?: PublicKey;
  }): Promise<TransactionSignature> {
    const signer = params.signerAuthority ?? this.wallet;
    const [task] = findTaskPda(params.swarm, params.taskId);
    const [treasury] = findTreasuryPda(params.swarm);

    const remaining = params.recipients.flatMap((r) => [
      { pubkey: r.membership, isSigner: false, isWritable: false },
      { pubkey: r.recipient, isSigner: false, isWritable: true },
    ]);

    return this.program.methods
      .settleTask()
      .accounts({
        signer,
        swarm: params.swarm,
        task,
        treasury,
        systemProgram: SystemProgram.programId,
      })
      .remainingAccounts(remaining)
      .rpc();
  }

  // ---------- Fetchers ----------

  async fetchAgent(authority: PublicKey): Promise<AgentAccount | null> {
    const [pda] = findAgentPda(authority);
    return (await this.program.account.agent.fetchNullable(pda)) as AgentAccount | null;
  }

  async fetchSwarm(creator: PublicKey, swarmId: BN | number): Promise<SwarmAccount | null> {
    const [pda] = findSwarmPda(creator, swarmId);
    return (await this.program.account.swarm.fetchNullable(pda)) as SwarmAccount | null;
  }

  async fetchTask(swarm: PublicKey, taskId: BN | number): Promise<TaskAccount | null> {
    const [pda] = findTaskPda(swarm, taskId);
    return (await this.program.account.task.fetchNullable(pda)) as TaskAccount | null;
  }

  async fetchMembership(
    swarm: PublicKey,
    agentAuthority: PublicKey,
  ): Promise<MembershipAccount | null> {
    const [agent] = findAgentPda(agentAuthority);
    const [pda] = findMembershipPda(swarm, agent);
    return (await this.program.account.membership.fetchNullable(
      pda,
    )) as MembershipAccount | null;
  }

  async listSwarmMembers(swarm: PublicKey): Promise<MembershipAccount[]> {
    const all = await this.program.account.membership.all([
      { memcmp: { offset: 8, bytes: swarm.toBase58() } },
    ]);
    return all.map((a) => a.account as unknown as MembershipAccount);
  }
}
