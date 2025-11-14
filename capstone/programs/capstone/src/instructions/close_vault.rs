use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::states::{SUBSCRIBER_VAULT_SEED, USDC_PUBKEY};

#[derive(Accounts)]
pub struct CloseVault<'info> {
    #[account(mut)]
    pub subscriber: Signer<'info>,

    #[account(
        address = USDC_PUBKEY,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = subscriber,
        associated_token::mint = mint,
        associated_token::authority = subscriber,
        associated_token::token_program = token_program
    )]
    pub subscriber_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [SUBSCRIBER_VAULT_SEED, subscriber.key.as_ref()],
        token::mint = mint,
        token::authority = subscriber_vault,
        token::token_program = token_program,
        bump
    )]
    pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,

    // PROGRAMS
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseVault<'info> {
    pub fn close_vault(&mut self, bumps: &CloseVaultBumps) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            SUBSCRIBER_VAULT_SEED,
            self.subscriber.key.as_ref(),
            &[bumps.subscriber_vault],
        ]];

        if self.subscriber_vault.amount.ne(&0) {
            transfer_checked(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TransferChecked {
                        from: self.subscriber_vault.to_account_info(),
                        to: self.subscriber.to_account_info(),
                        mint: self.mint.to_account_info(),
                        authority: self.subscriber_vault.to_account_info(),
                    },
                    signer_seeds,
                ),
                self.subscriber_vault.amount,
                self.mint.decimals,
            )?;
        }

        close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: self.subscriber_vault.to_account_info(),
                destination: self.subscriber.to_account_info(),
                authority: self.subscriber_vault.to_account_info(),
            },
            signer_seeds,
        ))
    }
}
