use api::wallet::network::{LogEntry, LogLevel, NodeStatus};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct NodeConsoleProps {
    pub status: NodeStatus,
    pub logs: Vec<LogEntry>,
    pub on_start_node: EventHandler<()>,
    pub on_stop_node: EventHandler<()>,
    pub is_starting: bool,
    pub is_stopping: bool,
}

pub fn NodeConsole(props: NodeConsoleProps) -> Element {
    let status = props.status;
    let logs = props.logs;

    rsx! {
        div {
            class: "node-console",

            // Header with status and controls
            div {
                class: "node-header",
                div {
                    class: "node-status",
                    div {
                        class: "status-indicator {get_status_class(&status)}",
                    }
                    div {
                        class: "status-info",
                        h3 { class: "status-title", "Nockchain Node" }
                        span { class: "status-text", "{get_status_text(&status)}" }
                    }
                }

                div {
                    class: "node-controls",
                    match status {
                        NodeStatus::Stopped => rsx! {
                            button {
                                class: "control-button start",
                                onclick: move |_| props.on_start_node.call(()),
                                disabled: props.is_starting,
                                if props.is_starting {
                                    span { class: "spinner" }
                                    "Starting..."
                                } else {
                                    "▶ Start Node"
                                }
                            }
                        },
                        NodeStatus::Running => rsx! {
                            button {
                                class: "control-button stop",
                                onclick: move |_| props.on_stop_node.call(()),
                                disabled: props.is_stopping,
                                if props.is_stopping {
                                    span { class: "spinner" }
                                    "Stopping..."
                                } else {
                                    "⏹ Stop Node"
                                }
                            }
                        },
                        NodeStatus::Starting => rsx! {
                            button {
                                class: "control-button starting",
                                disabled: true,
                                span { class: "spinner" }
                                "Starting..."
                            }
                        },
                        NodeStatus::Stopping => rsx! {
                            button {
                                class: "control-button stopping",
                                disabled: true,
                                span { class: "spinner" }
                                "Stopping..."
                            }
                        },
                        NodeStatus::Error(_) => rsx! {
                            button {
                                class: "control-button start",
                                onclick: move |_| props.on_start_node.call(()),
                                "🔄 Restart"
                            }
                        },
                    }
                }
            }

            // Console logs
            div {
                class: "console-container",
                div {
                    class: "console-header",
                    h4 { "Console Output" }
                    div {
                        class: "log-count",
                        "{logs.len()} lines"
                    }
                }

                div {
                    class: "console-logs",
                    id: "console-logs",
                    if logs.is_empty() {
                        div {
                            class: "console-empty",
                            "No logs yet. Start the node to see output."
                        }
                    } else {
                        for (index, log) in logs.iter().enumerate() {
                            div {
                                key: "{index}",
                                class: "log-line {get_log_level_class(&log.level)}",
                                span { class: "log-time", "{format_timestamp(&log.timestamp)}" }
                                span { class: "log-level", "{format_log_level(&log.level)}" }
                                span { class: "log-source", "[{format_log_source(&log.source)}]" }
                                span { class: "log-message", "{log.message}" }
                            }
                        }
                    }
                }
            }
        }

        style { {NODE_CONSOLE_CSS} }
    }
}

fn get_status_class(status: &NodeStatus) -> &'static str {
    match status {
        NodeStatus::Stopped => "stopped",
        NodeStatus::Starting => "starting",
        NodeStatus::Running => "running",
        NodeStatus::Stopping => "stopping",
        NodeStatus::Error(_) => "error",
    }
}

fn get_status_text(status: &NodeStatus) -> String {
    match status {
        NodeStatus::Stopped => "Stopped".to_string(),
        NodeStatus::Starting => "Starting...".to_string(),
        NodeStatus::Running => "Running".to_string(),
        NodeStatus::Stopping => "Stopping...".to_string(),
        NodeStatus::Error(msg) => format!("Error: {}", msg),
    }
}

fn get_log_level_class(level: &LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%H:%M:%S").to_string()
}

fn format_log_level(level: &LogLevel) -> String {
    match level {
        LogLevel::Trace => "TRACE".to_string(),
        LogLevel::Debug => "DEBUG".to_string(),
        LogLevel::Info => "INFO".to_string(),
        LogLevel::Warn => "WARN".to_string(),
        LogLevel::Error => "ERROR".to_string(),
    }
}

