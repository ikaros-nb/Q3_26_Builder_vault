use anchor_lang::prelude::*;

use crate::constants::VAULT_SEED;
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut, seeds = [VAULT_SEED], bump)]
    pub vault: Account<'info, VaultState>,
    pub authority: Signer<'info>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        Ok(())
    }
}