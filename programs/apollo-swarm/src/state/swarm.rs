use anchor_lang::prelude::*;

use crate::constants::{MAX_METADATA_URI_LEN, MAX_NAME_LEN};

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoordModel {
    RoleBased = 0,
    ThresholdApproval = 1,
    RoundRobin = 2,
    Voting = 3,
}

impl From<u8> for CoordModel {
    fn from(v: u8) -> Self {
        match v {
            1 => CoordModel::ThresholdApproval,
            2 => CoordModel::RoundRobin,
            3 => CoordModel::Voting,
            _ => CoordModel::RoleBased,
        }
    }
}

#[account]
#[derive(Debug)]
pub struct Swarm {
    pub authority: Pubkey,
    pub swarm_id: u64,
    pub name: String,
    pub metadata_uri: String,
    pub coord_model: u8,
    pub approval_threshold: u16,
    pub member_count: u32,
    pub task_count: u64,
    pub total_share_bps: u16,
    pub treasury_bump: u8,
    pub created_at: i64,
    pub bump: u8,
}

impl Swarm {
    // discriminator(8) + authority(32) + swarm_id(8) + name(4+MAX_NAME_LEN)
    // + metadata_uri(4+MAX_METADATA_URI_LEN) + coord_model(1) + approval_threshold(2)
    // + member_count(4) + task_count(8) + total_share_bps(2) + treasury_bump(1)
    // + created_at(8) + bump(1)
    pub const SPACE: usize = 8
        + 32
        + 8
        + (4 + MAX_NAME_LEN)
        + (4 + MAX_METADATA_URI_LEN)
        + 1
        + 2
        + 4
        + 8
        + 2
        + 1
        + 8
        + 1;
}
