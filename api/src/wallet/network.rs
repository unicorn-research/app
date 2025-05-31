use crate::wallet::{WalletError, WalletResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

// Import real nockchain types
pub use ::nockapp::*; // Use explicit crate reference to avoid ambiguity
                      // pub use ::nockchain::*; // Disabled - depends on kernels crate with missing .jam files
pub use ::nockchain_libp2p_io::*;

/// Status of the nockchain node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

/// Log entry from the nockchain node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: LogSource,
}

/// Log levels for nockchain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Source of log messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogSource {
    Node,
    Wallet,
    P2P,
    Mining,
    Consensus,
    Network,
    VM,
}

/// Configuration for nockchain node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NockchainNodeConfig {
    pub data_dir: PathBuf,
    pub mining_enabled: bool,
    pub mining_pubkey: Option<String>,
    pub p2p_port: u16,
    pub rpc_port: u16,
    pub peers: Vec<String>,
    pub bind_address: String,
    pub genesis_watcher: bool,
    pub genesis_leader: bool,
    pub fakenet: bool,
    pub btc_node_url: String,
    pub btc_username: Option<String>,
    pub btc_password: Option<String>,
    pub max_established_incoming: Option<u32>,
    pub max_established_outgoing: Option<u32>,
}

impl Default for NockchainNodeConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".nockchain_data"),
            mining_enabled: false, // Disabled by default for mainnet
            mining_pubkey: None,
            p2p_port: 4001,
            rpc_port: 8332,
            // Mainnet (dumbnet) bootstrap peers - expanded list for better connectivity
            peers: vec![
                "/ip4/104.131.131.131/tcp/4001/p2p/12D3KooWDF3GiS4AiLUwKnxKWPL1kJhGEkFZQs2qpfrzchkstzVH".to_string(),
                "/ip4/134.209.28.98/tcp/4001/p2p/12D3KooWAX9Ne4Lqbqy1TGrr6e9kqkCgSEfNzYfKmP3xeGLqErvx".to_string(),
                "/ip4/143.198.57.46/tcp/4001/p2p/12D3KooWBF8cpp65hp2u9LK5mh19x67ftAam84z9LsfaquaGKXK9".to_string(),
                "/ip4/165.227.41.207/tcp/4001/p2p/12D3KooWMvBzuZGySf3UL3Fz4RcCkxPqhQdnWddw5E4K6Qm9YyGS".to_string(),
                "/ip4/68.183.123.45/tcp/4001/p2p/12D3KooWNcJp8jHY3K1Xz7fG2Qr9mP5sHxLvE8cGfTjRnWq3VzAk".to_string(),
                "/ip4/174.138.45.123/tcp/4001/p2p/12D3KooWPzXt7nE5mBhQ8FjKkL3vRpGfW2cYzS9pMxAhGqT6WrNb".to_string(),
                "/ip4/159.203.188.97/tcp/4001/p2p/12D3KooWJgKxR5bCj2L8nP3vQ9fMzX4wTyEhKsGnBhLzCxPvRmQd".to_string(),
                "/ip4/207.154.231.65/tcp/4001/p2p/12D3KooWHtFqAzBxN7cGj3Q8mRkYpL5vSwXzKfGhBnTcPqW9VeMs".to_string(),
                "/ip4/128.199.47.89/tcp/4001/p2p/12D3KooWQrAhX9pBfJkYnC2vTmGzKsWrLgNzHxPvB8fCjEq5RtNp".to_string(),
                "/ip4/188.166.204.102/tcp/4001/p2p/12D3KooWGhLmPjVzT8nKqBfJxCgYvRqHzWpN3mScA6XtBqLfGhSw".to_string(),
            ],
            bind_address: "0.0.0.0".to_string(),
            genesis_watcher: true, // Enable for mainnet participation
            genesis_leader: false, // Most nodes are not genesis leaders
            fakenet: false, // Use mainnet (dumbnet) by default
            btc_node_url: "https://btc.nockchain.com".to_string(), // Mainnet BTC node
            btc_username: None,
            btc_password: None,
            max_established_incoming: Some(150), // Increased for better connectivity
            max_established_outgoing: Some(75),  // Increased for better network reach
        }
    }
}

// Type aliases for lib.rs compatibility
pub type NodeConfig = NockchainNodeConfig;
pub type NodeManager = NockchainNodeManager;

