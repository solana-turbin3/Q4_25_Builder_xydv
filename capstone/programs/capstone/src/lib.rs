use anchor_lang::prelude::*;
use tuktuk_program::RunTaskReturnV0;

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

    pub fn charge_user_recurring(ctx: Context<ChargeUserRecurring>) -> Result<RunTaskReturnV0> {
        ctx.accounts.charge_user_recurring()
    }

    pub fn cancel_subscription(ctx: Context<CancelSubscription>) -> Result<()> {
        ctx.accounts.cancel_subscription()
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        ctx.accounts.close_vault(&ctx.bumps)
    }
}
