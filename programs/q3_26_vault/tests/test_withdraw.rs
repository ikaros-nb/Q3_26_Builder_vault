mod common;

use common::*;

#[test]
fn withdraw_succeeds() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let vault_before = setup.vault_balance();
    let user_before = setup.user_balance();

    let transaction = setup.withdraw(WITHDRAW).expect("withdraw should succeed");

    assert_eq!(setup.vault_balance(), vault_before - WITHDRAW);
    assert!(setup.user_balance() > user_before);

    log_success("Withdraw", &transaction);
}

#[test]
fn withdraw_down_to_rent_exempt_succeeds() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    setup.withdraw(DEPOSIT).expect("withdraw should succeed");

    assert_eq!(setup.vault_balance(), setup.rent_exempt());
}

#[test]
fn withdraw_zero_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let vault_before = setup.vault_balance();

    assert_vault_error(setup.withdraw(0), VaultError::InvalidAmount);

    assert_eq!(setup.vault_balance(), vault_before, "vault must be untouched");
}

#[test]
fn withdraw_breaking_rent_exemption_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let vault_before = setup.vault_balance();
    let withdrawable = vault_before - setup.rent_exempt();

    assert_vault_error(setup.withdraw(withdrawable + 1), VaultError::InsufficientVaultBalance);

    assert_eq!(setup.vault_balance(), vault_before, "vault must be untouched");
}

#[test]
fn withdraw_more_than_balance_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let vault_before = setup.vault_balance();

    assert_vault_error(setup.withdraw(vault_before + 1), VaultError::InsufficientVaultBalance);

    assert_eq!(setup.vault_balance(), vault_before, "vault must be untouched");
}

#[test]
fn withdraw_by_other_user_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let vault_before = setup.vault_balance();
    let other = setup.other_user();

    assert_anchor_error(
        setup.withdraw_as(&other, DEPOSIT),
        AnchorErrorCode::ConstraintSeeds,
    );

    assert_eq!(setup.vault_balance(), vault_before, "vault must be untouched");
}
