use anchor_lang::prelude::*;
use tuktuk_program::tuktuk::program::Tuktuk;

use crate::{
    error::SubscriptionError,
    program::Capstone,
    states::{GlobalState, FEES, GLOBAL_STATE_SEED, QUEUE_AUTHORITY_SEED},
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [GLOBAL_STATE_SEED],
        space = GlobalState::DISCRIMINATOR.len() + GlobalState::INIT_SPACE,
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        seeds = [QUEUE_AUTHORITY_SEED],
        bump
    )]
    /// CHECK: via seeds
    pub queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: via signer, only can call this instruction
    pub task_queue: UncheckedAccount<'info>,

    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
    #[account(constraint = program_data.upgrade_authority_address == Some(signer.key()) @ SubscriptionError::InvalidSigner)]
    pub program_data: Account<'info, ProgramData>,
    #[account(constraint = this_program.programdata_address()? == Some(program_data.key()))]
    pub this_program: Program<'info, Capstone>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.global_state.set_inner(GlobalState {
            task_queue: self.task_queue.key(),
            queue_authority: self.queue_authority.key(),
            fees: FEES,
            queue_authority_bump: bumps.queue_authority,
            bump: bumps.global_state,
        });

        Ok(())
    }
}
