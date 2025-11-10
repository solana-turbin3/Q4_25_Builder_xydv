use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use tuktuk_program::{
    cron::program::Cron, tuktuk::program::Tuktuk, TaskQueueAuthorityV0, TaskQueueV0,
};

use crate::{
    error::SubscriptionError,
    events::SubscribeEvent,
    states::{
        Status, SubscriptionPlan, UserSubscription, SUBSCRIBER_VAULT_SEED, SUBSCRIPTION_SEED,
    },
};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Subscribe<'info> {
    #[account(mut)]
    pub subscriber: Signer<'info>,
    #[account(
        init,
        payer = subscriber,
        space = UserSubscription::DISCRIMINATOR.len() + UserSubscription::INIT_SPACE,
        seeds = [SUBSCRIPTION_SEED, subscriber.key.as_ref(), subscription_plan.key().as_ref()],
        bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    #[account(
        constraint = subscription_plan.active @ SubscriptionError::InactivePlan
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    #[account(
        address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = subscriber,
        associated_token::mint = mint,
        associated_token::authority = subscriber,
        associated_token::token_program = token_program
    )]
    pub subscriber_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = subscriber,
        seeds = [SUBSCRIBER_VAULT_SEED, subscriber.key.as_ref()],
        token::mint = mint,
        token::authority = subscriber_vault,
        token::token_program = token_program,
        bump
    )]
    pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,

    // TUKTUK ACCOUNTS
    #[account(mut)]
    pub task_queue: Account<'info, TaskQueueV0>,
    #[account(
      seeds = [b"task_queue_authority", task_queue.key().as_ref(), queue_authority.key().as_ref()],
      bump = task_queue_authority.bump_seed,
      seeds::program = tuktuk_program::tuktuk::ID,
    )]
    pub task_queue_authority: Account<'info, TaskQueueAuthorityV0>,
    #[account(
          seeds = [b"queue_authority"],
          bump
    )]
    /// CHECK: This is a PDA that will be the authority on the task queue
    pub queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: Initialized in CPI
    pub task: AccountInfo<'info>,
    // PROGRAMS
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub cron_program: Program<'info, Cron>,
    pub system_program: Program<'info, System>,
}

impl<'info> Subscribe<'info> {
    pub fn subscribe(&mut self, bumps: &SubscribeBumps) -> Result<()> {
        // think about extra security checks??

        self.transfer(self.subscription_plan.amount)?;

        self.user_subscription.set_inner(UserSubscription {
            subscriber: self.subscriber.key(),
            subscriber_ata: self.subscriber_ata.key(),
            subscription: self.subscription_plan.key(),
            status: Status::Active,
            failure_count: 0,
            next_cron_transaction_id: 0,
            queue_authority_bump: bumps.queue_authority,
            last_exec_ts: Clock::get()?.unix_timestamp,
            subscriber_vault_bump: bumps.subscriber_vault,
            bump: bumps.user_subscription,
        });

        // emit events so that it can be used as trigger for merchant backend
        emit!(SubscribeEvent {
            subscriber: self.subscriber.key(),
            subscription: self.subscription_plan.key(),
            status: Status::Active
        });

        Ok(())
    }

    // transfer one cycle plan amount on subscribe
    pub fn transfer(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.subscriber_ata.to_account_info(),
                mint: self.mint.to_account_info(),
                to: self.subscriber_vault.to_account_info(),
                authority: self.subscriber.to_account_info(),
            },
        );

        transfer_checked(ctx, amount, self.mint.decimals)
    }
}
