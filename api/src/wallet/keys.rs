use std::collections::HashMap;

use crate::wallet::{Address, WalletError, WalletResult};

/// Simplified key pair for debugging
#[derive(Debug, Clone)]
pub struct NockchainKeyPair {
    name: String,
    address: Address,
}

impl NockchainKeyPair {
    pub fn new(name: String) -> Self {
        // Create a dummy address for debugging
        let dummy_pubkey = [0u8; 32];
        Self {
            name,
            address: Address::from_public_key(dummy_pubkey),
        }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Simplified key manager for debugging
#[derive(Debug, Clone)]
pub struct NockchainKeyManager {
    keys: HashMap<String, NockchainKeyPair>,
    default_key: Option<String>,
}

impl Default for NockchainKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NockchainKeyManager {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            default_key: None,
        }
    }

    pub fn generate_key(&mut self, name: String) -> WalletResult<&NockchainKeyPair> {
        if self.keys.contains_key(&name) {
            return Err(WalletError::KeyExists(name));
        }

        let keypair = NockchainKeyPair::new(name.clone());
        self.keys.insert(name.clone(), keypair);

        if self.default_key.is_none() {
            self.default_key = Some(name.clone());
        }

        Ok(self.keys.get(&name).unwrap())
    }

    pub fn get_key(&self, name: &str) -> Option<&NockchainKeyPair> {
        self.keys.get(name)
    }

    pub fn get_default_key(&self) -> Option<&NockchainKeyPair> {
        self.default_key
            .as_ref()
            .and_then(|name| self.keys.get(name))
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.keys.keys().cloned().collect()
    }

    pub fn get_all_addresses(&self) -> HashMap<String, Address> {
        self.keys
            .iter()
            .map(|(name, keypair)| (name.clone(), keypair.address().clone()))
            .collect()
    }

    /// Dummy implementation for compatibility
    pub fn create_transaction_hash(
        &self,
        _inputs: &[TransactionInput],
        _outputs: &[TransactionOutput],
        _fee: u64,
    ) -> Vec<u8> {
        vec![0u8; 32]
    }

    /// Dummy implementation for compatibility  
    pub fn sign_with_key(&self, _key_name: &str, _data: &[u8]) -> Result<Vec<u8>, WalletError> {
        Ok(vec![0u8; 64])
    }
}

/// Dummy transaction for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NockchainTransaction {
    pub id: String,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub hash: Vec<u8>,
}

impl NockchainTransaction {
    pub fn new(id: String) -> Self {
        Self {
            id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            hash: Vec::new(),
        }
    }
}

/// Dummy transaction input
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionInput {
    pub amount: u64,
}

/// Dummy transaction output  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionOutput {
    pub amount: u64,
    pub recipient_address: String,
}

// Type aliases for compatibility
pub type KeyManager = NockchainKeyManager;
pub type KeyPair = NockchainKeyPair;
