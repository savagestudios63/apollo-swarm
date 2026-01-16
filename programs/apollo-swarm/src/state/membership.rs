use anchor_lang::prelude::*;

use crate::constants::MAX_ROLE_LEN;

#[account]
#[derive(Debug)]
pub struct Membership {
    pub swarm: Pubkey,
    pub agent: Pubkey,
    pub role: String,
    pub share_bps: u16,
    pub tasks_executed: u64,
    pub joined_at: i64,
    pub bump: u8,
}

impl Membership {
    // disc(8) + swarm(32) + agent(32) + role(4+MAX_ROLE_LEN) + share(2)
    // + tasks_executed(8) + joined_at(8) + bump(1)
    pub const SPACE: usize = 8 + 32 + 32 + (4 + MAX_ROLE_LEN) + 2 + 8 + 8 + 1;
}
