use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::spl_token_2022::instruction::AuthorityType,
    token_interface::{set_authority, Mint, SetAuthority, TokenAccount, TokenInterface},
};
use tuktuk_program::{
    cron::{
        cpi::{accounts::InitializeCronJobV0, initialize_cron_job_v0},
        program::Cron,
        types::InitializeCronJobArgsV0,
    },
    tuktuk::program::Tuktuk,
    TaskQueueAuthorityV0, TaskQueueV0,
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
    /// CHECK: Used in CPI
    pub user_cron_jobs: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: Used in CPI
    pub cron_job: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: Used in CPI
    pub cron_job_name_mapping: AccountInfo<'info>,
    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: AccountInfo<'info>,
    /// CHECK: Used to write return data
    #[account(mut)]
    pub task_return_account_1: AccountInfo<'info>,
    /// CHECK: Used to write return data
    #[account(mut)]
    pub task_return_account_2: AccountInfo<'info>,

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

        self.set_authority()?;

        self.user_subscription.set_inner(UserSubscription {
            subscriber: self.subscriber.key(),
            subscriber_ata: self.subscriber_mint_ata.key(),
            subscription: self.subscription_plan.key(),
            status: Status::Active,
            failure_count: 0,
            cron_job: self.cron_job.key(),
            next_cron_transaction_id: 0,
            queue_authority_bump: bumps.queue_authority,
            last_exec_ts: Clock::get()?.unix_timestamp,
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

    pub fn set_authority(&mut self) -> Result<()> {
        // it might be a case that we are already a authority
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            SetAuthority {
                current_authority: self.subscriber.to_account_info(),
                account_or_mint: self.subscriber_mint_ata.to_account_info(),
            },
        );

        // Add authority
        set_authority(ctx, AuthorityType::AccountOwner, None)
    }

    pub fn initialize_cron(&mut self, bumps: &SubscribeBumps) -> Result<()> {
        initialize_cron_job_v0(
            CpiContext::new_with_signer(
                self.cron_program.to_account_info(),
                InitializeCronJobV0 {
                    payer: self.subscriber.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    authority: self.queue_authority.to_account_info(),
                    user_cron_jobs: self.user_cron_jobs.to_account_info(),
                    cron_job: self.cron_job.to_account_info(),
                    cron_job_name_mapping: self.cron_job_name_mapping.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task: self.task.to_account_info(),
                    task_return_account_1: self.task_return_account_1.to_account_info(),
                    task_return_account_2: self.task_return_account_2.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    tuktuk_program: self.tuktuk_program.to_account_info(),
                },
                &[&[b"queue_authority", &[bumps.queue_authority]]],
            ),
            InitializeCronJobArgsV0 {
                name: format!("autopay service for {}", self.subscription_plan.name),
                schedule: self.subscription_plan.schedule.clone(),
                free_tasks_per_transaction: 0,
                num_tasks_per_queue_call: 5,
            },
        )
    }
}
