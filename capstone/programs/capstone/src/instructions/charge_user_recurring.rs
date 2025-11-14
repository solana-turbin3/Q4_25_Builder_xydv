use anchor_lang::{
    prelude::*,
    solana_program::{
        instruction::Instruction,
        sysvar::instructions::{
            load_current_index_checked, load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
        },
    },
    InstructionData,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use tuktuk_program::{
    compile_transaction, RunTaskReturnV0, TaskReturnV0, TransactionSourceV0, TriggerV0,
};

use crate::{
    error::SubscriptionError,
    events::{ChargeEvent, SubscriptionFailedEvent},
    states::{
        Status, SubscriptionPlan, UserSubscription, SUBSCRIBER_VAULT_SEED, SUBSCRIPTION_SEED,
    },
};

#[derive(Accounts)]
pub struct ChargeUserRecurring<'info> {
    /// CHECK: called via tuktuk
    pub subscriber: UncheckedAccount<'info>,
    /// CHECK: called via tuktuk
    pub merchant: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [SUBSCRIPTION_SEED, subscriber.key.as_ref(), subscription_plan.key().as_ref()],
        bump = user_subscription.bump
    )]
    pub user_subscription: Account<'info, UserSubscription>,
    #[account(
        constraint = subscription_plan.active @ SubscriptionError::InactivePlan
    )]
    pub subscription_plan: Account<'info, SubscriptionPlan>,
    #[account(
        mut,
        token::mint = mint,
        token::authority = subscriber_vault,
        token::token_program = token_program,
        seeds = [SUBSCRIBER_VAULT_SEED, subscriber.key.as_ref()],
        bump = user_subscription.subscriber_vault_bump
    )]
    pub subscriber_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
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
    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    /// CHECK: InstructionsSysvar account
    pub instructions: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ChargeUserRecurring<'info> {
    pub fn charge_user_recurring(&mut self) -> Result<RunTaskReturnV0> {
        self.set_next_task_id()?; // is this right place?

        // improvements: check cpi failure
        if self.subscriber_vault.amount < self.subscription_plan.amount {
            msg!("not enough amount of tokens in vault");

            match self.user_subscription.failure_count.checked_add(1) {
                Some(x) => self.user_subscription.failure_count = x,
                None => return err!(SubscriptionError::ArithmeticError),
            };

            if self.user_subscription.failure_count > self.subscription_plan.max_failure_count {
                msg!("max failure count reached, changing status to failed");

                self.user_subscription.status = Status::Failed;

                emit!(SubscriptionFailedEvent {
                    subscriber: self.subscriber.key(),
                    subscription: self.subscription_plan.key()
                });

                // return no task so that recursion ends
                return Ok(RunTaskReturnV0 {
                    tasks: vec![],
                    accounts: vec![],
                });
            }

            // improvement: dont hard-code
            let one_day_later = self.user_subscription.last_exec_ts.checked_add(60).unwrap();

            self.schedule_next_task(one_day_later)
        } else {
            self.transfer_tokens()?;

            self.user_subscription.failure_count = 0;

            let next_exec_ts = self
                .user_subscription
                .last_exec_ts
                .checked_add(self.subscription_plan.interval)
                .unwrap();

            self.user_subscription.last_exec_ts = next_exec_ts;

            emit!(ChargeEvent {
                subscriber: self.subscriber.key(),
                subscription: self.subscription_plan.key(),
                amount: self.subscription_plan.amount
            });

            self.schedule_next_task(next_exec_ts)
        }
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

    // set next task id using introspection, next id is the first element from free tasks
    pub fn set_next_task_id(&mut self) -> Result<()> {
        let index = load_current_index_checked(&self.instructions.to_account_info())? as usize;
        let instruction = load_instruction_at_checked(index, &self.instructions.to_account_info())?;
        let next_task_id = u16::from_le_bytes(instruction.data[12..14].try_into().unwrap());

        self.user_subscription.next_task_id = next_task_id;
        Ok(())
    }

    pub fn schedule_next_task(&mut self, timestamp: i64) -> Result<RunTaskReturnV0> {
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
                instructions: self.instructions.key(),
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
                trigger: TriggerV0::Timestamp(timestamp),
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: None,
                free_tasks: 1,
                description: "payment".to_string(),
            }],
            accounts: vec![],
        })
    }
}
