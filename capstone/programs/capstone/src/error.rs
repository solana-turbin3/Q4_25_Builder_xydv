use anchor_lang::prelude::*;

#[error_code]
pub enum SubscriptionError {
    #[msg("amount must be positive")]
    InvalidAmount,
    #[msg("name must be provided")]
    InvalidName,
    #[msg("mint account mismatch")]
    MintMismatch,
}
