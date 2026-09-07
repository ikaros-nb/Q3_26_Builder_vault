mod common;

use common::*;

#[test]
fn test_deposit() {
    let mut setup = Setup::new();

    let transaction = setup.initialize()
        .expect("initialize should succeed");

    msg!("\n\nInitialize transaction sucessfull");
    msg!("CUs Consumed: {}", transaction.compute_units_consumed);
    msg!("Tx Signature: {}", transaction.signature);
}