fn format_log_source(source: &api::wallet::network::LogSource) -> String {
    match source {
        api::wallet::network::LogSource::Node => "NODE".to_string(),
        api::wallet::network::LogSource::Wallet => "WALLET".to_string(),
        api::wallet::network::LogSource::P2P => "P2P".to_string(),
        api::wallet::network::LogSource::Mining => "MINING".to_string(),
        api::wallet::network::LogSource::Consensus => "CONSENSUS".to_string(),
        api::wallet::network::LogSource::Network => "NETWORK".to_string(),
        api::wallet::network::LogSource::VM => "VM".to_string(),
        api::wallet::network::LogSource::Debug => "DEBUG".to_string(),
    }
}

const NODE_CONSOLE_CSS: &str = r#"
.node-console {
    background: #1a1a1a;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 24px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.node-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px 24px;
    background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
    color: white;
}

.node-status {
    display: flex;
    align-items: center;
    gap: 12px;
}

.status-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    position: relative;
}

.status-indicator.stopped {
    background: #64748b;
}

.status-indicator.starting {
    background: #f59e0b;
    animation: pulse 2s infinite;
}

.status-indicator.running {
    background: #10b981;
    box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
}

.status-indicator.stopping {
    background: #f59e0b;
    animation: pulse 2s infinite;
}

.status-indicator.error {
    background: #ef4444;
    animation: blink 1s infinite;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

@keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
}

.status-info h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
}

.status-text {
    font-size: 14px;
    opacity: 0.8;
}

.node-controls {
    display: flex;
    gap: 12px;
}

.control-button {
    padding: 8px 16px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 6px;
}

.control-button:disabled {
    cursor: not-allowed;
    opacity: 0.6;
}

.control-button.start {
    background: #10b981;
    color: white;
}

.control-button.start:hover:not(:disabled) {
    background: #059669;
}

.control-button.stop {
    background: #ef4444;
    color: white;
}

.control-button.stop:hover:not(:disabled) {
    background: #dc2626;
}

.control-button.starting,
.control-button.stopping {
    background: #6b7280;
    color: white;
}

.spinner {
    width: 12px;
    height: 12px;
    border: 2px solid transparent;
    border-top: 2px solid currentColor;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

.console-container {
    background: #000;
    color: #e5e7eb;
}

.console-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 20px;
    background: #111827;
    border-bottom: 1px solid #374151;
}

.console-header h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 500;
    color: #9ca3af;
}

.log-count {
    font-size: 12px;
    color: #6b7280;
}

.console-logs {
    max-height: 400px;
    overflow-y: auto;
    font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Roboto Mono', monospace;
    font-size: 12px;
    line-height: 1.4;
}

.console-empty {
    padding: 40px 20px;
    text-align: center;
    color: #6b7280;
    font-style: italic;
}

.log-line {
    padding: 2px 20px;
    border-bottom: 1px solid #1f2937;
    display: flex;
    gap: 8px;
    align-items: baseline;
}

.log-line:hover {
    background: #1f2937;
}

.log-time {
    color: #6b7280;
    min-width: 60px;
    font-size: 11px;
}

.log-level {
    min-width: 50px;
    font-weight: 600;
    font-size: 11px;
}

.log-source {
    color: #9ca3af;
    min-width: 60px;
    font-size: 11px;
}

.log-message {
    flex: 1;
    word-break: break-word;
}

.log-line.trace .log-level {
    color: #6b7280;
}

.log-line.debug .log-level {
    color: #8b5cf6;
}

.log-line.info .log-level {
    color: #10b981;
}

.log-line.warn .log-level {
    color: #f59e0b;
}

.log-line.error {
    background: rgba(239, 68, 68, 0.1);
}

.log-line.error .log-level {
    color: #ef4444;
}

.log-line.error .log-message {
    color: #fecaca;
}

/* Scrollbar styling */
.console-logs::-webkit-scrollbar {
    width: 8px;
}

.console-logs::-webkit-scrollbar-track {
    background: #1f2937;
}

.console-logs::-webkit-scrollbar-thumb {
    background: #4b5563;
    border-radius: 4px;
}

.console-logs::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
}

@media (max-width: 768px) {
    .node-header {
        flex-direction: column;
        gap: 16px;
        padding: 16px 20px;
    }
    
    .node-controls {
        width: 100%;
        justify-content: center;
    }
    
    .console-logs {
        max-height: 300px;
        font-size: 11px;
    }
    
    .log-line {
        flex-direction: column;
        gap: 4px;
        padding: 8px 16px;
    }
    
    .log-time,
    .log-level,
    .log-source {
        min-width: auto;
    }
}
"#;
