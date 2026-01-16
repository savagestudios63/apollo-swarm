use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::constants::{SWARM_SEED, TREASURY_SEED};
use crate::events::TreasuryDeposit;
use crate::state::Swarm;

#[derive(Accounts)]
pub struct DepositTreasury<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [SWARM_SEED, swarm.authority.as_ref(), &swarm.swarm_id.to_le_bytes()],
        bump = swarm.bump,
    )]
    pub swarm: Account<'info, Swarm>,

    /// CHECK: System PDA holding treasury lamports.
    #[account(
        mut,
        seeds = [TREASURY_SEED, swarm.key().as_ref()],
        bump = swarm.treasury_bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<DepositTreasury>, amount: u64) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.payer.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, amount)?;

    emit!(TreasuryDeposit {
        swarm: ctx.accounts.swarm.key(),
        from: ctx.accounts.payer.key(),
        amount,
    });

    Ok(())
}
