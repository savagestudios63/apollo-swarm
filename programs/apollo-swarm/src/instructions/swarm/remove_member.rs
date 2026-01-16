use anchor_lang::prelude::*;

use crate::constants::{MEMBER_SEED, SWARM_SEED};
use crate::errors::ApolloError;
use crate::events::MemberRemoved;
use crate::state::{Membership, Swarm};

#[derive(Accounts)]
pub struct RemoveMember<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
        has_one = authority @ ApolloError::UnauthorizedSwarm,
    )]
    pub swarm: Account<'info, Swarm>,

    #[account(
        mut,
        close = authority,
        seeds = [MEMBER_SEED, swarm.key().as_ref(), membership.agent.as_ref()],
        bump = membership.bump,
        constraint = membership.swarm == swarm.key() @ ApolloError::NotMember,
    )]
    pub membership: Account<'info, Membership>,
}

pub fn handler(ctx: Context<RemoveMember>) -> Result<()> {
    let swarm = &mut ctx.accounts.swarm;
    let membership = &ctx.accounts.membership;

    swarm.member_count = swarm
        .member_count
        .checked_sub(1)
        .ok_or(ApolloError::InvalidMemberCount)?;
    swarm.total_share_bps = swarm
        .total_share_bps
        .checked_sub(membership.share_bps)
        .ok_or(ApolloError::MathOverflow)?;

    emit!(MemberRemoved {
        swarm: swarm.key(),
        agent: membership.agent,
    });

    Ok(())
}
