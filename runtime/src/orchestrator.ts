import { type Program, type Idl, BN } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import { ApolloClient, TaskState } from "@apollo-swarm/sdk";

export type TaskHandler = (ctx: TaskContext) => Promise<string>;

export interface TaskContext {
  swarm: PublicKey;
  taskId: BN;
  payloadUri: string;
  requiredRole: string;
  reward: BN;
}

export interface OrchestratorOptions {
  agentRole: string;
  swarm: PublicKey;
  handler: TaskHandler;
  /** Optional filter — only execute tasks whose required_role matches one of these. */
  acceptRoles?: string[];
}

/**
 * Minimal runtime that subscribes to on-chain events and delegates task
 * execution to a user-supplied handler (which typically calls an LLM or
 * external system). Transactions for accept/complete are submitted by the
 * wallet configured in the Anchor provider.
 */
export class Orchestrator {
  private client: ApolloClient;
  private listeners: number[] = [];

  constructor(
    program: Program<Idl>,
    private readonly opts: OrchestratorOptions,
  ) {
    this.client = new ApolloClient(program);
  }

  async start(): Promise<void> {
    const { program } = this.client;

    const createdListener = program.addEventListener(
      "TaskCreated",
      async (evt, _slot) => {
        await this.handleTaskCreated(evt).catch((e) =>
          console.error("[apollo] task handler error:", e),
        );
      },
    );
    this.listeners.push(createdListener);

    console.log(
      `[apollo] orchestrator online for role="${this.opts.agentRole}" on swarm ${this.opts.swarm.toBase58()}`,
    );
  }

  async stop(): Promise<void> {
    await Promise.all(
      this.listeners.map((id) => this.client.program.removeEventListener(id)),
    );
    this.listeners = [];
  }

  private async handleTaskCreated(evt: {
    swarm: PublicKey;
    taskId: BN;
    requiredRole: string;
    reward: BN;
  }): Promise<void> {
    if (!evt.swarm.equals(this.opts.swarm)) return;

    const roleFilter = this.opts.acceptRoles ?? [this.opts.agentRole];
    if (evt.requiredRole !== "" && !roleFilter.includes(evt.requiredRole)) return;

    const task = await this.client.fetchTask(evt.swarm, evt.taskId);
    if (!task || task.state !== TaskState.Created) return;

    await this.client.acceptTask({ swarm: evt.swarm, taskId: evt.taskId });

    const resultUri = await this.opts.handler({
      swarm: evt.swarm,
      taskId: evt.taskId,
      payloadUri: task.payloadUri,
      requiredRole: task.requiredRole,
      reward: task.reward,
    });

    await this.client.completeTask({
      swarm: evt.swarm,
      taskId: evt.taskId,
      resultUri,
    });

    console.log(
      `[apollo] task #${evt.taskId.toString()} completed → ${resultUri}`,
    );
  }
}
