use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct SubscriptionPlan {
    pub merchant: Pubkey,
    pub mint: Pubkey,
    pub merchant_ata: Pubkey,
    #[max_len(50)]
    pub name: String,
    pub amount: u64,
    #[max_len(10)] // todo: get exact max_size needed for schedule
    pub schedule: String,
    pub max_failure_count: u8,
}
