pub mod balance;
pub mod keys;
pub mod network;
pub mod storage;
pub mod transaction;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Wallet errors with comprehensive error handling
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key already exists: {0}")]
    KeyExists(String),

    #[error("No default key set")]
    NoDefaultKey,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Block validation error: {0}")]
    BlockValidation(String),

    #[error("Consensus error: {0}")]
    Consensus(String),
}

pub type WalletResult<T> = Result<T, WalletError>;

/// Nockchain-style address derived from public key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Address {
    pub public_key: [u8; 32],
}

impl Address {
    pub fn from_public_key(public_key: [u8; 32]) -> Self {
        Self { public_key }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut public_key = [0u8; 32];
        let len = bytes.len().min(32);
        public_key[..len].copy_from_slice(&bytes[..len]);
        Self { public_key }
    }

    pub fn to_string(&self) -> String {
        bs58::encode(&self.public_key).into_string()
    }

    pub fn from_string(s: &str) -> WalletResult<Self> {
        let decoded = bs58::decode(s)
            .into_vec()
            .map_err(|e| WalletError::InvalidAddress(format!("Base58 decode error: {}", e)))?;

        if decoded.len() != 32 {
            return Err(WalletError::InvalidAddress(
                "Invalid address length".to_string(),
            ));
        }

        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&decoded);

        Ok(Self { public_key })
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Balance information for an address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Balance {
    pub confirmed: u64,
    pub unconfirmed: u64,
    pub locked: u64,
}

impl Balance {
    pub fn new() -> Self {
        Self {
            confirmed: 0,
            unconfirmed: 0,
            locked: 0,
        }
    }

    pub fn total(&self) -> u64 {
        self.confirmed + self.unconfirmed
    }

    pub fn available(&self) -> u64 {
        self.confirmed.saturating_sub(self.locked)
    }
}

/// UTXO note for nockchain wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub address: Address,
    pub amount: u64,
    pub block_height: Option<u64>,
    pub transaction_id: String,
    pub output_index: u32,
    pub spent: bool,
    pub locked: bool,
    pub created_at: DateTime<Utc>,
}

/// Transaction status in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed { block_height: u64 },
    Failed { reason: String },
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub status: TransactionStatus,
    pub amount: u64,
    pub fee: u64,
    pub from_address: Option<Address>,
    pub to_address: Option<Address>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub is_outgoing: bool,
}

/// Nockchain block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub height: u64,
}

impl BlockHeader {
    /// Calculate the hash of this block header
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.previous_hash);
        hasher.update(&self.merkle_root);
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.bits.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.height.to_le_bytes());

        hasher.finalize().into()
    }

    /// Check if this block meets the proof-of-work difficulty requirement
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.hash();
        let target = difficulty_to_target(self.bits);

        // Check if hash is less than target (big-endian comparison)
        for i in 0..32 {
            match hash[i].cmp(&target[i]) {
                std::cmp::Ordering::Less => return true,
                std::cmp::Ordering::Greater => return false,
                std::cmp::Ordering::Equal => continue,
            }
        }
        true
    }
}

/// Full nockchain block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<keys::NockchainTransaction>,
}

impl Block {
    /// Create a new block
    pub fn new(
        previous_hash: [u8; 32],
        transactions: Vec<keys::NockchainTransaction>,
        height: u64,
        bits: u32,
    ) -> Self {
        let merkle_root = calculate_merkle_root(&transactions);
        let timestamp = Utc::now().timestamp() as u64;

        let header = BlockHeader {
            version: 1,
            previous_hash,
            merkle_root,
            timestamp,
            bits,
            nonce: 0,
            height,
        };

        Self {
            header,
            transactions,
        }
    }

    /// Mine this block by finding a valid nonce
    pub fn mine(&mut self) -> WalletResult<()> {
        const MAX_NONCE: u64 = u64::MAX;

        for nonce in 0..MAX_NONCE {
            self.header.nonce = nonce;

            if self.header.meets_difficulty() {
                return Ok(());
            }

            // Update timestamp occasionally during mining
            if nonce % 100000 == 0 {
                self.header.timestamp = Utc::now().timestamp() as u64;
            }
        }

        Err(WalletError::Consensus(
            "Failed to find valid nonce".to_string(),
        ))
    }