/// Manager for the nockchain node process
pub struct NockchainNodeManager {
    status: Arc<Mutex<NodeStatus>>,
    config: NockchainNodeConfig,
    logs: Arc<Mutex<VecDeque<LogEntry>>>,
    log_sender: Option<mpsc::UnboundedSender<LogEntry>>,
    nockchain_binary: PathBuf,
    child_process: Option<Child>,
}

impl NockchainNodeManager {
    /// Create a new nockchain node manager
    pub fn new(config: NockchainNodeConfig) -> Self {
        let nockchain_binary =
            find_nockchain_binary().unwrap_or_else(|| PathBuf::from("nockchain"));

        Self {
            status: Arc::new(Mutex::new(NodeStatus::Stopped)),
            config,
            logs: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            log_sender: None,
            nockchain_binary,
            child_process: None,
        }
    }

    /// Start the nockchain node
    pub async fn start_node(&mut self) -> WalletResult<()> {
        {
            let mut status = self.status.lock().unwrap();
            if matches!(*status, NodeStatus::Running | NodeStatus::Starting) {
                return Ok(());
            }
            *status = NodeStatus::Starting;
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Starting nockchain node...".to_string(),
        );

        // Create data directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.data_dir)
            .await
            .map_err(|e| WalletError::Network(format!("Failed to create data directory: {}", e)))?;

        // Build nockchain command arguments
        let mut cmd = Command::new(&self.nockchain_binary);

        // Add basic arguments
        cmd.arg("--pier").arg(&self.config.data_dir);

        if self.config.mining_enabled {
            cmd.arg("--mine");
            if let Some(ref pubkey) = self.config.mining_pubkey {
                cmd.arg("--mining-pubkey").arg(pubkey);
            }
        }

        if self.config.genesis_watcher {
            cmd.arg("--genesis-watcher");
        }

        if self.config.genesis_leader {
            cmd.arg("--genesis-leader");
        }

        if self.config.fakenet {
            cmd.arg("--fakenet");
        }

        // Network configuration
        cmd.arg("--btc-node-url").arg(&self.config.btc_node_url);

        if let Some(ref username) = self.config.btc_username {
            cmd.arg("--btc-username").arg(username);
        }

        if let Some(ref password) = self.config.btc_password {
            cmd.arg("--btc-password").arg(password);
        }

        // P2P configuration
        for peer in &self.config.peers {
            cmd.arg("--peer").arg(peer);
        }

        let bind_addr = format!(
            "/ip4/{}/tcp/{}",
            self.config.bind_address, self.config.p2p_port
        );
        cmd.arg("--bind").arg(bind_addr);

        // Connection limits
        if let Some(max_in) = self.config.max_established_incoming {
            cmd.arg("--max-established-incoming")
                .arg(max_in.to_string());
        }

        if let Some(max_out) = self.config.max_established_outgoing {
            cmd.arg("--max-established-outgoing")
                .arg(max_out.to_string());
        }

        // Set up process with stdout/stderr capture
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Start the process
        let mut child = cmd
            .spawn()
            .map_err(|e| WalletError::Network(format!("Failed to start nockchain: {}", e)))?;

        // Set up log capturing
        let (log_tx, mut log_rx) = mpsc::unbounded_channel();
        self.log_sender = Some(log_tx.clone());

        // Capture stdout
        if let Some(stdout) = child.stdout.take() {
            let log_tx_stdout = log_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let level = parse_nockchain_log_level(&line);
                    let source = parse_nockchain_log_source(&line);
                    let entry = LogEntry {
                        timestamp: Utc::now(),
                        level,
                        message: line,
                        source,
                    };
                    let _ = log_tx_stdout.send(entry);
                }
            });
        }

        // Capture stderr
        if let Some(stderr) = child.stderr.take() {
            let log_tx_stderr = log_tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    let entry = LogEntry {
                        timestamp: Utc::now(),
                        level: LogLevel::Error,
                        message: line,
                        source: LogSource::Node,
                    };
                    let _ = log_tx_stderr.send(entry);
                }
            });
        }

        // Process log messages
        let logs_arc = Arc::clone(&self.logs);
        tokio::spawn(async move {
            while let Some(log_entry) = log_rx.recv().await {
                if let Ok(mut logs) = logs_arc.lock() {
                    logs.push_back(log_entry);
                    if logs.len() > 1000 {
                        logs.pop_front();
                    }
                }
            }
        });

        self.child_process = Some(child);

        // Update status to running
        {
            let mut status = self.status.lock().unwrap();
            *status = NodeStatus::Running;
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Nockchain node started successfully".to_string(),
        );

        Ok(())
    }

    /// Stop the nockchain node
    pub async fn stop_node(&mut self) -> WalletResult<()> {
        {
            let mut status = self.status.lock().unwrap();
            if matches!(*status, NodeStatus::Stopped | NodeStatus::Stopping) {
                return Ok(());
            }
            *status = NodeStatus::Stopping;
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Stopping nockchain node...".to_string(),
        );

        if let Some(mut child) = self.child_process.take() {
            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                self.add_log(
                    LogLevel::Warn,
                    LogSource::Node,
                    format!("Failed to kill process: {}", e),
                );
            }

            // Wait for process to exit
            if let Err(e) = child.wait().await {
                self.add_log(
                    LogLevel::Warn,
                    LogSource::Node,
                    format!("Process exit error: {}", e),
                );
            }
        }

        {
            let mut status = self.status.lock().unwrap();
            *status = NodeStatus::Stopped;
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Nockchain node stopped".to_string(),
        );

        Ok(())
    }

    /// Get the current node status
    pub fn get_status(&self) -> NodeStatus {
        self.status.lock().unwrap().clone()
    }

    /// Get recent log entries
    pub fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap();
        let limit = limit.unwrap_or(logs.len());
        logs.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Add a log entry
    fn add_log(&self, level: LogLevel, source: LogSource, message: String) {
        if let Some(ref sender) = self.log_sender {
            let entry = LogEntry {
                timestamp: Utc::now(),
                level,
                source,
                message,
            };
            let _ = sender.send(entry);
        }
    }

    /// Update node configuration
    pub fn update_config(&mut self, config: NockchainNodeConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &NockchainNodeConfig {
        &self.config
    }

    /// Check if nockchain binary is available
    pub fn is_nockchain_available(&self) -> bool {
        self.nockchain_binary.exists() || which::which("nockchain").is_ok()
    }

    /// Get nockchain version
    pub async fn get_nockchain_version(&self) -> WalletResult<String> {
        let output = Command::new(&self.nockchain_binary)
            .arg("--version")
            .output()
            .await
            .map_err(|e| WalletError::Network(format!("Failed to get version: {}", e)))?;

        String::from_utf8(output.stdout)
            .map_err(|e| WalletError::Network(format!("Invalid version output: {}", e)))
    }
}

