use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, MAX_REASON_LEN, SWARM_SEED, TASK_SEED};
use crate::errors::ApolloError;
use crate::events::TaskFailed;
use crate::state::{Agent, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct FailTask<'info> {
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, signer.key().as_ref()],
        bump = agent.bump,
        has_one = authority @ ApolloError::UnauthorizedAgent,
    )]
    pub agent: Account<'info, Agent>,

    /// CHECK: Linked via has_one.
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
    )]
    pub swarm: Account<'info, Swarm>,

    #[account(
        mut,
        seeds = [TASK_SEED, swarm.key().as_ref(), &task.task_id.to_le_bytes()],
        bump = task.bump,
    )]
    pub task: Account<'info, Task>,
}

pub fn handler(ctx: Context<FailTask>, reason: String) -> Result<()> {
    require!(reason.len() <= MAX_REASON_LEN, ApolloError::StringTooLong);

    let agent_key = ctx.accounts.agent.key();
    let task = &mut ctx.accounts.task;

    // Either the executor (self-fail) or the swarm authority can fail a task.
    let is_executor = task.executor == agent_key;
    let is_swarm_authority = ctx.accounts.swarm.authority == ctx.accounts.signer.key();
    require!(is_executor || is_swarm_authority, ApolloError::UnauthorizedAgent);

    require!(
        task.state == TaskState::Created as u8
            || task.state == TaskState::Accepted as u8
            || task.state == TaskState::Completed as u8,
        ApolloError::InvalidTaskState
    );

    task.failure_reason = reason.clone();
    task.state = TaskState::Failed as u8;

    if is_executor {
        let agent = &mut ctx.accounts.agent;
        agent.tasks_failed = agent.tasks_failed.saturating_add(1);
        agent.reputation = agent.reputation.saturating_sub(1);
    }

    emit!(TaskFailed {
        task: task.key(),
        reason,
    });

    Ok(())
}
