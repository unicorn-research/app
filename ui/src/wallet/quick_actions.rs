use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct QuickActionsProps {
    pub on_send: EventHandler<()>,
    pub on_receive: EventHandler<()>,
    pub on_swap: Option<EventHandler<()>>,
    pub on_buy: Option<EventHandler<()>>,
}

pub fn QuickActions(props: QuickActionsProps) -> Element {
    rsx! {
        div {
            class: "quick-actions",

            button {
                class: "action-button send",
                onclick: move |_| props.on_send.call(()),
                div { class: "action-icon", "↗" }
                span { "Send" }
            }

            button {
                class: "action-button receive",
                onclick: move |_| props.on_receive.call(()),
                div { class: "action-icon", "↙" }
                span { "Receive" }
            }

            if let Some(on_swap) = props.on_swap {
                button {
                    class: "action-button swap",
                    onclick: move |_| on_swap.call(()),
                    div { class: "action-icon", "⇄" }
                    span { "Swap" }
                }
            }

            if let Some(on_buy) = props.on_buy {
                button {
                    class: "action-button buy",
                    onclick: move |_| on_buy.call(()),
                    div { class: "action-icon", "+" }
                    span { "Buy" }
                }
            }
        }

        style { {QUICK_ACTIONS_CSS} }
    }
}

const QUICK_ACTIONS_CSS: &str = r#"
.quick-actions {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 16px;
    margin-bottom: 32px;
    padding: 0 4px;
}

.action-button {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 20px 16px;
    border: none;
    border-radius: 16px;
    background: white;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
    cursor: pointer;
    transition: all 0.2s ease;
    min-height: 100px;
    position: relative;
    overflow: hidden;
}

.action-button::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, transparent 0%, transparent 100%);
    transition: all 0.3s ease;
    z-index: 1;
}

.action-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.12);
}

.action-button:active {
    transform: translateY(0);
}

.action-button.send::before {
    background: linear-gradient(135deg, rgba(255, 107, 107, 0.1) 0%, rgba(255, 107, 107, 0.05) 100%);
}

.action-button.receive::before {
    background: linear-gradient(135deg, rgba(46, 213, 115, 0.1) 0%, rgba(46, 213, 115, 0.05) 100%);
}

.action-button.swap::before {
    background: linear-gradient(135deg, rgba(0, 123, 255, 0.1) 0%, rgba(0, 123, 255, 0.05) 100%);
}

.action-button.buy::before {
    background: linear-gradient(135deg, rgba(255, 193, 7, 0.1) 0%, rgba(255, 193, 7, 0.05) 100%);
}

.action-icon {
    font-size: 24px;
    margin-bottom: 8px;
    font-weight: bold;
    z-index: 2;
    position: relative;
}

.action-button.send .action-icon {
    color: #ff6b6b;
}

.action-button.receive .action-icon {
    color: #2ed573;
}

.action-button.swap .action-icon {
    color: #007bff;
}

.action-button.buy .action-icon {
    color: #ffc107;
}

.action-button span {
    font-size: 14px;
    font-weight: 600;
    color: #333;
    z-index: 2;
    position: relative;
}

@media (max-width: 768px) {
    .quick-actions {
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
        margin-bottom: 24px;
    }
    
    .action-button {
        padding: 16px 12px;
        min-height: 80px;
    }
    
    .action-icon {
        font-size: 20px;
        margin-bottom: 6px;
    }
    
    .action-button span {
        font-size: 12px;
    }
}

@media (max-width: 480px) {
    .quick-actions {
        gap: 8px;
    }
    
    .action-button {
        padding: 12px 8px;
        min-height: 70px;
        border-radius: 12px;
    }
}
"#;
