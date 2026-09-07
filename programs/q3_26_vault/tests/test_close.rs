mod common;

use common::*;

#[test]
fn close_succeeds() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.deposit(DEPOSIT).expect("deposit should succeed");

    let user_before = setup.user_balance();
    let vault_before = setup.vault_balance();
    let state_rent = setup.balance(&setup.vault_state);

    let transaction = setup.close().expect("close should succeed");

    // Both accounts are emptied: the vault by the transfer, the state by
    // `close = user`.
    assert_eq!(setup.vault_balance(), 0);
    assert_eq!(setup.balance(&setup.vault_state), 0);

    // The user gets everything back, minus the transaction fee.
    assert!(setup.user_balance() > user_before);
    assert!(setup.user_balance() <= user_before + vault_before + state_rent);

    log_success("Close", &transaction);
}

#[test]
fn close_twice_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");

    setup.close().expect("first close should succeed");

    assert_anchor_error(setup.close(), AnchorErrorCode::AccountNotInitialized);
}

#[test]
fn deposit_after_close_fails() {
    let mut setup = Setup::new();
    setup.initialize().expect("initialize should succeed");
    setup.close().expect("close should succeed");

    assert_anchor_error(
        setup.deposit(DEPOSIT),
        AnchorErrorCode::AccountNotInitialized,
    );
}
