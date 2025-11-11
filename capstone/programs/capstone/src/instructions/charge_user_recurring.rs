use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use tuktuk_program::{
    compile_transaction, RunTaskReturnV0, TaskReturnV0, TransactionSourceV0, TriggerV0,
};

use crate::{
    error::SubscriptionError,
    states::{SubscriptionPlan, UserSubscription, SUBSCRIBER_VAULT_SEED, SUBSCRIPTION_SEED},
};

#[derive(Accounts)]
pub struct ChargeUserRecurring<'info> {
    /// CHECK: called via tuktuk
    pub subscriber: UncheckedAccount<'info>,
    /// CHECK: called via tuktuk
    pub merchant: UncheckedAccount<'info>,
    #[account(
        seeds = [SUBSCRIPTION_SEED, subscriber.key.as_ref(), subscription_plan.key().as_ref()],
        bump = user_subscription.bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    #[account(
        constraint = subscription_plan.active @ SubscriptionError::InactivePlan
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    #[account(
        seeds = [SUBSCRIBER_VAULT_SEED, subscriber.key.as_ref()],
        token::mint = mint,
        token::authority = subscriber_vault,
        token::token_program = token_program,
        bump = user_subscription.subscriber_vault_bump
    )]
    pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        associated_token::mint = mint,
        associated_token::authority = merchant,
        associated_token::token_program = token_program
    )]
    pub merchant_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        address = subscription_plan.mint @ SubscriptionError::MintMismatch,
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    // programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChargeUserRecurring<'info> {
    pub fn charge_user_recurring(&mut self) -> Result<RunTaskReturnV0> {
        self.transfer_tokens()?;

        let instructions = vec![Instruction {
            program_id: crate::ID,
            accounts: crate::accounts::ChargeUserRecurring {
                subscriber: self.subscriber.key(),
                merchant: self.merchant.key(),
                user_subscription: self.user_subscription.key(),
                subscription_plan: self.subscription_plan.key(),
                subscriber_vault: self.subscriber_vault.key(),
                merchant_ata: self.merchant_ata.key(),
                mint: self.mint.key(),
                associated_token_program: self.associated_token_program.key(),
                token_program: self.token_program.key(),
                system_program: self.system_program.key(),
            }
            .to_account_metas(None),
            data: crate::instruction::ChargeUserRecurring.data(),
        }];

        let (compiled_tx, _) = compile_transaction(instructions, vec![])?; // signer seeds?

        Ok(RunTaskReturnV0 {
            tasks: vec![TaskReturnV0 {
                trigger: TriggerV0::Now,
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: None,
                free_tasks: 1,
                description: format!(""),
            }],
            accounts: vec![],
        })
    }

    pub fn transfer_tokens(&mut self) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            SUBSCRIBER_VAULT_SEED,
            self.subscriber.key.as_ref(),
            &[self.user_subscription.subscriber_vault_bump],
        ]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.subscriber_vault.to_account_info(),
                to: self.merchant_ata.to_account_info(),
                mint: self.mint.to_account_info(),
                authority: self.subscriber_vault.to_account_info(),
            },
            signer_seeds,
        );

        transfer_checked(ctx, self.subscription_plan.amount, self.mint.decimals)
    }
}
