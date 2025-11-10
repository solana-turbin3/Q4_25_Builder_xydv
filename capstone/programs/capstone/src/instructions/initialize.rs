use anchor_lang::{prelude::*, solana_program::hash::hash};
use tuktuk_program::{
    tuktuk::{
        cpi::{
            accounts::{InitializeTaskQueueV0, InitializeTuktukConfigV0},
            initialize_task_queue_v0, initialize_tuktuk_config_v0,
        },
        program::Tuktuk,
    },
    types::InitializeTaskQueueArgsV0,
    InitializeTuktukConfigArgsV0, TaskQueueNameMappingV0, TaskQueueV0, TuktukConfigV0,
};

use crate::states::{GlobalState, FEES, GLOBAL_STATE_SEED};

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
        seeds = [b"queue_authority"],
        bump
    )]
    /// CHECK: via seeds
    pub queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub task_queue: Account<'info, TaskQueueV0>,
    #[account(
        init,
        payer = signer,
        space = TaskQueueNameMappingV0::INIT_SPACE,
        seeds = [
            "task_queue_name_mapping".as_bytes(),
            tuktuk_config.key().as_ref(),
            &hash("a".as_str())
        ],
        bump
    )]
    pub task_queue_name_mapping: Box<Account<'info, TaskQueueNameMappingV0>>,
    #[account(
      init,
      payer = signer,
      space = TuktukConfigV0::INIT_SPACE + 60,
      seeds = [b"tuktuk_config"],
      bump,
    )]
    pub tuktuk_config: Account<'info, TuktukConfigV0>,
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.global_state.set_inner(GlobalState {
            fees: FEES,
            queue_authority_bump: bumps.queue_authority,
            bump: bumps.global_state,
        });
        Ok(())
    }

    pub fn initialize_queue(&mut self) -> Result<()> {
        initialize_tuktuk_config_v0(
            CpiContext::new(
                self.tuktuk_program.to_account_info(),
                InitializeTuktukConfigV0 {
                    payer: self.signer.to_account_info(),
                    approver: self.signer.to_account_info(),
                    authority: self.signer.to_account_info(),
                    tuktuk_config: self.tuktuk_config.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            InitializeTuktukConfigArgsV0 {
                min_deposit: 1_000_000_000,
            },
        )?;

        initialize_task_queue_v0(
            CpiContext::new(
                self.tuktuk_program.to_account_info(),
                InitializeTaskQueueV0 {
                    payer: self.signer.to_account_info(),
                    tuktuk_config: self.tuktuk_config.to_account_info(),
                    update_authority: self.signer.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_name_mapping: self.task_queue_name_mapping.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
            ),
            InitializeTaskQueueArgsV0 {
                min_crank_reward: todo!(),
                name: todo!(),
                capacity: todo!(),
                lookup_tables: todo!(),
                stale_task_age: todo!(),
            },
        )
    }
}
