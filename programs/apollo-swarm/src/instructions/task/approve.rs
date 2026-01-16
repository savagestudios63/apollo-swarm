use anchor_lang::prelude::*;

use crate::constants::{
    AGENT_SEED, APPROVAL_SEED, MEMBER_SEED, SWARM_SEED, TASK_SEED,
};
use crate::errors::ApolloError;
use crate::events::TaskApproved;
use crate::state::{Agent, Approval, Membership, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct ApproveTask<'info> {
    #[account(mut)]
    pub approver: Signer<'info>,

    #[account(
        seeds = [AGENT_SEED, approver.key().as_ref()],
        bump = approver_agent.bump,
        has_one = authority @ ApolloError::UnauthorizedAgent,
    )]
    pub approver_agent: Account<'info, Agent>,

    /// CHECK: Linked via has_one.
    pub authority: UncheckedAccount<'info>,

    #[account(
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
    )]
    pub swarm: Account<'info, Swarm>,

    #[account(
        seeds = [MEMBER_SEED, swarm.key().as_ref(), approver_agent.key().as_ref()],
        bump = membership.bump,
        constraint = membership.swarm == swarm.key() @ ApolloError::NotMember,
        constraint = membership.agent == approver_agent.key() @ ApolloError::NotMember,
    )]
    pub membership: Account<'info, Membership>,

    #[account(
        mut,
        seeds = [TASK_SEED, swarm.key().as_ref(), &task.task_id.to_le_bytes()],
        bump = task.bump,
        constraint = task.swarm == swarm.key() @ ApolloError::ExecutorMismatch,
    )]
    pub task: Account<'info, Task>,

    #[account(
        init,
        payer = approver,
        space = Approval::SPACE,
        seeds = [APPROVAL_SEED, task.key().as_ref(), approver_agent.key().as_ref()],
        bump,
    )]
    pub approval: Account<'info, Approval>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApproveTask>) -> Result<()> {
    require!(
        ctx.accounts.approver_agent.is_active(),
        ApolloError::AgentInactive
    );

    let task = &mut ctx.accounts.task;
    require!(
        task.state == TaskState::Completed as u8,
        ApolloError::InvalidTaskState
    );
    require!(
        task.executor != ctx.accounts.approver_agent.key(),
        ApolloError::UnauthorizedAgent
    );

    let approval = &mut ctx.accounts.approval;
    approval.task = task.key();
    approval.approver = ctx.accounts.approver_agent.key();
    approval.approved_at = Clock::get()?.unix_timestamp;
    approval.bump = ctx.bumps.approval;

    task.approvals = task
        .approvals
        .checked_add(1)
        .ok_or(ApolloError::MathOverflow)?;

    emit!(TaskApproved {
        task: task.key(),
        approver: approval.approver,
        approvals: task.approvals,
        threshold: ctx.accounts.swarm.approval_threshold,
    });

    Ok(())
}
