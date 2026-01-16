use anchor_lang::prelude::*;

#[constant]
pub const AGENT_SEED: &[u8] = b"agent";

#[constant]
pub const SWARM_SEED: &[u8] = b"swarm";

#[constant]
pub const MEMBER_SEED: &[u8] = b"member";

#[constant]
pub const TASK_SEED: &[u8] = b"task";

#[constant]
pub const APPROVAL_SEED: &[u8] = b"approval";

#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";

pub const MAX_METADATA_URI_LEN: usize = 200;
pub const MAX_NAME_LEN: usize = 64;
pub const MAX_ROLE_LEN: usize = 32;
pub const MAX_REASON_LEN: usize = 128;

pub const BPS_DENOMINATOR: u16 = 10_000;
