import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

export const PROGRAM_ID = new PublicKey(
  "ApoLLoSwarm111111111111111111111111111111111",
);

export enum AgentStatus {
  Active = 0,
  Paused = 1,
  Revoked = 2,
}

export enum CoordModel {
  RoleBased = 0,
  ThresholdApproval = 1,
  RoundRobin = 2,
  Voting = 3,
}

export enum TaskState {
  Created = 0,
  Accepted = 1,
  Completed = 2,
  Verified = 3,
  Failed = 4,
  Settled = 5,
}

export interface AgentAccount {
  authority: PublicKey;
  role: string;
  metadataUri: string;
  status: number;
  reputation: BN;
  tasksCompleted: BN;
  tasksFailed: BN;
  createdAt: BN;
  bump: number;
}

export interface SwarmAccount {
  authority: PublicKey;
  swarmId: BN;
  name: string;
  metadataUri: string;
  coordModel: number;
  approvalThreshold: number;
  memberCount: number;
  taskCount: BN;
  totalShareBps: number;
  treasuryBump: number;
  createdAt: BN;
  bump: number;
}

export interface MembershipAccount {
  swarm: PublicKey;
  agent: PublicKey;
  role: string;
  shareBps: number;
  tasksExecuted: BN;
  joinedAt: BN;
  bump: number;
}

export interface TaskAccount {
  swarm: PublicKey;
  creator: PublicKey;
  executor: PublicKey;
  taskId: BN;
  requiredRole: string;
  payloadUri: string;
  resultUri: string;
  failureReason: string;
  reward: BN;
  state: number;
  approvals: number;
  workBased: boolean;
  createdAt: BN;
  acceptedAt: BN;
  completedAt: BN;
  bump: number;
}

export interface SettlementRecipient {
  membership: PublicKey;
  recipient: PublicKey;
}
