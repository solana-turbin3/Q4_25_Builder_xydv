use anchor_lang::prelude::*;

declare_id!("SubsfvX3BEXk4JpzwFXUAL5H51rZtyooPSTprzjGeTz");

mod error;
mod events;
mod instructions;
mod states;

use instructions::*;
use states::*;

#[program]
pub mod capstone {
    use super::*;

    pub fn create_subscription(
        ctx: Context<CreateSubscription>,
        args: CreateSubscriptionArgs,
    ) -> Result<()> {
        ctx.accounts.create_subscription(args, &ctx.bumps)?;
        ctx.accounts.charge_fees(FEES)
    }

    pub fn subscribe(ctx: Context<Subscribe>) -> Result<()> {
        ctx.accounts.subscribe(&ctx.bumps)
    }

    pub fn charge_user(_ctx: Context<ChargeUser>) -> Result<()> {
        Ok(())
    }
}
