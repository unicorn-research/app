use api::wallet::network::{LogEntry, LogLevel, LogSource, NockchainNodeRunner, NodeStatus};
use api::wallet::WalletError;
use api::Balance;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use ui::{BalanceCard, Hero, Navbar, NodeConsole};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/node")]
    Node {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        div { style: "min-height: 100vh; display: flex; flex-direction: column;",
            Navbar {}
            main { style: "flex: 1; padding: 20px;",
                Outlet::<Route> {}
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let balance = Balance {
        confirmed: 0,
        unconfirmed: 0,
        locked: 0,
    };

    rsx! {
        div {
            Hero {}
            BalanceCard { balance, is_loading: false }

            div { style: "margin-top: 40px;",
                h2 { style: "color: #333; margin-bottom: 20px;", "Quick Actions" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 20px;",
                    div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center;",
                        h3 { style: "color: #333; margin-bottom: 10px;", "Send" }
                        p { style: "color: #666; margin-bottom: 15px;", "Send funds to another address" }
                        button { style: "background: #007bff; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer;", "Send Funds" }
                    }
                    div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center;",
                        h3 { style: "color: #333; margin-bottom: 10px;", "Receive" }
                        p { style: "color: #666; margin-bottom: 15px;", "Generate a receive address" }
                        button { style: "background: #28a745; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer;", "Get Address" }
                    }
                    div { style: "background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center;",
                        h3 { style: "color: #333; margin-bottom: 10px;", "Node" }
                        p { style: "color: #666; margin-bottom: 15px;", "Manage your nockchain node" }
                        button { style: "background: #6f42c1; color: white; padding: 10px 20px; border: none; border-radius: 4px; cursor: pointer;", "Node Settings" }
                    }
                }
            }
        }
    }
}

