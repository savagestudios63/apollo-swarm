use anchor_lang::prelude::*;

use crate::constants::{MAX_METADATA_URI_LEN, MAX_REASON_LEN, MAX_ROLE_LEN};

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Created = 0,
    Accepted = 1,
    Completed = 2,
    Verified = 3,
    Failed = 4,
    Settled = 5,
}

impl From<u8> for TaskState {
    fn from(v: u8) -> Self {
        match v {
            1 => TaskState::Accepted,
            2 => TaskState::Completed,
            3 => TaskState::Verified,
            4 => TaskState::Failed,
            5 => TaskState::Settled,
            _ => TaskState::Created,
        }
    }
}

#[account]
#[derive(Debug)]
pub struct Task {
    pub swarm: Pubkey,
    pub creator: Pubkey,
    pub executor: Pubkey,
    pub task_id: u64,
    pub required_role: String,
    pub payload_uri: String,
    pub result_uri: String,
    pub failure_reason: String,
    pub reward: u64,
    pub state: u8,
    pub approvals: u16,
    pub work_based: bool,
    pub created_at: i64,
    pub accepted_at: i64,
    pub completed_at: i64,
    pub bump: u8,
}

impl Task {
    // disc(8) + swarm(32) + creator(32) + executor(32) + task_id(8)
    // + required_role(4+MAX_ROLE_LEN) + payload_uri(4+MAX_METADATA_URI_LEN)
    // + result_uri(4+MAX_METADATA_URI_LEN) + failure_reason(4+MAX_REASON_LEN)
    // + reward(8) + state(1) + approvals(2) + work_based(1)
    // + created_at(8) + accepted_at(8) + completed_at(8) + bump(1)
    pub const SPACE: usize = 8
        + 32
        + 32
        + 32
        + 8
        + (4 + MAX_ROLE_LEN)
        + (4 + MAX_METADATA_URI_LEN)
        + (4 + MAX_METADATA_URI_LEN)
        + (4 + MAX_REASON_LEN)
        + 8
        + 1
        + 2
        + 1
        + 8
        + 8
        + 8
        + 1;
}
