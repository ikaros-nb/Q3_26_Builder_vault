use anchor_lang::prelude::*;

use crate::constants::VAULT_SEED;
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [VAULT_SEED], bump)]
    pub vault: Account<'info, VaultState>,
    pub authority: Signer<'info>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self) -> Result<()> {
        Ok(())
    }
}
