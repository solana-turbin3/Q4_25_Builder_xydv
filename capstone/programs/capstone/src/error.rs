use anchor_lang::prelude::*;

// todo: add proper messages
#[error_code]
pub enum SubscriptionError {
    #[msg("amount must be positive")]
    InvalidAmount,
    #[msg("name must be provided")]
    InvalidName,
    #[msg("mint account mismatch")]
    MintMismatch,
    #[msg("inactive plan")]
    InactivePlan,
    #[msg("invalid schedule")]
    InvalidSchedule,
    #[msg("max retries reached")]
    MaxRetriesReached,
    #[msg("arithmetic error")]
    ArithmeticError,
}
