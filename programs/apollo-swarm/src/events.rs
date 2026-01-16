use anchor_lang::prelude::*;

#[event]
pub struct AgentRegistered {
    pub agent: Pubkey,
    pub authority: Pubkey,
    pub role: String,
}

#[event]
pub struct AgentUpdated {
    pub agent: Pubkey,
    pub status: u8,
}

#[event]
pub struct SwarmCreated {
    pub swarm: Pubkey,
    pub creator: Pubkey,
    pub swarm_id: u64,
    pub coord_model: u8,
}

#[event]
pub struct MemberAdded {
    pub swarm: Pubkey,
    pub agent: Pubkey,
    pub share_bps: u16,
    pub role: String,
}

#[event]
pub struct MemberRemoved {
    pub swarm: Pubkey,
    pub agent: Pubkey,
}

#[event]
pub struct TaskCreated {
    pub task: Pubkey,
    pub swarm: Pubkey,
    pub task_id: u64,
    pub required_role: String,
    pub reward: u64,
}

#[event]
pub struct TaskAccepted {
    pub task: Pubkey,
    pub executor: Pubkey,
}

#[event]
pub struct TaskCompleted {
    pub task: Pubkey,
    pub executor: Pubkey,
    pub result_uri: String,
}

#[event]
pub struct TaskApproved {
    pub task: Pubkey,
    pub approver: Pubkey,
    pub approvals: u16,
    pub threshold: u16,
}

#[event]
pub struct TaskVerified {
    pub task: Pubkey,
}

#[event]
pub struct TaskFailed {
    pub task: Pubkey,
    pub reason: String,
}

#[event]
pub struct TreasuryDeposit {
    pub swarm: Pubkey,
    pub from: Pubkey,
    pub amount: u64,
}

#[event]
pub struct TaskSettled {
    pub task: Pubkey,
    pub swarm: Pubkey,
    pub total_paid: u64,
    pub work_based: bool,
}
