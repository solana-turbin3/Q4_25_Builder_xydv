use std::str::FromStr;

use anchor_lang::{
    prelude::*,
    solana_program::hash::hash,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use clockwork_cron::Schedule;

use crate::{
    error::SubscriptionError,
    states::{SubscriptionPlan, PLAN_SEED, VAULT_SEED},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateSubscriptionArgs {
    pub name: String,
    pub amount: u64,
    pub schedule: String,
    pub max_failure_count: u8,
}

#[derive(Accounts)]
#[instruction(args: CreateSubscriptionArgs)]
pub struct CreateSubscription<'info> {
    #[account(mut)]
    pub merchant: Signer<'info>,
    #[account(
        init,
        payer = merchant,
        space = SubscriptionPlan::DISCRIMINATOR.len() + SubscriptionPlan::INIT_SPACE,
        seeds = [PLAN_SEED, merchant.key.as_ref(), {hash(args.name.as_bytes()).as_ref()}],
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
    pub merchant_mint_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateSubscription<'info> {
    // this function checks all the inputs and initializes the SubscriptionPlan PDA
    pub fn create_subscription(
        &mut self,
        args: CreateSubscriptionArgs,
        bumps: &CreateSubscriptionBumps,
    ) -> Result<()> {
        require!(args.amount > 0, SubscriptionError::InvalidAmount);
        require!(args.name.len() != 0, SubscriptionError::InvalidName);
        require!(
            Schedule::from_str(&args.schedule).is_ok(),
            SubscriptionError::InvalidSchedule
        );

        self.subscription_plan.set_inner(SubscriptionPlan {
            merchant: self.merchant.key(),
            mint: self.mint.key(),
            merchant_ata: self.merchant_mint_ata.key(),
            name: args.name,
            amount: args.amount,
            active: true,
            schedule: args.schedule,
            max_failure_count: args.max_failure_count,
            bump: bumps.subscription_plan,
        });

        Ok(())
    }

    // charge the one-time fees for merchant for each SubscriptionPlan initialization
    pub fn charge_fees(&mut self, lamports: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.merchant.to_account_info(),
                to: self.vault.to_account_info(),
            },
        );

        transfer(ctx, lamports)
    }
}
