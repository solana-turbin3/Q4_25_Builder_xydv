use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    error::SubscriptionError,
    states::{SubscriptionPlan, PLAN_SEED},
};

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,
    #[account(
        init,
        payer = merchant,
        space = SubscriptionPlan::DISCRIMINATOR.len() + SubscriptionPlan::INIT_SPACE,
        seeds = [PLAN_SEED, merchant.key.as_ref(), name.as_bytes().as_ref()],
        bump
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = merchant,
        associated_token::mint = mint,
        associated_token::authority = merchant,
        associated_token::token_program = token_program
    )]
    pub merchant_mint_ata: InterfaceAccount<'info, TokenAccount>, // ?? preinitialized??

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateSubscription<'info> {
    pub fn create_subscription(
        &mut self,
        name: String,
        amount: u64,
        schedule: String,
        max_failure_count: u8,
        bumps: &CreateSubscriptionBumps,
    ) -> Result<()> {
        require!(amount > 0, SubscriptionError::InvalidAmount);
        // name is used as a seed parameter
        require!(name.len() != 0, SubscriptionError::InvalidName);
        // todo: more checks on schedule??

        self.subscription_plan.set_inner(SubscriptionPlan {
            merchant: self.merchant.key(),
            mint: self.mint.key(),
            merchant_ata: self.merchant_mint_ata.key(),
            name,
            amount,
            active: true,
            schedule,
            max_failure_count,
            bump: bumps.subscription_plan,
        });

        Ok(())
    }
}
