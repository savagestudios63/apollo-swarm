use anchor_lang::prelude::*;

use crate::constants::{
    AGENT_SEED, BPS_DENOMINATOR, MAX_ROLE_LEN, MEMBER_SEED, SWARM_SEED,
};
use crate::errors::ApolloError;
use crate::events::MemberAdded;
use crate::state::{Agent, Membership, Swarm};

#[derive(Accounts)]
pub struct AddMember<'info> {
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
        seeds = [AGENT_SEED, agent.authority.as_ref()],
        bump = agent.bump,
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        init,
        payer = authority,
        space = Membership::SPACE,
        seeds = [MEMBER_SEED, swarm.key().as_ref(), agent.key().as_ref()],
        bump,
    )]
    pub membership: Account<'info, Membership>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AddMember>, role: String, share_bps: u16) -> Result<()> {
    require!(role.len() <= MAX_ROLE_LEN, ApolloError::StringTooLong);
    require!(ctx.accounts.agent.is_active(), ApolloError::AgentInactive);

    let swarm = &mut ctx.accounts.swarm;
    let new_total = swarm
        .total_share_bps
        .checked_add(share_bps)
        .ok_or(ApolloError::MathOverflow)?;
    require!(new_total <= BPS_DENOMINATOR, ApolloError::ShareOverflow);

    let membership = &mut ctx.accounts.membership;
    membership.swarm = swarm.key();
    membership.agent = ctx.accounts.agent.key();
    membership.role = role.clone();
    membership.share_bps = share_bps;
    membership.tasks_executed = 0;
    membership.joined_at = Clock::get()?.unix_timestamp;
    membership.bump = ctx.bumps.membership;

    swarm.member_count = swarm
        .member_count
        .checked_add(1)
        .ok_or(ApolloError::MathOverflow)?;
    swarm.total_share_bps = new_total;

    emit!(MemberAdded {
        swarm: swarm.key(),
        agent: membership.agent,
        share_bps,
        role,
    });

    Ok(())
}
