use anchor_lang::prelude::*;

declare_id!("SubsfvX3BEXk4JpzwFXUAL5H51rZtyooPSTprzjGeTz");

mod error;
mod events;
mod instructions;
mod states;

use instructions::*;

#[program]
pub mod capstone {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)
    }

    pub fn create_subscription(
        ctx: Context<CreateSubscription>,
        args: CreateSubscriptionArgs,
    ) -> Result<()> {
        ctx.accounts.create_subscription(args, &ctx.bumps)?;
        ctx.accounts.charge_fees()
    }

    pub fn subscribe(ctx: Context<Subscribe>) -> Result<()> {
        ctx.accounts.subscribe(&ctx.bumps)
    }

    pub fn charge_user_recurring(_ctx: Context<ChargeUserRecurring>) -> Result<()> {
        Ok(())
    }

    // pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
    //     ctx.accounts.revoke_delegate()
    // }
}
