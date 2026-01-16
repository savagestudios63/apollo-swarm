use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("ApoLLoSwarm111111111111111111111111111111111");

#[program]
pub mod apollo_swarm {
    use super::*;

    // ---------- Agent Registry ----------

    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        role: String,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::agent::register::handler(ctx, role, metadata_uri)
    }

    pub fn update_agent(
        ctx: Context<UpdateAgent>,
        status: Option<u8>,
        metadata_uri: Option<String>,
    ) -> Result<()> {
        instructions::agent::update::handler(ctx, status, metadata_uri)
    }

    // ---------- Swarm Registry ----------

    pub fn create_swarm(
        ctx: Context<CreateSwarm>,
        swarm_id: u64,
        name: String,
        metadata_uri: String,
        coord_model: u8,
        approval_threshold: u16,
    ) -> Result<()> {
        instructions::swarm::create::handler(
            ctx,
            swarm_id,
            name,
            metadata_uri,
            coord_model,
            approval_threshold,
        )
    }

    pub fn add_member(
        ctx: Context<AddMember>,
        role: String,
        share_bps: u16,
    ) -> Result<()> {
        instructions::swarm::add_member::handler(ctx, role, share_bps)
    }

    pub fn remove_member(ctx: Context<RemoveMember>) -> Result<()> {
        instructions::swarm::remove_member::handler(ctx)
    }

    // ---------- Task Delegation ----------

    pub fn create_task(
        ctx: Context<CreateTask>,
        task_id: u64,
        required_role: String,
        payload_uri: String,
        reward: u64,
        work_based: bool,
    ) -> Result<()> {
        instructions::task::create::handler(
            ctx,
            task_id,
            required_role,
            payload_uri,
            reward,
            work_based,
        )
    }

    pub fn accept_task(ctx: Context<AcceptTask>) -> Result<()> {
        instructions::task::accept::handler(ctx)
    }

    pub fn complete_task(ctx: Context<CompleteTask>, result_uri: String) -> Result<()> {
        instructions::task::complete::handler(ctx, result_uri)
    }

    pub fn fail_task(ctx: Context<FailTask>, reason: String) -> Result<()> {
        instructions::task::fail::handler(ctx, reason)
    }

    // ---------- Coordination ----------

    pub fn approve_task(ctx: Context<ApproveTask>) -> Result<()> {
        instructions::task::approve::handler(ctx)
    }

    pub fn verify_task(ctx: Context<VerifyTask>) -> Result<()> {
        instructions::task::verify::handler(ctx)
    }

    // ---------- Treasury / Settlement ----------

    pub fn deposit_treasury(ctx: Context<DepositTreasury>, amount: u64) -> Result<()> {
        instructions::treasury::deposit::handler(ctx, amount)
    }

    pub fn settle_task<'info>(
        ctx: Context<'_, '_, 'info, 'info, SettleTask<'info>>,
    ) -> Result<()> {
        instructions::treasury::settle::handler(ctx)
    }
}
