use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, MAX_METADATA_URI_LEN, MAX_ROLE_LEN};
use crate::errors::ApolloError;
use crate::events::AgentRegistered;
use crate::state::{Agent, AgentStatus};

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Agent::SPACE,
        seeds = [AGENT_SEED, authority.key().as_ref()],
        bump,
    )]
    pub agent: Account<'info, Agent>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterAgent>,
    role: String,
    metadata_uri: String,
) -> Result<()> {
    require!(role.len() <= MAX_ROLE_LEN, ApolloError::StringTooLong);
    require!(
        metadata_uri.len() <= MAX_METADATA_URI_LEN,
        ApolloError::StringTooLong
    );

    let agent = &mut ctx.accounts.agent;
    agent.authority = ctx.accounts.authority.key();
    agent.role = role.clone();
    agent.metadata_uri = metadata_uri;
    agent.status = AgentStatus::Active as u8;
    agent.reputation = 0;
    agent.tasks_completed = 0;
    agent.tasks_failed = 0;
    agent.created_at = Clock::get()?.unix_timestamp;
    agent.bump = ctx.bumps.agent;

    emit!(AgentRegistered {
        agent: agent.key(),
        authority: agent.authority,
        role,
    });

    Ok(())
}
