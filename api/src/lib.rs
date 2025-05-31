//! This crate contains all shared fullstack server functions.

pub mod wallet;

/// Simple echo function (not a server function for now)
pub fn echo_string(input: String) -> String {
    input
}

// Re-export wallet types for easier access
pub use wallet::{
    Address, Balance, Note, Transaction, TransactionStatus, WalletConfig, WalletError, WalletResult,
};

pub use wallet::keys::{KeyManager, KeyPair, TransactionInput, TransactionOutput};

// Re-export node management types
pub use wallet::network::{LogEntry, LogLevel, LogSource, NodeConfig, NodeManager, NodeStatus};
