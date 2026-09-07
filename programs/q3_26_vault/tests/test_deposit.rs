mod common;

use common::*;

#[test]
fn deposit_succeeds() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");

    let vault_before = setup.vault_balance();
    let user_before = setup.user_balance();

    let transaction = setup.deposit(DEPOSIT).expect("deposit should succeed");

    assert_eq!(setup.vault_balance(), vault_before + DEPOSIT);
    assert!(setup.user_balance() <= user_before - DEPOSIT);

    log_success("Deposit", &transaction);
}

#[test]
fn deposit_accumulates() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");

    let vault_before = setup.vault_balance();
    setup.deposit(DEPOSIT).expect("first deposit should succeed");
    setup.deposit(DEPOSIT).expect("second deposit should succeed");

    assert_eq!(setup.vault_balance(), vault_before + 2 * DEPOSIT);
}

#[test]
fn deposit_zero_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");

    let vault_before = setup.vault_balance();

    assert_vault_error(setup.deposit(0), VaultError::InvalidAmount);

    assert_eq!(setup.vault_balance(), vault_before, "vault must be untouched");
}

#[test]
fn deposit_without_initialize_fails() {
    let mut setup = Setup::new();

    assert_anchor_error(
        setup.deposit(DEPOSIT),
        AnchorErrorCode::AccountNotInitialized,
    );
}
