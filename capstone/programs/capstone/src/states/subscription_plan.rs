use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct SubscriptionPlan {
    pub merchant: Pubkey,
    pub mint: Pubkey,
    pub merchant_ata: Pubkey, // ?? derive onchain from find program address sync?
    pub amount: u64,
    pub active: bool,
    pub max_failure_count: u8,
    #[max_len(50)]
    pub name: String,
    #[max_len(10)] // todo: get exact max_size needed for schedule
    pub schedule: String,
    pub bump: u8,
}

pub const PLAN_SEED: &[u8] = b"plan";
pub const VAULT_SEED: &[u8] = b"vault";
pub const FEES: u64 = 10_000_000; // 0.01 SOL per subscription, for now no automation charges (maybe add in future)