#[component]
fn Node() -> Element {
    // Create a shared node runner instance with proper Arc<Mutex<>> handling
    let node_runner = use_signal(|| Arc::new(Mutex::new(NockchainNodeRunner::new())));
    let mut node_status = use_signal(|| NodeStatus::Stopped);
    let mut logs = use_signal(|| Vec::<LogEntry>::new());
    let mut is_starting = use_signal(|| false);
    let mut is_stopping = use_signal(|| false);
    let mut log_level = use_signal(|| LogLevel::Info);
    let mut auto_scroll = use_signal(|| true);

    // Initialize with basic startup log
    use_effect(move || {
        if logs.read().is_empty() {
            logs.set(vec![LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                source: LogSource::Node,
                message: "Nockchain node ready to start. Click Start Node to begin.".to_string(),
            }]);
        }
    });

    let start_node_handler = move |_| {
        let node_runner_clone = node_runner.clone();
        let mut is_starting_clone = is_starting.clone();
        let mut node_status_clone = node_status.clone();
        let mut logs_clone = logs.clone();

        // Prevent multiple start attempts
        if *is_starting.read()
            || matches!(
                *node_status.read(),
                NodeStatus::Running | NodeStatus::Starting
            )
        {
            return;
        }

        is_starting.set(true);
        node_status.set(NodeStatus::Starting);

        // Add initial log immediately
        {
            let mut current_logs = logs_clone.read().clone();
            current_logs.push(LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Info,
                source: LogSource::Node,
                message: "🚀 Starting nockchain node with libraries...".to_string(),
            });
            logs_clone.set(current_logs);
        }

        spawn(async move {
            // Add timeout protection
            let start_result = tokio::time::timeout(tokio::time::Duration::from_secs(30), async {
                // Try to acquire lock with timeout
                let runner_result =
                    tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
                        match node_runner_clone.read().lock() {
                            Ok(mut runner) => {
                                // Add progress log
                                let mut current_logs = logs_clone.read().clone();
                                current_logs.push(LogEntry {
                                    timestamp: chrono::Utc::now(),
                                    level: LogLevel::Info,
                                    source: LogSource::Node,
                                    message: "🔧 Initializing node components...".to_string(),
                                });
                                logs_clone.set(current_logs);

                                runner.start_node().await
                            }
                            Err(e) => Err(WalletError::Network(format!("Lock error: {}", e))),
                        }
                    })
                    .await;

                match runner_result {
                    Ok(result) => result,
                    Err(_) => Err(WalletError::Network(
                        "Timeout acquiring node lock".to_string(),
                    )),
                }
            })
            .await;

            // Handle the result
            match start_result {
                Ok(Ok(())) => {
                    node_status_clone.set(NodeStatus::Running);
                    let mut current_logs = logs_clone.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Info,
                        source: LogSource::Node,
                        message: "✅ Node started successfully!".to_string(),
                    });
                    logs_clone.set(current_logs);

                    // Get fresh logs from node
                    if let Ok(runner) = node_runner_clone.read().lock() {
                        let node_logs = runner.get_logs(50);
                        if !node_logs.is_empty() {
                            logs_clone.set(node_logs);
                        }
                    }
                }
                Ok(Err(e)) => {
                    let error_msg = format!("❌ Failed to start node: {}", e);
                    node_status_clone.set(NodeStatus::Error(error_msg.clone()));
                    let mut current_logs = logs_clone.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Error,
                        source: LogSource::Node,
                        message: error_msg,
                    });
                    logs_clone.set(current_logs);
                }
                Err(_) => {
                    let error_msg = "⏰ Node start timeout after 30 seconds".to_string();
                    node_status_clone.set(NodeStatus::Error(error_msg.clone()));
                    let mut current_logs = logs_clone.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Error,
                        source: LogSource::Node,
                        message: error_msg,
                    });
                    logs_clone.set(current_logs);
                }
            }

            is_starting_clone.set(false);
        });
    };

    let stop_node_handler = move |_| {
        let node_runner_clone = node_runner.clone();
        let mut is_stopping_clone = is_stopping.clone();
        let mut node_status_clone = node_status.clone();
        let mut logs_clone = logs.clone();

        is_stopping.set(true);
        node_status.set(NodeStatus::Stopping);

        spawn(async move {
            // Safely handle the mutex lock
            let stop_result = match node_runner_clone.read().lock() {
                Ok(mut runner) => runner.stop_node().await,
                Err(e) => Err(WalletError::Network(format!(
                    "Failed to acquire node runner lock: {}",
                    e
                ))),
            };

            match stop_result {
                Ok(()) => {
                    node_status_clone.set(NodeStatus::Stopped);
                    // Get the latest logs from the node runner
                    if let Ok(runner) = node_runner_clone.read().lock() {
                        let node_logs = runner.get_logs(50);
                        logs_clone.set(node_logs);
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to stop node: {}", e);
                    node_status_clone.set(NodeStatus::Error(error_msg.clone()));
                    let mut current_logs = logs_clone.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Error,
                        source: LogSource::Node,
                        message: error_msg,
                    });
                    logs_clone.set(current_logs);
                }
            }
            is_stopping_clone.set(false);
        });
    };

    // Periodic log updates from the running node with timeout protection
    use_effect(move || {
        let node_runner_clone = node_runner.clone();
        let mut logs_clone = logs.clone();
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Try to get logs with timeout to prevent hanging
                let log_result = tokio::time::timeout(tokio::time::Duration::from_secs(1), async {
                    if let Ok(runner) = node_runner_clone.read().lock() {
                        if runner.is_running() {
                            Some(runner.get_logs(50))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .await;

                if let Ok(Some(node_logs)) = log_result {
                    if !node_logs.is_empty() && node_logs.len() != logs_clone.read().len() {
                        logs_clone.set(node_logs);
                    }
                }
            }
        });
    });

    // Filter logs based on selected level
    let filtered_logs = logs
        .read()
        .iter()
        .filter(|log| {
            match *log_level.read() {
                LogLevel::Trace => true, // Show all
                LogLevel::Debug => !matches!(log.level, LogLevel::Trace),
                LogLevel::Info => {
                    matches!(log.level, LogLevel::Info | LogLevel::Warn | LogLevel::Error)
                }
                LogLevel::Warn => matches!(log.level, LogLevel::Warn | LogLevel::Error),
                LogLevel::Error => matches!(log.level, LogLevel::Error),
            }
        })
        .cloned()
        .collect::<Vec<_>>();

    // Get current node configuration for display
    let node_config = {
        if let Ok(runner) = node_runner.read().lock() {
            runner.get_config().clone()
        } else {
            // Fallback to default config if lock fails
            api::wallet::network::NockchainNodeConfig::default()
        }
    };

    rsx! {
        div {
                h2 {
                style: "color: #333; margin-bottom: 24px; display: flex; align-items: center; gap: 12px;",
                "🦄 Node Management"
            }
            p {
                style: "color: #666; margin-bottom: 24px; font-size: 16px;",
                "Manage your nockchain full node. Start the node to participate in the network, mine blocks, and validate transactions."
            }

            // Logging controls
            div {
                style: "background: #f8f9fa; padding: 16px; border-radius: 8px; margin-bottom: 16px; display: flex; align-items: center; gap: 20px; flex-wrap: wrap;",
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    label {
                        style: "font-weight: 600; color: #333;",
                        "Log Level:"
                    }
                    select {
                        style: "padding: 6px 12px; border: 1px solid #ccc; border-radius: 4px; background: white;",
                        onchange: move |evt| {
                            let level = match evt.value().as_str() {
                                "trace" => LogLevel::Trace,
                                "debug" => LogLevel::Debug,
                                "info" => LogLevel::Info,
                                "warn" => LogLevel::Warn,
                                "error" => LogLevel::Error,
                                _ => LogLevel::Info,
                            };
                            log_level.set(level);
                        },
                        option { value: "trace", "TRACE (All logs)" }
                        option { value: "debug", "DEBUG" }
                        option { value: "info", selected: true, "INFO" }
                        option { value: "warn", "WARN" }
                        option { value: "error", "ERROR" }
                    }
                }
                div {
                    style: "display: flex; align-items: center; gap: 8px;",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: *auto_scroll.read(),
                            onchange: move |evt| auto_scroll.set(evt.checked()),
                        }
                        span { style: "margin-left: 4px; color: #333;", "Auto-scroll" }
                    }
                }
                div {
                    style: "color: #666; font-size: 14px;",
                    "Showing {filtered_logs.len()} / {logs.read().len()} logs"
                }
            }

            NodeConsole {
                status: node_status.read().clone(),
                logs: filtered_logs,
                on_start_node: start_node_handler,
                on_stop_node: stop_node_handler,
                is_starting: *is_starting.read(),
                is_stopping: *is_stopping.read(),
            }

            // Node configuration info - using real config from node runner
            div {
                style: "background: #f8f9fa; padding: 20px; border-radius: 8px; margin-top: 24px;",
                h3 {
                    style: "color: #333; margin-bottom: 16px;",
                    "Node Configuration"
                }
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; color: #666;",
                    div {
                        strong { "Network: " }
                        if node_config.fakenet {
                            span { style: "color: #ffc107; font-weight: 600;", "Fakenet (Test)" }
                        } else {
                            span { style: "color: #28a745; font-weight: 600;", "Mainnet (Dumbnet)" }
                        }
                    }
                    div {
                        strong { "P2P Port: " }
                        "{node_config.p2p_port}"
                    }
                    div {
                        strong { "RPC Port: " }
                        "{node_config.rpc_port}"
                    }
                    div {
                        strong { "Genesis Watcher: " }
                        if node_config.genesis_watcher {
                            span { style: "color: #007bff;", "Enabled" }
                        } else {
                            span { style: "color: #6c757d;", "Disabled" }
                        }
                    }
                    div {
                        strong { "Mining: " }
                        if node_config.mining_enabled {
                            span { style: "color: #28a745;", "Enabled" }
                        } else {
                            span { style: "color: #6c757d;", "Disabled" }
                        }
                    }
                    div {
                        strong { "Max Peers: " }
                        if let (Some(incoming), Some(outgoing)) = (node_config.max_established_incoming, node_config.max_established_outgoing) {
                            "{incoming + outgoing} ({incoming} in, {outgoing} out)"
                        } else {
                            "Unlimited"
                        }
                    }
                }

                div {
                    style: "margin-top: 16px; padding-top: 16px; border-top: 1px solid #dee2e6;",
                    h4 {
                        style: "color: #333; margin-bottom: 8px; font-size: 14px;",
                        "Bootstrap Peers ({node_config.peers.len()} nodes)"
                    }
                    div {
                        style: "font-family: monospace; font-size: 12px; color: #6c757d; line-height: 1.4; max-height: 120px; overflow-y: auto;",
                        for peer in node_config.peers.iter() {
                            div { "• {peer}" }
                        }
                    }
                }

                div {
                    style: "margin-top: 16px; padding-top: 16px; border-top: 1px solid #dee2e6;",
                    h4 {
                        style: "color: #333; margin-bottom: 8px; font-size: 14px;",
                        "Data Directory"
                    }
                    div {
                        style: "font-family: monospace; font-size: 12px; color: #6c757d; word-break: break-all;",
                        "{node_config.data_dir.display()}"
                    }
                }
            }
        }
    }
}
