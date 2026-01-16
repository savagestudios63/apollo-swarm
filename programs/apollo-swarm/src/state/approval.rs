use anchor_lang::prelude::*;

#[account]
#[derive(Debug)]
pub struct Approval {
    pub task: Pubkey,
    pub approver: Pubkey,
    pub approved_at: i64,
    pub bump: u8,
}

impl Approval {
    // disc(8) + task(32) + approver(32) + approved_at(8) + bump(1)
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1;
}
