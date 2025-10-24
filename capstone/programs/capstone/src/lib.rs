use anchor_lang::prelude::*;

declare_id!("KCfcxRdDpnqL1JQiLTSwokRjFudgRZYs8uHXSGy8tu5");

#[program]
pub mod capstone {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
