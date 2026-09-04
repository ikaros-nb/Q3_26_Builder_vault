use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

use crate::constants::{STATE_SEED, VAULT_SEED};
use crate::error::VaultError;
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STATE_SEED, user.key().as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, VaultError::InvalidAmount);

        let vault_balance = self.vault.get_lamports();
        let rent_exempt = Rent::get()?.minimum_balance(0);
        require!(
            vault_balance.checked_sub(amount) >= Some(rent_exempt),
            VaultError::InsufficientVaultBalance
        );

        let cpi_program = self.system_program.key();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };
        let signer_seeds: [&[&[u8]]; 1] = [&[
            VAULT_SEED,
            self.user.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump]
        ]];
        let cpi_tx = CpiContext::new_with_signer(
            cpi_program,
            cpi_accounts,
            &signer_seeds,
        );

        transfer(cpi_tx, amount)?;

        Ok(())
    }
}