    /// Validate this block
    pub fn validate(&self) -> WalletResult<()> {
        // Check proof of work
        if !self.header.meets_difficulty() {
            return Err(WalletError::BlockValidation(
                "Invalid proof of work".to_string(),
            ));
        }

        // Check merkle root
        let calculated_merkle = calculate_merkle_root(&self.transactions);
        if calculated_merkle != self.header.merkle_root {
            return Err(WalletError::BlockValidation(
                "Invalid merkle root".to_string(),
            ));
        }

        // Validate all transactions
        for tx in &self.transactions {
            // Basic transaction validation would go here
            if tx.inputs.is_empty() {
                return Err(WalletError::BlockValidation(
                    "Transaction has no inputs".to_string(),
                ));
            }
            if tx.outputs.is_empty() {
                return Err(WalletError::BlockValidation(
                    "Transaction has no outputs".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the block hash
    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }
}

/// Calculate merkle root of transactions
fn calculate_merkle_root(transactions: &[keys::NockchainTransaction]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    if transactions.is_empty() {
        return [0u8; 32];
    }

    let mut hashes: Vec<[u8; 32]> = transactions
        .iter()
        .map(|tx| {
            let mut hash = [0u8; 32];
            let len = std::cmp::min(32, tx.hash.len());
            hash[..len].copy_from_slice(&tx.hash[..len]);
            hash
        })
        .collect();

    while hashes.len() > 1 {
        let mut next_level = Vec::new();

        for chunk in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&chunk[0]);

            if chunk.len() == 2 {
                hasher.update(&chunk[1]);
            } else {
                // Odd number of hashes, duplicate the last one
                hasher.update(&chunk[0]);
            }

            next_level.push(hasher.finalize().into());
        }

        hashes = next_level;
    }

    hashes[0]
}

/// Convert difficulty bits to target hash
fn difficulty_to_target(bits: u32) -> [u8; 32] {
    let exponent = ((bits >> 24) & 0xff) as usize;
    let mantissa = bits & 0x00ffffff;

    let mut target = [0u8; 32];

    if exponent <= 3 {
        let target_value = mantissa >> (8 * (3 - exponent));
        target[29] = (target_value >> 16) as u8;
        target[30] = (target_value >> 8) as u8;
        target[31] = target_value as u8;
    } else if exponent < 32 {
        let start_byte = 32 - exponent;
        target[start_byte] = (mantissa >> 16) as u8;
        target[start_byte + 1] = (mantissa >> 8) as u8;
        target[start_byte + 2] = mantissa as u8;
    }

    target
}

/// Blockchain state and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub initial_difficulty: u32,
    pub target_block_time: u64,              // seconds
    pub difficulty_adjustment_interval: u64, // blocks
    pub max_block_size: usize,
    pub genesis_hash: [u8; 32],
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            initial_difficulty: 0x1d00ffff,       // Bitcoin-style difficulty
            target_block_time: 600,               // 10 minutes
            difficulty_adjustment_interval: 2016, // ~2 weeks
            max_block_size: 1_000_000,            // 1MB
            genesis_hash: [0u8; 32],
        }
    }
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub blockchain: BlockchainConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub node_addresses: Vec<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub p2p_port: u16,
    pub rpc_port: u16,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub require_pin: bool,
    pub pin_timeout_minutes: u64,
    pub enable_biometrics: bool,
    pub auto_lock_minutes: u64,
}

// Re-export important nockchain types for external use
pub use keys::{NockchainKeyManager, NockchainKeyPair, NockchainTransaction};
pub use network::{
    LogEntry, LogLevel, LogSource, NockchainNodeConfig, NockchainNodeManager, NockchainNodeRunner,
    NodeStatus,
};
pub use transaction::TransactionManager;
