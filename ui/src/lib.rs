//! This crate contains all shared UI for the workspace.

pub mod echo;
pub mod hero;
pub mod navbar;
pub mod wallet;

// Re-export commonly used components
pub use echo::Echo;
pub use hero::Hero;
pub use navbar::Navbar;

// Re-export wallet components
pub use wallet::{BalanceCard, NodeConsole, QuickActions, ReceiveView, SendForm, TransactionList};
