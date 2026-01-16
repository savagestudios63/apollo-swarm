use anchor_lang::prelude::*;

use crate::constants::{
    AGENT_SEED, MAX_METADATA_URI_LEN, MAX_ROLE_LEN, MEMBER_SEED, SWARM_SEED, TASK_SEED,
};
use crate::errors::ApolloError;
use crate::events::TaskCreated;
use crate::state::{Agent, Membership, Swarm, Task, TaskState};

#[derive(Accounts)]
#[instruction(task_id: u64)]
pub struct CreateTask<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [AGENT_SEED, creator.key().as_ref()],
        bump = creator_agent.bump,
        has_one = authority @ ApolloError::UnauthorizedAgent,
        constraint = creator_agent.authority == creator.key(),
    )]
    pub creator_agent: Account<'info, Agent>,

    /// CHECK: Linked via has_one on creator_agent.
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
    )]
    pub swarm: Account<'info, Swarm>,

    #[account(
        seeds = [MEMBER_SEED, swarm.key().as_ref(), creator_agent.key().as_ref()],
        bump = creator_membership.bump,
        constraint = creator_membership.swarm == swarm.key() @ ApolloError::NotMember,
        constraint = creator_membership.agent == creator_agent.key() @ ApolloError::NotMember,
    )]
    pub creator_membership: Account<'info, Membership>,

    #[account(
        init,
        payer = creator,
        space = Task::SPACE,
        seeds = [TASK_SEED, swarm.key().as_ref(), &task_id.to_le_bytes()],
        bump,
    )]
    pub task: Account<'info, Task>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateTask>,
    task_id: u64,
    required_role: String,
    payload_uri: String,
    reward: u64,
    work_based: bool,
) -> Result<()> {
    require!(
        required_role.len() <= MAX_ROLE_LEN,
        ApolloError::StringTooLong
    );
    require!(
        payload_uri.len() <= MAX_METADATA_URI_LEN,
        ApolloError::StringTooLong
    );
    require!(
        ctx.accounts.creator_agent.is_active(),
        ApolloError::AgentInactive
    );

    let swarm = &mut ctx.accounts.swarm;
    require!(
        task_id == swarm.task_count,
        ApolloError::InvalidTaskState
    );

    let task = &mut ctx.accounts.task;
    task.swarm = swarm.key();
    task.creator = ctx.accounts.creator_agent.key();
    task.executor = Pubkey::default();
    task.task_id = task_id;
    task.required_role = required_role.clone();
    task.payload_uri = payload_uri;
    task.result_uri = String::new();
    task.failure_reason = String::new();
    task.reward = reward;
    task.state = TaskState::Created as u8;
    task.approvals = 0;
    task.work_based = work_based;
    task.created_at = Clock::get()?.unix_timestamp;
    task.accepted_at = 0;
    task.completed_at = 0;
    task.bump = ctx.bumps.task;

    swarm.task_count = swarm
        .task_count
        .checked_add(1)
        .ok_or(ApolloError::MathOverflow)?;

    emit!(TaskCreated {
        task: task.key(),
        swarm: swarm.key(),
        task_id,
        required_role,
        reward,
    });

    Ok(())
}
