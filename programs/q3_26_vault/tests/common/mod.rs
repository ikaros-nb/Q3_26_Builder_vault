use anchor_lang::{
    InstructionData, ToAccountMetas,
    prelude::Pubkey,
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
pub use anchor_lang::prelude::msg;
pub use q3_26_vault::error::VaultError;
pub use solana_keypair::Signer;

/// Convenience alias for what every instruction helper returns.
pub type TxResult = Result<TransactionMetadata, FailedTransactionMetadata>;

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
        svm.airdrop(&user.pubkey(), 10_000_000_000)
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

    fn send(&mut self, ix: Instruction, signer: &Keypair) -> TxResult {
        let message = Message::new(&[ix], Some(&signer.pubkey()));
        let recent_blockhash = self.svm.latest_blockhash();
        let transaction = Transaction::new(&[signer], message, recent_blockhash);
        self.svm.send_transaction(transaction)
    }
}