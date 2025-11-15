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
    pub interval: i64,
    pub bump: u8,
}

pub const PLAN_SEED: &[u8] = b"plan";
pub const VAULT_SEED: &[u8] = b"fees_vault";
pub const FEES: u64 = 10_000_000; // 0.01 SOL per subscription, for now no automation charges (maybe add in future)
pub const USDC_PUBKEY: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
