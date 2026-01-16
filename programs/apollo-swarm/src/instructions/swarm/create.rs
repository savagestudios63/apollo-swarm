use anchor_lang::prelude::*;

use crate::constants::{
    MAX_METADATA_URI_LEN, MAX_NAME_LEN, SWARM_SEED, TREASURY_SEED,
};
use crate::errors::ApolloError;
use crate::events::SwarmCreated;
use crate::state::{CoordModel, Swarm};

#[derive(Accounts)]
#[instruction(swarm_id: u64)]
pub struct CreateSwarm<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Swarm::SPACE,
        seeds = [SWARM_SEED, authority.key().as_ref(), &swarm_id.to_le_bytes()],
        bump,
    )]
    pub swarm: Account<'info, Swarm>,

    /// CHECK: System-owned PDA that holds treasury lamports for this swarm.
    #[account(
        seeds = [TREASURY_SEED, swarm.key().as_ref()],
        bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateSwarm>,
    swarm_id: u64,
    name: String,
    metadata_uri: String,
    coord_model: u8,
    approval_threshold: u16,
) -> Result<()> {
    require!(name.len() <= MAX_NAME_LEN, ApolloError::StringTooLong);
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        ApolloError::StringTooLong
    );
    require!(coord_model <= 3, ApolloError::UnsupportedCoordination);

    let swarm = &mut ctx.accounts.swarm;
    swarm.authority = ctx.accounts.authority.key();
    swarm.swarm_id = swarm_id;
    swarm.name = name;
    swarm.metadata_uri = metadata_uri;
    swarm.coord_model = coord_model;
    swarm.approval_threshold = approval_threshold;
    swarm.member_count = 0;
    swarm.task_count = 0;
    swarm.total_share_bps = 0;
    swarm.treasury_bump = ctx.bumps.treasury;
    swarm.created_at = Clock::get()?.unix_timestamp;
    swarm.bump = ctx.bumps.swarm;

    emit!(SwarmCreated {
        swarm: swarm.key(),
        creator: swarm.authority,
        swarm_id,
        coord_model,
    });

    let _ = CoordModel::from(coord_model);
    Ok(())
}
