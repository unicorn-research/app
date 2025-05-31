use crate::wallet::keys::{KeyManager, TransactionInput, TransactionOutput};
use crate::wallet::{Address, Transaction, TransactionStatus, WalletError, WalletResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Transaction builder for creating new transactions
#[derive(Debug)]
pub struct TransactionBuilder {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    fee: u64,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            fee: 0,
        }
    }

    /// Add an input to the transaction
    pub fn add_input(&mut self, input: TransactionInput) {
        self.inputs.push(input);
    }

    /// Add an output to the transaction
    pub fn add_output(&mut self, output: TransactionOutput) {
        self.outputs.push(output);
    }

    /// Set the transaction fee
    pub fn set_fee(&mut self, fee: u64) {
        self.fee = fee;
    }

    /// Calculate total input amount
    pub fn total_input(&self) -> u64 {
        self.inputs.iter().map(|input| input.amount).sum()
    }

    /// Calculate total output amount
    pub fn total_output(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }

    /// Validate the transaction
    pub fn validate(&self) -> WalletResult<()> {
        if self.inputs.is_empty() {
            return Err(WalletError::Transaction("No inputs provided".to_string()));
        }

        if self.outputs.is_empty() {
            return Err(WalletError::Transaction("No outputs provided".to_string()));
        }

        let total_input = self.total_input();
        let total_output = self.total_output();
        let total_spent = total_output + self.fee;

        if total_input < total_spent {
            return Err(WalletError::InsufficientFunds {
                required: total_spent,
                available: total_input,
            });
        }

        Ok(())
    }

    /// Build and sign the transaction
    pub fn build_and_sign(
        &self,
        key_manager: &KeyManager,
        key_name: &str,
    ) -> WalletResult<SignedTransaction> {
        self.validate()?;

        // Create transaction hash
        let tx_hash = key_manager.create_transaction_hash(&self.inputs, &self.outputs, self.fee);

        // Sign the transaction
        let signature = key_manager.sign_with_key(key_name, &tx_hash)?;

        // Create transaction ID (in a real implementation, this would be more sophisticated)
        let tx_id = hex::encode(&tx_hash);

        let signed_tx = SignedTransaction {
            id: tx_id.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            fee: self.fee,
            signature: signature.to_vec(),
            hash: tx_hash,
        };

        Ok(signed_tx)
    }
}

/// A signed transaction ready for broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub id: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub fee: u64,
    pub signature: Vec<u8>,
    pub hash: Vec<u8>,
}

/// Transaction manager for handling transaction lifecycle
#[derive(Debug)]
pub struct TransactionManager {
    pending_transactions: Vec<Transaction>,
    confirmed_transactions: Vec<Transaction>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            pending_transactions: Vec::new(),
            confirmed_transactions: Vec::new(),
        }
    }

    /// Add a pending transaction
    pub fn add_pending_transaction(&mut self, signed_tx: SignedTransaction, is_outgoing: bool) {
        let transaction = Transaction {
            id: signed_tx.id,
            status: TransactionStatus::Pending,
            amount: signed_tx.outputs.iter().map(|o| o.amount).sum(),
            fee: signed_tx.fee,
            from_address: None, // TODO: Determine from inputs
            to_address: signed_tx
                .outputs
                .first()
                .map(|o| Address::from_string(&o.recipient_address).ok())
                .flatten(),
            created_at: Utc::now(),
            confirmed_at: None,
            is_outgoing,
        };

        self.pending_transactions.push(transaction);
    }

    /// Confirm a transaction
    pub fn confirm_transaction(&mut self, tx_id: &str, block_height: u64) -> WalletResult<()> {
        if let Some(pos) = self
            .pending_transactions
            .iter()
            .position(|tx| tx.id == tx_id)
        {
            let mut transaction = self.pending_transactions.remove(pos);
            transaction.status = TransactionStatus::Confirmed { block_height };
            transaction.confirmed_at = Some(Utc::now());

            self.confirmed_transactions.push(transaction);
            Ok(())
        } else {
            Err(WalletError::Transaction(format!(
                "Transaction {} not found",
                tx_id
            )))
        }
    }

    /// Get all transactions (pending + confirmed)
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        let mut all_transactions = Vec::new();
        all_transactions.extend(self.pending_transactions.clone());
        all_transactions.extend(self.confirmed_transactions.clone());

        // Sort by creation time (newest first)
        all_transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        all_transactions
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> &[Transaction] {
        &self.pending_transactions
    }

    /// Get confirmed transactions
    pub fn get_confirmed_transactions(&self) -> &[Transaction] {
        &self.confirmed_transactions
    }
}
