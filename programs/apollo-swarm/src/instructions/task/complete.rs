use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, MAX_METADATA_URI_LEN, SWARM_SEED, TASK_SEED};
use crate::errors::ApolloError;
use crate::events::TaskCompleted;
use crate::state::{Agent, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct CompleteTask<'info> {
    pub executor: Signer<'info>,

    #[account(
        seeds = [AGENT_SEED, executor.key().as_ref()],
        bump = executor_agent.bump,
        has_one = authority @ ApolloError::UnauthorizedAgent,
    )]
    pub executor_agent: Account<'info, Agent>,

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
        constraint = task.executor == executor_agent.key() @ ApolloError::ExecutorMismatch,
    )]
    pub task: Account<'info, Task>,
}

pub fn handler(ctx: Context<CompleteTask>, result_uri: String) -> Result<()> {
    require!(
        result_uri.len() <= MAX_METADATA_URI_LEN,
        ApolloError::StringTooLong
    );

    let task = &mut ctx.accounts.task;
    require!(
        task.state == TaskState::Accepted as u8,
        ApolloError::InvalidTaskState
    );

    task.result_uri = result_uri.clone();
    task.state = TaskState::Completed as u8;
    task.completed_at = Clock::get()?.unix_timestamp;

    emit!(TaskCompleted {
        task: task.key(),
        executor: task.executor,
        result_uri,
    });

    Ok(())
}
