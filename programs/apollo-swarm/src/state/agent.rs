use anchor_lang::prelude::*;

use crate::constants::{MAX_METADATA_URI_LEN, MAX_ROLE_LEN};

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentStatus {
    Active = 0,
    Paused = 1,
    Revoked = 2,
}

impl From<u8> for AgentStatus {
    fn from(v: u8) -> Self {
        match v {
            1 => AgentStatus::Paused,
            2 => AgentStatus::Revoked,
            _ => AgentStatus::Active,
        }
    }
}

#[account]
#[derive(Debug)]
pub struct Agent {
    pub authority: Pubkey,
    pub role: String,
    pub metadata_uri: String,
    pub status: u8,
    pub reputation: i64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl Agent {
    // discriminator(8) + authority(32) + role(4+MAX_ROLE_LEN)
    // + metadata_uri(4+MAX_METADATA_URI_LEN) + status(1) + reputation(8)
    // + tasks_completed(8) + tasks_failed(8) + created_at(8) + bump(1)
    pub const SPACE: usize = 8
        + 32
        + (4 + MAX_ROLE_LEN)
        + (4 + MAX_METADATA_URI_LEN)
        + 1
        + 8
        + 8
        + 8
        + 8
        + 1;

    pub fn is_active(&self) -> bool {
        self.status == AgentStatus::Active as u8
    }
}
