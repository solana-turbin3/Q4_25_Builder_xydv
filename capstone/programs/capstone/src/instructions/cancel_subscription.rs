use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::TokenInterface};
use tuktuk_program::{
    tuktuk::{
        cpi::{accounts::DequeueTaskV0, dequeue_task_v0},
        program::Tuktuk,
    },
    TaskQueueAuthorityV0,
};

use crate::{
    events::CancelSubscriptionEvent,
    states::{
        GlobalState, Status, UserSubscription, GLOBAL_STATE_SEED, QUEUE_AUTHORITY_SEED,
        SUBSCRIPTION_SEED,
    },
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
        seeds = [GLOBAL_STATE_SEED],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    // TUKTUK
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
    #[account(
        mut,
        // seeds = [b"task".as_ref(), task_queue.key().as_ref(), user_subscription.next_task_id.to_le_bytes().as_ref()],
        // seeds::program = tuktuk_program::tuktuk::ID,
        // constraint = !task.data_is_empty(),
        // bump, // ??
    )]
    /// CHECK: via seeds
    pub task: AccountInfo<'info>,

    // PROGRAMS
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelSubscription<'info> {
    pub fn cancel_subscription(&mut self) -> Result<()> {
        if self.user_subscription.status == Status::Active {
            self.dequeue_task()?;
        };

        self.user_subscription.status = Status::Canceled;

        emit!(CancelSubscriptionEvent {
            subscriber: self.subscriber.key(),
            subscription: self.user_subscription.subscription.key()
        });

        Ok(())
    }

    pub fn dequeue_task(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            QUEUE_AUTHORITY_SEED,
            &[self.global_state.queue_authority_bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.tuktuk_program.to_account_info(),
            DequeueTaskV0 {
                queue_authority: self.queue_authority.to_account_info(),
                rent_refund: self.task_queue.to_account_info(), // ??
                task_queue_authority: self.task_queue_authority.to_account_info(),
                task_queue: self.task_queue.to_account_info(),
                task: self.task.to_account_info(),
            },
            signer_seeds,
        );

        dequeue_task_v0(ctx)
    }
}
