#![allow(warnings)]

use std::sync::{Arc, Mutex};

use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::Token,
    token_interface::{
        transfer_checked, Mint, PermanentDelegateInitialize, TokenAccount, TokenInterface,
        TransferChecked,
    },
};
use tuktuk_program::{
    compile_transaction,
    cron::{
        cpi::{accounts::AddCronTransactionV0, add_cron_transaction_v0},
        program::Cron,
        types::{AddCronTransactionArgsV0, TransactionSourceV0},
    },
    tuktuk::program::Tuktuk,
};

use crate::{
    error::SubscriptionError,
    states::{SubscriptionPlan, UserSubscription, SUBSCRIPTION_SEED},
};

#[derive(Accounts)]
pub struct ChargeUser<'info> {
    #[account(mut)]
    /// CHECK: called via tuktuk
    pub subscriber: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: called via tuktuk
    pub merchant: AccountInfo<'info>,
    #[account(
        seeds = [SUBSCRIPTION_SEED, subscriber.key.as_ref(), subscription_plan.key().as_ref()],
        bump = user_subscription.bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    #[account(
        constraint = subscription_plan.active @ SubscriptionError::InactivePlan
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = subscriber,
        associated_token::token_program = token_program
    )]
    pub subscriber_mint_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = merchant,
        associated_token::token_program = token_program
    )]
    pub merchant_mint_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    // programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub cron_program: Program<'info, Cron>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChargeUser<'info> {
    pub fn transfer_tokens(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            SUBSCRIPTION_SEED,
            &self.subscriber.key().to_bytes(),
            &self.subscription_plan.key().to_bytes(),
            &[self.user_subscription.bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.subscriber_mint_ata.to_account_info(),
                to: self.merchant_mint_ata.to_account_info(),
                mint: self.mint.to_account_info(),
                authority: self.user_subscription.to_account_info(),
            },
            signer_seeds,
        );

        transfer_checked(ctx, self.subscription_plan.amount, self.mint.decimals)
    }

    pub fn add_cron_transaction(&mut self) -> Result<()> {
        let ixs = vec![Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::ChargeUser {
                subscriber: self.subscriber.key(),
                merchant: self.merchant.key(),
                user_subscription: self.user_subscription.key(),
                subscription_plan: self.subscription_plan.key(),
                subscriber_mint_ata: self.subscriber_mint_ata.key(),
                merchant_mint_ata: self.merchant_mint_ata.key(),
                mint: self.mint.key(),
                associated_token_program: self.associated_token_program.key(),
                token_program: self.token_program.key(),
                tuktuk_program: self.tuktuk_program.key(),
                cron_program: self.cron_program.key(),
                system_program: self.system_program.key(),
            }
            .to_account_metas(None),
            data: crate::instruction::ChargeUser.data(),
        }];

        let (compiled_tx, _) = compile_transaction(ixs, vec![])?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"queue_authority",
            &[self.user_subscription.queue_authority_bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.cron_program.to_account_info(),
            AddCronTransactionV0 {
                payer: todo!(),
                authority: todo!(),
                cron_job: todo!(),
                cron_job_transaction: todo!(),
                system_program: todo!(),
            },
            signer_seeds,
        );

        add_cron_transaction_v0(
            ctx,
            AddCronTransactionArgsV0 {
                index: todo!(), // todo: add state
                transaction_source: TransactionSourceV0::CompiledV0(compiled_tx.into()),
            },
        )
    }

    pub fn requeue_task(&mut self) -> Result<()> {
        require_gte!(
            self.subscription_plan.max_failure_count,
            self.user_subscription.failure_count,
            SubscriptionError::MaxRetriesReached
        );

        todo!("create task for next day");

        match self.user_subscription.failure_count.checked_add(1) {
            Some(x) => self.user_subscription.failure_count = x,
            None => return err!(SubscriptionError::ArithmeticError),
        };

        Ok(())
    }
}
