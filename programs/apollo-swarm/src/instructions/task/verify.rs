use anchor_lang::prelude::*;

use crate::constants::{AGENT_SEED, SWARM_SEED, TASK_SEED};
use crate::errors::ApolloError;
use crate::events::TaskVerified;
use crate::state::{Agent, CoordModel, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct VerifyTask<'info> {
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [AGENT_SEED, task.executor.as_ref()],
        bump = executor_agent.bump,
        constraint = executor_agent.key() == task.executor @ ApolloError::ExecutorMismatch,
    )]
    pub executor_agent: Account<'info, Agent>,

    #[account(
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
    )]
    pub swarm: Account<'info, Swarm>,

    #[account(
        mut,
        seeds = [TASK_SEED, swarm.key().as_ref(), &task.task_id.to_le_bytes()],
        bump = task.bump,
        constraint = task.swarm == swarm.key() @ ApolloError::ExecutorMismatch,
    )]
    pub task: Account<'info, Task>,
}

pub fn handler(ctx: Context<VerifyTask>) -> Result<()> {
    let swarm = &ctx.accounts.swarm;
    let task = &mut ctx.accounts.task;

    require!(
        task.state == TaskState::Completed as u8,
        ApolloError::InvalidTaskState
    );

    match CoordModel::from(swarm.coord_model) {
        CoordModel::RoleBased | CoordModel::RoundRobin => {
            // Only the swarm authority can verify.
            require!(
                swarm.authority == ctx.accounts.signer.key(),
                ApolloError::UnauthorizedSwarm
            );
        }
        CoordModel::ThresholdApproval | CoordModel::Voting => {
            require!(
                task.approvals >= swarm.approval_threshold,
                ApolloError::ThresholdNotMet
            );
        }
    }

    task.state = TaskState::Verified as u8;

    let exec = &mut ctx.accounts.executor_agent;
    exec.tasks_completed = exec.tasks_completed.saturating_add(1);
    exec.reputation = exec.reputation.saturating_add(1);

    emit!(TaskVerified { task: task.key() });

    Ok(())
}
