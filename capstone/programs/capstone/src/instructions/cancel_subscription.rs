use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{revoke, Mint, Revoke, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct CancelSubscription<'info> {
    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        // address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = subscriber,
        associated_token::token_program = token_program
    )]
    pub subscriber_mint_ata: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelSubscription<'info> {
    pub fn revoke_delegate(&mut self) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Revoke {
                source: self.subscriber_mint_ata.to_account_info(),
                authority: self.subscriber.to_account_info(),
            },
        );

        // revokes authority even if no delegate is set
        // https://github.com/solana-program/token/blob/a7c488ca39ed4cd71a87950ed854929816e9099f/program/src/processor.rs#L414
        revoke(ctx)
    }
}
