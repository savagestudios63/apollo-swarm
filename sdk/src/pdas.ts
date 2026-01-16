import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

import { PROGRAM_ID } from "./types";

const enc = (s: string) => Buffer.from(s, "utf8");

export const AGENT_SEED = enc("agent");
export const SWARM_SEED = enc("swarm");
export const MEMBER_SEED = enc("member");
export const TASK_SEED = enc("task");
export const APPROVAL_SEED = enc("approval");
export const TREASURY_SEED = enc("treasury");

function u64Le(n: BN | number): Buffer {
  const bn = BN.isBN(n) ? n : new BN(n);
  return bn.toArrayLike(Buffer, "le", 8);
}

export function findAgentPda(authority: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [AGENT_SEED, authority.toBuffer()],
    PROGRAM_ID,
  );
}

export function findSwarmPda(
  creator: PublicKey,
  swarmId: BN | number,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SWARM_SEED, creator.toBuffer(), u64Le(swarmId)],
    PROGRAM_ID,
  );
}

export function findMembershipPda(
  swarm: PublicKey,
  agent: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [MEMBER_SEED, swarm.toBuffer(), agent.toBuffer()],
    PROGRAM_ID,
  );
}

export function findTaskPda(
  swarm: PublicKey,
  taskId: BN | number,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TASK_SEED, swarm.toBuffer(), u64Le(taskId)],
    PROGRAM_ID,
  );
}

export function findApprovalPda(
  task: PublicKey,
  approverAgent: PublicKey,
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [APPROVAL_SEED, task.toBuffer(), approverAgent.toBuffer()],
    PROGRAM_ID,
  );
}

export function findTreasuryPda(swarm: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [TREASURY_SEED, swarm.toBuffer()],
    PROGRAM_ID,
  );
}
