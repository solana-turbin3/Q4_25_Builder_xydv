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
    TaskQueueAuthorityV0, TransactionSourceV0, TriggerV0,
};

use crate::{
    error::SubscriptionError,
    events::SubscribeEvent,
    states::{
        GlobalState, Status, SubscriptionPlan, UserSubscription, GLOBAL_STATE_SEED,
        QUEUE_AUTHORITY_SEED, SUBSCRIBER_VAULT_SEED, SUBSCRIPTION_SEED,
    },
};

#[derive(Accounts)]
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
        mut,
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

    #[account(
        seeds = [GLOBAL_STATE_SEED],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    // TUKTUK ACCOUNTS
    #[account(mut)]
    /// CHECK: via signer, only can call this instruction
    pub task_queue: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [QUEUE_AUTHORITY_SEED],
        bump = global_state.queue_authority_bump
    )]
    /// CHECK: via seeds
    pub queue_authority: UncheckedAccount<'info>,
    #[account(
      seeds = [b"task_queue_authority", task_queue.key().as_ref(), queue_authority.key().as_ref()],
      bump = task_queue_authority.bump_seed,
      seeds::program = tuktuk_program::tuktuk::ID,
    )]
    pub task_queue_authority: Account<'info, TaskQueueAuthorityV0>,
    #[account(mut)]
    /// CHECK: Initialized in CPI
    pub task: AccountInfo<'info>,

    // PROGRAMS
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub tuktuk_program: Program<'info, Tuktuk>,
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
            transaction_id: 0,
            last_exec_ts: Clock::get()?.unix_timestamp,
            subscriber_vault_bump: bumps.subscriber_vault,
            bump: bumps.user_subscription,
        });

        self.schedule(self.user_subscription.transaction_id)?;

        // emit events so that it can be used as trigger for merchant backend
        emit!(SubscribeEvent {
            subscriber: self.subscriber.key(),
            subscription: self.subscription_plan.key(),
            status: Status::Active
        });

        Ok(())
    }

    // transfer one cycle plan amount on subscribe (todo: if vault already has some usdc, dont transfer than)
    pub fn transfer(&mut self, amount: u64) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.subscriber_ata.to_account_info(),
                to: self.subscriber_vault.to_account_info(),
                mint: self.mint.to_account_info(),
                authority: self.subscriber.to_account_info(),
            },
        );

        transfer_checked(ctx, amount, self.mint.decimals)
    }

    pub fn schedule(&mut self, task_id: u16) -> Result<()> {
        let ixs = vec![Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::ChargeUserRecurring {
                subscriber: self.subscriber.key(),
                merchant: self.subscription_plan.merchant.key(),
                user_subscription: self.user_subscription.key(),
                subscription_plan: self.subscription_plan.key(),
                merchant_ata: self.subscription_plan.merchant_ata.key(),
                mint: self.mint.key(),
                subscriber_vault: self.subscriber_vault.key(),
                associated_token_program: self.associated_token_program.key(),
                token_program: self.token_program.key(),
                system_program: self.system_program.key(),
            }
            .to_account_metas(None),
            data: crate::instruction::ChargeUserRecurring.data(),
        }];

        let (compiled_tx, _) = compile_transaction(ixs, vec![])?;

        let signer_seeds: &[&[&[u8]]] = &[&[
            QUEUE_AUTHORITY_SEED,
            &[self.global_state.queue_authority_bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.tuktuk_program.to_account_info(),
            QueueTaskV0 {
                payer: self.queue_authority.to_account_info(),
                queue_authority: self.queue_authority.to_account_info(),
                task_queue: self.task_queue.to_account_info(),
                task_queue_authority: self.task_queue_authority.to_account_info(),
                task: self.task.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
            signer_seeds,
        );

        queue_task_v0(
            ctx,
            QueueTaskArgsV0 {
                id: task_id,
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: None,
                free_tasks: 15, // this is for recursion, this task will queue one more task
                description: "payment for subscription".to_string(),
            },
        )
    }
}
