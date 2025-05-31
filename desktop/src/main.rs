use api::wallet::network::{LogEntry, LogLevel, LogSource, NockchainNodeRunner, NodeStatus};
use api::Balance;
use dioxus::prelude::*;
use std::sync::{Arc, Mutex};
use ui::{BalanceCard, Hero, Navbar, NodeConsole};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
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
        div {
            style: "padding: 20px; max-width: 1200px; margin: 0 auto; font-family: system-ui, sans-serif;",
            Navbar {}
            Outlet::<Route> {}
        }
    }
}

#[component]
fn Home() -> Element {
    let balance = Balance {
        confirmed: 1000000,
        unconfirmed: 50000,
        locked: 0,
    };

    rsx! {
        Layout {}
        div {
            Hero {}
            BalanceCard {
                balance: balance,
                is_loading: false,
            }
            div {
                style: "text-align: center; margin-top: 40px;",
                h3 { "Welcome to Nockchain Wallet" }
                p { "Your secure, self-sovereign wallet with built-in full node support." }
                p { "Navigate to the Node tab to manage your nockchain full node." }
            }
        }
    }
}

#[component]
fn Node() -> Element {
    // Create a shared node runner instance
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
                message: "Node ready to start. Click the start button to begin.".to_string(),
            }]);
        }
    });

    let start_node_handler = move |_| {
        let node_runner_clone = node_runner.clone();

        is_starting.set(true);
        node_status.set(NodeStatus::Starting);

        spawn(async move {
            let result = {
                let mut runner = node_runner_clone.read().lock().unwrap();
                runner.start_node().await
            };

            match result {
                Ok(()) => {
                    node_status.set(NodeStatus::Running);
                    // Get the latest logs from the node runner
                    let node_logs = {
                        let runner = node_runner_clone.read().lock().unwrap();
                        runner.get_logs(50)
                    };
                    logs.set(node_logs);
                }
                Err(e) => {
                    node_status.set(NodeStatus::Error(format!("Failed to start node: {}", e)));
                    let mut current_logs = logs.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Error,
                        source: LogSource::Node,
                        message: format!("Failed to start node: {}", e),
                    });
                    logs.set(current_logs);
                }
            }
            is_starting.set(false);
        });
    };

    let stop_node_handler = move |_| {
        let node_runner_clone = node_runner.clone();

        is_stopping.set(true);
        node_status.set(NodeStatus::Stopping);

        spawn(async move {
            let mut runner = node_runner_clone.read().lock().unwrap();

            match runner.stop_node().await {
                Ok(()) => {
                    node_status.set(NodeStatus::Stopped);
                    // Get the latest logs from the node runner
                    let node_logs = runner.get_logs(50);
                    logs.set(node_logs);
                }
                Err(e) => {
                    node_status.set(NodeStatus::Error(format!("Failed to stop node: {}", e)));
                    let mut current_logs = logs.read().clone();
                    current_logs.push(LogEntry {
                        timestamp: chrono::Utc::now(),
                        level: LogLevel::Error,
                        source: LogSource::Node,
                        message: format!("Failed to stop node: {}", e),
                    });
                    logs.set(current_logs);
                }
            }
            is_stopping.set(false);
        });
    };

    // Periodic log updates from the running node
    use_effect(move || {
        let node_runner_clone = node_runner.clone();
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let runner = node_runner_clone.read().lock().unwrap();
                if runner.is_running() {
                    let node_logs = runner.get_logs(50);
                    if !node_logs.is_empty() {
                        logs.set(node_logs);
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

    rsx! {
        Layout {}
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

            ui::wallet::NodeConsole {
                status: node_status.read().clone(),
                logs: filtered_logs,
                on_start_node: start_node_handler,
                on_stop_node: stop_node_handler,
                is_starting: *is_starting.read(),
                is_stopping: *is_stopping.read(),
            }

            // Node configuration info
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
                        span { style: "color: #28a745; font-weight: 600;", "Mainnet (Dumbnet)" }
                    }
                    div {
                        strong { "P2P Port: " }
                        "4001"
                    }
                    div {
                        strong { "RPC Port: " }
                        "8332"
                    }
                    div {
                        strong { "Genesis Watcher: " }
                        span { style: "color: #007bff;", "Enabled" }
                    }
                    div {
                        strong { "Mining: " }
                        span { style: "color: #6c757d;", "Disabled" }
                    }
                    div {
                        strong { "Max Peers: " }
                        "225 (150 in, 75 out)"
                    }
                }

                div {
                    style: "margin-top: 16px; padding-top: 16px; border-top: 1px solid #dee2e6;",
                    h4 {
                        style: "color: #333; margin-bottom: 8px; font-size: 14px;",
                        "Bootstrap Peers (10 nodes)"
                    }
                    div {
                        style: "font-family: monospace; font-size: 12px; color: #6c757d; line-height: 1.4; display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                        div {
                            "• 104.131.131.131:4001" br {}
                            "• 134.209.28.98:4001" br {}
                            "• 143.198.57.46:4001" br {}
                            "• 165.227.41.207:4001" br {}
                            "• 68.183.123.45:4001"
                        }
                        div {
                            "• 174.138.45.123:4001" br {}
                            "• 159.203.188.97:4001" br {}
                            "• 207.154.231.65:4001" br {}
                            "• 128.199.47.89:4001" br {}
                            "• 188.166.204.102:4001"
                        }
                    }
                }
            }
        }
    }
}
