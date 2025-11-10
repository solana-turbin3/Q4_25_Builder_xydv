// this instruction is recurring task

use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
    },
    types::QueueTaskArgsV0,
    TaskQueueV0,
};

use crate::{
    error::SubscriptionError,
    states::{SubscriptionPlan, UserSubscription, SUBSCRIBER_VAULT_SEED, SUBSCRIPTION_SEED},
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
        seeds = [SUBSCRIBER_VAULT_SEED, subscriber.key.as_ref()],
        token::mint = mint,
        token::authority = subscriber_vault,
        token::token_program = token_program,
        bump = user_subscription.subscriber_vault_bump
    )]
    pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = merchant,
        associated_token::token_program = token_program
    )]
    pub merchant_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    // tuktuk
    #[account(mut)]
    pub task_queue: Account<'info, TaskQueueV0>,

    #[account(
        seeds = [b"queue_authority"],
        bump // optimize later
    )]
    pub queue_authority: UncheckedAccount<'info>,

    // programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChargeUser<'info> {
    pub fn transfer_tokens(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            SUBSCRIBER_VAULT_SEED,
            self.subscriber.key.as_ref(),
            &[self.user_subscription.subscriber_vault_bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.subscriber_vault.to_account_info(),
                to: self.merchant_ata.to_account_info(),
                mint: self.mint.to_account_info(),
                authority: self.subscriber_vault.to_account_info(),
            },
            signer_seeds,
        );

        transfer_checked(ctx, self.subscription_plan.amount, self.mint.decimals)
    }

    pub fn add_next_transaction_to_queue(&mut self) -> Result<()> {
        let ixs = vec![Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::ChargeUser {
                subscriber: self.subscriber.key(),
                merchant: self.merchant.key(),
                user_subscription: self.user_subscription.key(),
                subscription_plan: self.subscription_plan.key(),
                merchant_ata: self.merchant_ata.key(),
                mint: self.mint.key(),
                subscriber_vault: self.subscriber_vault.key(),
                task_queue: self.task_queue.key(),
                queue_authority: self.queue_authority.key(),
                associated_token_program: self.associated_token_program.key(),
                token_program: self.token_program.key(),
                tuktuk_program: self.tuktuk_program.key(),
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
            self.tuktuk_program.to_account_info(),
            QueueTaskV0 {
                payer: todo!(),
                queue_authority: self.queue_authority.to_account_info(),
                task_queue_authority: todo!(),
                task_queue: todo!(),
                task: todo!(),
                system_program: todo!(),
            },
            signer_seeds,
        );

        queue_task_v0(
            ctx,
            QueueTaskArgsV0 {
                id: todo!(),
                trigger: todo!(),
                transaction: todo!(),
                crank_reward: todo!(),
                free_tasks: todo!(),
                description: todo!(),
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

        // match self.user_subscription.failure_count.checked_add(1) {
        //     Some(x) => self.user_subscription.failure_count = x,
        //     None => return err!(SubscriptionError::ArithmeticError),
        // };

        // Ok(())
    }
}
