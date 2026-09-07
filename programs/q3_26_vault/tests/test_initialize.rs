mod common;

use anchor_lang::prelude::Pubkey;
use common::*;
use q3_26_vault::constants::{STATE_SEED, VAULT_SEED};

#[test]
fn initialize_succeeds() {
    let mut setup = Setup::new();

    let transaction = setup.initialize()
        .expect("initialize should succeed");

    // The bumps stored in state must be the canonical ones for both PDAs.
    let state = setup.state();
    let user = setup.user.pubkey();
    let (_, state_bump) =
        Pubkey::find_program_address(&[STATE_SEED, user.as_ref()], &q3_26_vault::id());
    let (_, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, user.as_ref()], &q3_26_vault::id());
    assert_eq!(state.state_bump, state_bump);
    assert_eq!(state.vault_bump, vault_bump);

    // The vault is funded with exactly the rent-exempt minimum, nothing more.
    assert_eq!(setup.vault_balance(), setup.rent_exempt());

    log_success("Initialize", &transaction);
}

#[test]
fn initialize_twice_fails() {
    let mut setup = Setup::new();

    setup.initialize().expect("first initialize should succeed");

    // Not an Anchor code: `init` reaches the System program's CreateAccount,
    // which fails with SystemError::AccountAlreadyInUse (0).
    assert_custom_error(setup.initialize(), 0);
}
