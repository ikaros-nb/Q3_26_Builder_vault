#![allow(dead_code, unused_imports)]

use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas,
    prelude::{Pubkey, Rent},
    solana_program::{instruction::{Instruction, error::InstructionError}},
    system_program::ID as SYSTEM_PROGRAM_ID,
};
use litesvm::{
    types::{FailedTransactionMetadata, TransactionMetadata},
    LiteSVM,
};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;

// Re-exported so the test files need only `use common::*`.
pub use anchor_lang::error::ErrorCode as AnchorErrorCode;
pub use anchor_lang::prelude::msg;
pub use q3_26_vault::error::VaultError;
pub use solana_keypair::Signer;

/// Convenience alias for what every instruction helper returns.
pub type TxResult = Result<TransactionMetadata, FailedTransactionMetadata>;

pub const AIRDROP: u64 = 10_000_000_000;
pub const DEPOSIT: u64 = 1_000_000;
pub const WITHDRAW: u64 = 1_000_000;

pub struct Setup {
    pub svm: LiteSVM,
    pub user: Keypair,
    pub vault_state: Pubkey,
    pub vault: Pubkey,
}

impl Setup {
    pub fn new() -> Self {
        let program_id = q3_26_vault::id();

        let mut svm = LiteSVM::new();
        let bytes = include_bytes!(concat!(
            env!("CARGO_TARGET_TMPDIR"),
            "/../deploy/q3_26_vault.so"
        ));
        svm.add_program(program_id, bytes)
            .expect("load program: run `anchor build` first");

        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), AIRDROP)
            .expect("airdrop");

        let (vault_state, _) = Pubkey::find_program_address(
            &[
                q3_26_vault::constants::STATE_SEED,
                user.pubkey().as_ref(),
            ],
            &program_id,
        );

        let (vault, _) = Pubkey::find_program_address(
            &[
                q3_26_vault::constants::VAULT_SEED,
                user.pubkey().as_ref(),
            ],
            &program_id,
        );

        Self {
            svm,
            user,
            vault_state,
            vault,
        }
    }

    // ----- observers -----

    pub fn state(&self) -> q3_26_vault::state::VaultState {
        let acc = self.svm.get_account(&self.vault_state).unwrap();
        let mut data: &[u8] = &acc.data.as_ref();
        q3_26_vault::state::VaultState::try_deserialize(&mut data).unwrap()
    }

    /// Rent-exempt minimum for a zero-data account.
    pub fn rent_exempt(&self) -> u64 {
        Rent::default().minimum_balance(0)
    }

    /// Lamports held by `key`, or 0 when the account does not exist.
    pub fn balance(&self, key: &Pubkey) -> u64 {
        self.svm.get_account(key).map_or(0, |acc| acc.lamports)
    }

    pub fn vault_balance(&self) -> u64 {
        self.balance(&self.vault)
    }

    pub fn user_balance(&self) -> u64 {
        self.balance(&self.user.pubkey())
    }

    /// A second funded wallet, for the tests that check nobody else can touch this vault.
    pub fn other_user(&mut self) -> Keypair {
        let other = Keypair::new();
        self.svm.airdrop(&other.pubkey(), AIRDROP).expect("airdrop");
        other
    }

    // ----- instructions -----

    pub fn initialize(&mut self) -> TxResult {
        let ix = Instruction {
            program_id: q3_26_vault::id(),
            accounts: q3_26_vault::accounts::Initialize {
                user: self.user.pubkey(),
                vault_state: self.vault_state,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: q3_26_vault::instruction::Initialize {}.data(),
        };
        self.send(ix, &self.user.insecure_clone())
    }

    pub fn deposit(&mut self, amount: u64) -> TxResult {
        let ix = Instruction {
            program_id: q3_26_vault::id(),
            accounts: q3_26_vault::accounts::Deposit {
                user: self.user.pubkey(),
                vault_state: self.vault_state,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: q3_26_vault::instruction::Deposit {
                amount,
            }.data(),
        };
        self.send(ix, &self.user.insecure_clone())
    }

    pub fn withdraw(&mut self, amount: u64) -> TxResult {
        self.withdraw_as(&self.user.insecure_clone(), amount)
    }

    pub fn withdraw_as(&mut self, signer: &Keypair, amount: u64) -> TxResult {
        let ix = Instruction {
            program_id: q3_26_vault::id(),
            accounts: q3_26_vault::accounts::Withdraw {
                user: signer.pubkey(),
                vault_state: self.vault_state,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: q3_26_vault::instruction::Withdraw {
                amount,
            }.data(),
        };
        self.send(ix, signer)
    }

    pub fn close(&mut self) -> TxResult {
        let ix = Instruction {
            program_id: q3_26_vault::id(),
            accounts: q3_26_vault::accounts::Close {
                user: self.user.pubkey(),
                vault_state: self.vault_state,
                vault: self.vault,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: q3_26_vault::instruction::Close {}.data(),
        };
        self.send(ix, &self.user.insecure_clone())
    }

    fn send(&mut self, ix: Instruction, signer: &Keypair) -> TxResult {
        let message = Message::new(&[ix], Some(&signer.pubkey()));
        // A fresh blockhash per transaction, so two identical instructions in a
        // row do not collide as the same signature (`AlreadyProcessed`).
        self.svm.expire_blockhash();
        let recent_blockhash = self.svm.latest_blockhash();
        let transaction = Transaction::new(&[signer], message, recent_blockhash);
        self.svm.send_transaction(transaction)
    }
}

// ----- assertions -----

/// Errors raised by `require!` in the instruction handlers, at 6000 and above.
pub fn assert_vault_error(result: TxResult, expected: VaultError) {
    assert_custom_error(result, u32::from(expected));
}

/// Same, for errors Anchor raises from `#[account(...)]` attributes. They live below 6000,
/// in the ranges reserved by `anchor_lang::error::ErrorCode`.
pub fn assert_anchor_error(result: TxResult, expected: AnchorErrorCode) {
    assert_custom_error(result, u32::from(expected));
}

pub fn assert_custom_error(result: TxResult, expected_code: u32) {
    let failure = result.expect_err("transaction should have failed");
    assert_eq!(
        failure.err,
        TransactionError::InstructionError(0, InstructionError::Custom(expected_code)),
        "logs: {:#?}",
        failure.meta.logs,
    );
}

pub fn expect_failure(result: TxResult) -> FailedTransactionMetadata {
    match result {
        Ok(meta) => panic!(
            "expected the transaction to fail, but it succeeded: {}",
            meta.signature
        ),
        Err(failure) => failure,
    }
}

pub fn log_success(label: &str, transaction: &TransactionMetadata) {
    msg!("\n\n{} transaction successful", label);
    msg!("CUs Consumed: {}", transaction.compute_units_consumed);
    msg!("Tx Signature: {}", transaction.signature);
}
