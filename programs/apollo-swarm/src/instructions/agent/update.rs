use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, MAX_METADATA_URI_LEN};
use crate::errors::ApolloError;
use crate::events::AgentUpdated;
use crate::state::Agent;

#[derive(Accounts)]
pub struct UpdateAgent<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, authority.key().as_ref()],
        bump = agent.bump,
        has_one = authority @ ApolloError::UnauthorizedAgent,
    )]
    pub agent: Account<'info, Agent>,
}

pub fn handler(
    ctx: Context<UpdateAgent>,
    status: Option<u8>,
    metadata_uri: Option<String>,
) -> Result<()> {
    let agent = &mut ctx.accounts.agent;

    if let Some(s) = status {
        require!(s <= 2, ApolloError::InvalidTaskState);
        agent.status = s;
    }

    if let Some(uri) = metadata_uri {
        require!(
            uri.len() <= MAX_METADATA_URI_LEN,
            ApolloError::StringTooLong
        );
        agent.metadata_uri = uri;
    }

    emit!(AgentUpdated {
        agent: agent.key(),
        status: agent.status,
    });

    Ok(())
}
