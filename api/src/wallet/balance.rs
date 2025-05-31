use crate::wallet::{Address, Balance, Note, WalletError, WalletResult};
use std::collections::HashMap;
use uuid::Uuid;

/// Balance manager for tracking UTXOs and balances
#[derive(Debug)]
pub struct BalanceManager {
    notes: HashMap<Uuid, Note>,
    address_balances: HashMap<Address, Balance>,
}

impl BalanceManager {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            address_balances: HashMap::new(),
        }
    }

    /// Add a new note (UTXO) to the wallet
    pub fn add_note(&mut self, note: Note) -> WalletResult<()> {
        let address = note.address.clone();
        let amount = note.amount;
        let block_height = note.block_height;

        // Add note to collection
        self.notes.insert(note.id, note);

        // Update balance for this address
        let balance = self
            .address_balances
            .entry(address)
            .or_insert_with(Balance::new);

        if block_height.is_some() {
            balance.confirmed += amount;
        } else {
            balance.unconfirmed += amount;
        }

        Ok(())
    }

    /// Mark a note as spent
    pub fn spend_note(&mut self, note_id: Uuid) -> WalletResult<()> {
        if let Some(note) = self.notes.get_mut(&note_id) {
            if note.spent {
                return Err(WalletError::Transaction("Note already spent".to_string()));
            }

            note.spent = true;

            // Update balance
            let balance = self
                .address_balances
                .get_mut(&note.address)
                .ok_or_else(|| WalletError::Storage("Address balance not found".to_string()))?;

            if note.block_height.is_some() {
                balance.confirmed = balance.confirmed.saturating_sub(note.amount);
            } else {
                balance.unconfirmed = balance.unconfirmed.saturating_sub(note.amount);
            }

            Ok(())
        } else {
            Err(WalletError::KeyNotFound(format!(
                "Note {} not found",
                note_id
            )))
        }
    }

    /// Get balance for a specific address
    pub fn get_balance(&self, address: &Address) -> Balance {
        self.address_balances
            .get(address)
            .cloned()
            .unwrap_or_else(Balance::new)
    }

    /// Get total balance across all addresses
    pub fn get_total_balance(&self) -> Balance {
        let mut total = Balance::new();

        for balance in self.address_balances.values() {
            total.confirmed += balance.confirmed;
            total.unconfirmed += balance.unconfirmed;
            total.locked += balance.locked;
        }

        total
    }

    /// Get available notes for spending
    pub fn get_spendable_notes(&self, address: &Address, amount: u64) -> Vec<&Note> {
        self.notes
            .values()
            .filter(|note| {
                note.address == *address
                    && !note.spent
                    && !note.locked
                    && note.block_height.is_some() // Only confirmed notes
            })
            .collect()
    }

    /// Get all notes for an address
    pub fn get_notes_for_address(&self, address: &Address) -> Vec<&Note> {
        self.notes
            .values()
            .filter(|note| note.address == *address)
            .collect()
    }
}
