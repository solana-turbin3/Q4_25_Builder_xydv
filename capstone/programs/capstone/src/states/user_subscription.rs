use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorDeserialize, AnchorSerialize, Clone)]
pub enum Status {
    Active,
    Failed,
    Canceled, // todo: future
}

#[derive(InitSpace)]
#[account]
pub struct UserSubscription {
    pub subscriber: Pubkey,
    pub subscriber_ata: Pubkey,
    pub subscription: Pubkey,
    pub status: Status,
    pub failure_count: u8,
    pub cron_job: Pubkey,
    pub last_exec_ts: i64,
    pub next_cron_transaction_id: u32,
    pub queue_authority_bump: u8,
    pub bump: u8,
}

pub const SUBSCRIPTION_SEED: &[u8] = b"subscription";
