use anchor_lang::prelude::*;

use crate::constants::{
    BPS_DENOMINATOR, MEMBER_SEED, SWARM_SEED, TASK_SEED, TREASURY_SEED,
};
use crate::errors::ApolloError;
use crate::events::TaskSettled;
use crate::state::{Membership, Swarm, Task, TaskState};

#[derive(Accounts)]
pub struct SettleTask<'info> {
    pub signer: Signer<'info>,

    #[account(
        mut,
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

    /// CHECK: System PDA holding treasury lamports.
    #[account(
        mut,
        seeds = [TREASURY_SEED, swarm.key().as_ref()],
        bump = swarm.treasury_bump,
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Settle a verified task.
///
/// `remaining_accounts` layout (alternating pairs):
///     [ Membership_0, recipient_0, Membership_1, recipient_1, ... ]
///
/// * Work-based tasks: exactly one pair is required — the executor's
///   membership and the wallet that should receive the full reward.
/// * Share-based tasks: one pair per paid member; sum of their `share_bps`
///   must equal `swarm.total_share_bps`. Each member receives
///   `reward * member.share_bps / swarm.total_share_bps`.
pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, SettleTask<'info>>) -> Result<()> {
    let swarm_key = ctx.accounts.swarm.key();
    let task = &mut ctx.accounts.task;
    require!(
        task.state == TaskState::Verified as u8,
        ApolloError::InvalidTaskState
    );

    let treasury_info = ctx.accounts.treasury.to_account_info();
    let reward = task.reward;
    let treasury_balance = treasury_info.lamports();
    require!(treasury_balance >= reward, ApolloError::InsufficientTreasury);

    let remaining = ctx.remaining_accounts;
    require!(remaining.len() % 2 == 0, ApolloError::InvalidMemberCount);
    require!(!remaining.is_empty(), ApolloError::InvalidMemberCount);

    let treasury_seeds: &[&[u8]] = &[
        TREASURY_SEED,
        swarm_key.as_ref(),
        std::slice::from_ref(&ctx.accounts.swarm.treasury_bump),
    ];
    let signer_seeds = &[treasury_seeds];

    let mut total_paid: u64 = 0;

    if task.work_based {
        require!(remaining.len() == 2, ApolloError::InvalidMemberCount);
        let (member_acc, recipient_acc) = (&remaining[0], &remaining[1]);
        let membership = parse_membership(member_acc, &swarm_key)?;
        require!(
            membership.agent == task.executor,
            ApolloError::ExecutorMismatch
        );
        transfer_from_treasury(
            &treasury_info,
            recipient_acc,
            &ctx.accounts.system_program,
            signer_seeds,
            reward,
        )?;
        total_paid = reward;
    } else {
        let swarm = &ctx.accounts.swarm;
        let divisor = if swarm.total_share_bps == 0 {
            BPS_DENOMINATOR
        } else {
            swarm.total_share_bps
        };

        let mut share_sum: u16 = 0;
        let pairs = remaining.chunks_exact(2);
        for pair in pairs {
            let membership = parse_membership(&pair[0], &swarm_key)?;
            share_sum = share_sum
                .checked_add(membership.share_bps)
                .ok_or(ApolloError::MathOverflow)?;
            let amount = (reward as u128)
                .checked_mul(membership.share_bps as u128)
                .ok_or(ApolloError::MathOverflow)?
                .checked_div(divisor as u128)
                .ok_or(ApolloError::MathOverflow)? as u64;
            if amount > 0 {
                transfer_from_treasury(
                    &treasury_info,
                    &pair[1],
                    &ctx.accounts.system_program,
                    signer_seeds,
                    amount,
                )?;
                total_paid = total_paid
                    .checked_add(amount)
                    .ok_or(ApolloError::MathOverflow)?;
            }
        }

        require!(
            share_sum == swarm.total_share_bps,
            ApolloError::InvalidMemberCount
        );
    }

    task.state = TaskState::Settled as u8;

    emit!(TaskSettled {
        task: task.key(),
        swarm: swarm_key,
        total_paid,
        work_based: task.work_based,
    });

    Ok(())
}

fn parse_membership(info: &AccountInfo, expected_swarm: &Pubkey) -> Result<Membership> {
    let membership: Account<Membership> = Account::try_from(info)?;
    require!(
        membership.swarm == *expected_swarm,
        ApolloError::NotMember
    );

    let expected = Pubkey::create_program_address(
        &[
            MEMBER_SEED,
            expected_swarm.as_ref(),
            membership.agent.as_ref(),
            std::slice::from_ref(&membership.bump),
        ],
        &crate::ID,
    )
    .map_err(|_| ApolloError::NotMember)?;
    require_keys_eq!(expected, info.key(), ApolloError::NotMember);

    Ok(membership.into_inner())
}

fn transfer_from_treasury<'info>(
    treasury: &AccountInfo<'info>,
    recipient: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let cpi_ctx = CpiContext::new_with_signer(
        system_program.to_account_info(),
        anchor_lang::system_program::Transfer {
            from: treasury.clone(),
            to: recipient.clone(),
        },
        signer_seeds,
    );
    anchor_lang::system_program::transfer(cpi_ctx, amount)
}
