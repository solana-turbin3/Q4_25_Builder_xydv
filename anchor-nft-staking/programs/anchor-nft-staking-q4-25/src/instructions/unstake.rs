use anchor_lang::prelude::*;
use mpl_core::{
    instructions::{RemovePluginV1CpiBuilder, UpdatePluginV1CpiBuilder},
    types::{FreezeDelegate, Plugin, PluginType},
    ID as CORE_PROGRAM_ID,
};

use crate::{
    errors::StakeError,
    state::{StakeAccount, StakeConfig, UserAccount},
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = !asset.data_is_empty() @ StakeError::AssetNotInitialized,
        constraint = asset.owner == &CORE_PROGRAM_ID @ StakeError::InvalidAsset
    )]
    /// CHECK: checked via cpi
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @ StakeError::InvalidCollection,
        constraint = !collection.data_is_empty() @ StakeError::CollectionNotInitialized
    )]
    /// CHECK: Verified by mpl-core
    pub collection: UncheckedAccount<'info>,

    #[account(
        seeds = [b"config".as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        mut,
        close = user,
        seeds = [b"stake", config.key().as_ref(), asset.key.as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == user.key() @ StakeError::NotOwner
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"user".as_ref(), user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: Verified by address constraint
    pub core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Unstake<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        let time_in_between =
            ((self.stake_account.staked_at - Clock::get()?.unix_timestamp) / (60 * 60 * 24)) as u32;

        require_gte!(
            time_in_between,
            self.config.freeze_period,
            StakeError::FreezePeriodNotPassed
        );

        let points = time_in_between * self.config.points_per_stake as u32;

        match self.user_account.points.checked_add(points) {
            Some(x) => self.user_account.points = x,
            None => return err!(StakeError::AssetAlreadyInitialized), // for now throw this error
        }

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"stake",
            &self.config.key().to_bytes(),
            &self.asset.key().to_bytes(),
            &[self.stake_account.bump],
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.stake_account.to_account_info())) // cause we set plugin init authority when staking
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;

        RemovePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::FreezeDelegate)
            .invoke()?;

        match self.user_account.amount_staked.checked_sub(1) {
            Some(x) => self.user_account.amount_staked = x,
            None => return err!(StakeError::AssetAlreadyInitialized), // for now throw this error
        }

        Ok(())
    }
}
