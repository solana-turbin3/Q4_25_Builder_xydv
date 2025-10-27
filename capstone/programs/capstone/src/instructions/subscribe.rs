use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{approve_checked, ApproveChecked, Mint, TokenAccount, TokenInterface},
};

use crate::{
    error::SubscriptionError,
    events::SubscribeEvent,
    states::{Status, SubscriptionPlan, UserSubscription, SUBSCRIPTION_SEED},
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
    pub subscriber_mint_ata: InterfaceAccount<'info, TokenAccount>, // ?? preinitialized??

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Subscribe<'info> {
    pub fn subscribe(&mut self, bumps: &SubscribeBumps) -> Result<()> {
        // think about extra security checks??

        self.delegate(self.subscription_plan.amount)?;

        self.user_subscription.set_inner(UserSubscription {
            subscriber: self.subscriber.key(),
            subscriber_ata: self.subscriber_mint_ata.key(),
            subscription: self.subscription_plan.key(),
            status: Status::Active,
            failure_count: 0,
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

    pub fn delegate(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            ApproveChecked {
                to: self.subscriber_mint_ata.to_account_info(),
                mint: self.mint.to_account_info(),
                delegate: self.user_subscription.to_account_info(), // users subscription account has delegate access (or should i delegate to a new only delegate pda?)
                authority: self.subscriber.to_account_info(),
            },
        );

        // is this amount used up once used or the max cap a delegate can transfer at once?
        approve_checked(ctx, amount, self.mint.decimals)
    }
}
