use anchor_lang::prelude::*;

#[derive(InitSpace, AnchorDeserialize, AnchorSerialize, Clone)]
pub enum Status {
    Active,
    Failed,
    Canceled,
}

#[derive(InitSpace)]
#[account]
pub struct UserSubscription {
    pub subscriber: Pubkey,
    pub subscriber_ata: Pubkey,
    pub subscription: Pubkey,
    pub status: Status,
    pub failure_count: u8,
}
