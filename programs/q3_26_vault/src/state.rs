use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub count: u64,
    pub authority: Pubkey,
}
