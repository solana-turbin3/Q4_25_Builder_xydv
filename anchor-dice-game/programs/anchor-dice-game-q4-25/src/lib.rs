pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("DZDRzKdTu4SweFFjDutMgPqu55Qt9TLbhWG1cMAikYVp");

#[program]
pub mod anchor_dice_game_q4_25 {
    use super::*;
    use anchor_lang::prelude::*;

    mod state;
    mod instructions;
    mod error;
    
    use instructions::*;
    use error::*;
    
    declare_id!("J86V1Echaw6CB1aMbGVbmgCb37RUBcev9QmruuR91mma");
    
    #[program]
    pub mod anchor_dice_2024 {
        use super::*;
    
        pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
            ctx.accounts.init(amount)
        }
    
        pub fn place_bet(ctx: Context<PlaceBet>, seed: u128, roll: u8, amount: u64) -> Result<()> {
            ctx.accounts.create_bet(seed, roll, amount, &ctx.bumps)?;
            ctx.accounts.deposit(amount)
        }
    
        // pub fn resolve_bet(ctx: Context<ResolveBet>, sig: Vec<u8>) -> Result<()> {
        //     ctx.accounts.verify_ed25519_signature(&sig)?;
        //     ctx.accounts.resolve_bet(&sig, &ctx.bumps)
        }
    
        pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
            ctx.accounts.refund_bet(&ctx.bumps)
        }
}
