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

    pub fn create_subscription(
        ctx: Context<CreateSubscription>,
        name: String,
        amount: u64,
        schedule: String,
        max_failure_count: u8,
    ) -> Result<()> {
        ctx.accounts
            .create_subscription(name, amount, schedule, max_failure_count, &ctx.bumps)
    }

    pub fn subscribe(ctx: Context<Subscribe>) -> Result<()> {
        ctx.accounts.subscribe(&ctx.bumps)
    }

    pub fn charge_user(_ctx: Context<ChargeUser>) -> Result<()> {
        Ok(())
    }
}
