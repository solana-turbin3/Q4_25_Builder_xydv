use anchor_lang::prelude::*;

use crate::states::Status;

#[event]
pub struct SubscribeEvent {
    pub subscriber: Pubkey,
    pub subscription: Pubkey,
    pub status: Status,
}
