use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, MEMBER_SEED, SWARM_SEED, TASK_SEED};
use crate::errors::ApolloError;
use crate::events::TaskAccepted;
use crate::state::{Agent, Membership, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct AcceptTask<'info> {
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
        seeds = [MEMBER_SEED, swarm.key().as_ref(), executor_agent.key().as_ref()],
        bump = membership.bump,
        constraint = membership.swarm == swarm.key() @ ApolloError::NotMember,
        constraint = membership.agent == executor_agent.key() @ ApolloError::NotMember,
    )]
    pub membership: Account<'info, Membership>,

    #[account(
        mut,
        seeds = [TASK_SEED, swarm.key().as_ref(), &task.task_id.to_le_bytes()],
        bump = task.bump,
        constraint = task.swarm == swarm.key() @ ApolloError::ExecutorMismatch,
    )]
    pub task: Account<'info, Task>,
}

pub fn handler(ctx: Context<AcceptTask>) -> Result<()> {
    require!(
        ctx.accounts.executor_agent.is_active(),
        ApolloError::AgentInactive
    );

    let task = &mut ctx.accounts.task;
    require!(
        task.state == TaskState::Created as u8,
        ApolloError::InvalidTaskState
    );

    if !task.required_role.is_empty() {
        require!(
            task.required_role == ctx.accounts.membership.role
                || task.required_role == ctx.accounts.executor_agent.role,
            ApolloError::RoleMismatch
        );
    }

    task.executor = ctx.accounts.executor_agent.key();
    task.state = TaskState::Accepted as u8;
    task.accepted_at = Clock::get()?.unix_timestamp;

    emit!(TaskAccepted {
        task: task.key(),
        executor: task.executor,
    });

    Ok(())
}
