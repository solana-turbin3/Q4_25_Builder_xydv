use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub task_queue: Pubkey,
    pub queue_authority: Pubkey, // PDA
    pub fees: u64,
    pub queue_authority_bump: u8,
    pub bump: u8,
}

pub const GLOBAL_STATE_SEED: &[u8] = b"global";
pub const QUEUE_AUTHORITY_SEED: &[u8] = b"queue_authority";
