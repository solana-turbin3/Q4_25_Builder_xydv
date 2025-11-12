use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
    },
    types::UpdateTaskQueueArgsV0,
    TaskQueueAuthorityV0, TransactionSourceV0, TriggerV0,
};

use crate::{
    events::CancelSubscriptionEvent,
    states::{UserSubscription, SUBSCRIPTION_SEED},
};

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        mut,
        close = subscriber,
        seeds = [SUBSCRIPTION_SEED, subscriber.key.as_ref(), user_subscription.subscription.key().as_ref()],
        bump = user_subscription.bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,

    #[account(
        // address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = subscriber,
        associated_token::token_program = token_program
    )]
    pub subscriber_ata: InterfaceAccount<'info, TokenAccount>,

    // PROGRAMS
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelSubscription<'info> {
    pub fn cancel_subscription(&mut self) -> Result<()> {
        self.close_cron()?;

        emit!(CancelSubscriptionEvent {
            subscriber: self.subscriber.key(),
            subscription: self.user_subscription.subscription.key()
        });

        Ok(())
    }

    pub fn close_cron(&mut self) -> Result<()> {
        todo!()
    }
}