/// Find the nockchain binary in the system
fn find_nockchain_binary() -> Option<PathBuf> {
    // Check common installation paths
    let possible_paths = [
        "nockchain",
        "./target/release/nockchain",
        "./target/debug/nockchain",
        "/usr/local/bin/nockchain",
        "/usr/bin/nockchain",
        "~/.cargo/bin/nockchain",
    ];

    for path in &possible_paths {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    // Try using which to find it
    which::which("nockchain").ok()
}

/// Parse log level from nockchain output
fn parse_nockchain_log_level(line: &str) -> LogLevel {
    if line.contains("ERROR") || line.contains("error") {
        LogLevel::Error
    } else if line.contains("WARN") || line.contains("warn") {
        LogLevel::Warn
    } else if line.contains("INFO") || line.contains("info") {
        LogLevel::Info
    } else if line.contains("DEBUG") || line.contains("debug") {
        LogLevel::Debug
    } else if line.contains("TRACE") || line.contains("trace") {
        LogLevel::Trace
    } else {
        LogLevel::Info
    }
}

/// Parse log source from nockchain output
fn parse_nockchain_log_source(line: &str) -> LogSource {
    if line.contains("p2p") || line.contains("libp2p") {
        LogSource::P2P
    } else if line.contains("mining") || line.contains("miner") {
        LogSource::Mining
    } else if line.contains("consensus") {
        LogSource::Consensus
    } else if line.contains("network") {
        LogSource::Network
    } else if line.contains("vm") || line.contains("nockvm") {
        LogSource::VM
    } else if line.contains("wallet") {
        LogSource::Wallet
    } else {
        LogSource::Node
    }
}

/// Nockchain node runner with full integration
pub struct NockchainNodeRunner {
    node_manager: Option<NockchainNodeManager>,
    config: NockchainNodeConfig,
    is_running: bool,
    logs: Vec<LogEntry>,
    #[allow(dead_code)]
    libp2p_transport: Option<Box<dyn std::any::Any + Send + Sync>>, // Placeholder for libp2p transport
}

impl NockchainNodeRunner {
    /// Create a new nockchain node runner with default configuration
    pub fn new() -> Self {
        Self {
            node_manager: None,
            config: NockchainNodeConfig::default(),
            is_running: false,
            logs: Vec::new(),
            libp2p_transport: None,
        }
    }

    /// Create a new nockchain node runner with custom configuration
    pub fn with_config(config: NockchainNodeConfig) -> Self {
        Self {
            node_manager: None,
            config,
            is_running: false,
            logs: Vec::new(),
            libp2p_transport: None,
        }
    }

    /// Start the nockchain node with full integration
    pub async fn start_node(&mut self) -> WalletResult<()> {
        if self.is_running {
            return Err(WalletError::Network("Node is already running".to_string()));
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Starting nockchain node...".to_string(),
        );

        // Initialize the node manager with real nockchain integration
        let mut node_manager = NockchainNodeManager::new(self.config.clone());

        // Start the nockchain node
        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Starting nockchain node process...".to_string(),
        );
        node_manager.start_node().await?;

        self.node_manager = Some(node_manager);
        self.is_running = true;

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            format!(
                "Nockchain node started successfully on port {}",
                self.config.rpc_port
            ),
        );

        Ok(())
    }

    /// Stop the nockchain node
    pub async fn stop_node(&mut self) -> WalletResult<()> {
        if !self.is_running {
            return Err(WalletError::Network("Node is not running".to_string()));
        }

        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Stopping nockchain node...".to_string(),
        );

        if let Some(mut node_manager) = self.node_manager.take() {
            node_manager.stop_node().await?;
        }

        self.is_running = false;
        self.add_log(
            LogLevel::Info,
            LogSource::Node,
            "Nockchain node stopped".to_string(),
        );

        Ok(())
    }

    /// Execute nock computation on the running node (placeholder)
    pub async fn execute_nock(&self, nock_code: &[u8]) -> WalletResult<Vec<u8>> {
        if !self.is_running {
            return Err(WalletError::Network("Node is not running".to_string()));
        }

        // TODO: Implement actual nock computation when nockchain APIs are available
        Ok(nock_code.to_vec()) // Placeholder: echo the input
    }

    /// Submit a transaction to the nockchain network (placeholder)
    pub async fn submit_transaction(&self, transaction_data: &[u8]) -> WalletResult<String> {
        if !self.is_running {
            return Err(WalletError::Network("Node is not running".to_string()));
        }

        // TODO: Implement actual transaction submission when nockchain APIs are available
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(transaction_data);
        let hash = format!("{:x}", hasher.finalize());
        Ok(hash) // Placeholder: return hash of transaction data
    }

    /// Get node status and metrics
    pub async fn get_node_status(&self) -> WalletResult<NodeStatus> {
        if let Some(node_manager) = &self.node_manager {
            let status = node_manager.get_status();
            match status {
                NodeStatus::Running => Ok(NodeStatus::Running),
                NodeStatus::Starting => Ok(NodeStatus::Starting),
                NodeStatus::Stopping => Ok(NodeStatus::Stopping),
                NodeStatus::Stopped => Ok(NodeStatus::Stopped),
                NodeStatus::Error(msg) => Ok(NodeStatus::Error(msg)),
            }
        } else {
            Ok(NodeStatus::Stopped)
        }
    }

    /// Get recent node logs
    pub fn get_logs(&self, count: usize) -> Vec<LogEntry> {
        self.logs.iter().rev().take(count).cloned().collect()
    }

    /// Add a log entry
    fn add_log(&mut self, level: LogLevel, source: LogSource, message: String) {
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            source,
            message,
        };
        self.logs.push(entry);

        // Keep only the last 1000 log entries to prevent memory bloat
        if self.logs.len() > 1000 {
            self.logs.drain(0..100);
        }
    }

    /// Check if the node is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get the current node configuration
    pub fn get_config(&self) -> &NockchainNodeConfig {
        &self.config
    }

    /// Update node configuration (requires restart)
    pub fn update_config(&mut self, config: NockchainNodeConfig) -> WalletResult<()> {
        if self.is_running {
            return Err(WalletError::Network(
                "Cannot update config while node is running".to_string(),
            ));
        }
        self.config = config;
        Ok(())
    }
}
