use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

use crate::constants::{STATE_SEED, VAULT_SEED};
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
        seeds = [STATE_SEED, user.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [VAULT_SEED, user.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(self.vault.data_len());
        let cpi_program = self.system_program.key();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_tx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_tx, rent_exempt)?;

        self.vault_state.set_inner(VaultState {
            vault_bump: bumps.vault,
            state_bump: bumps.vault_state,
        });

        Ok(())
    }
}
