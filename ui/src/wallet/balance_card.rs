use api::Balance;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct BalanceCardProps {
    pub balance: Balance,
    pub is_loading: bool,
}

pub fn BalanceCard(props: BalanceCardProps) -> Element {
    let balance = props.balance;
    let is_loading = props.is_loading;

    rsx! {
        div {
            class: "balance-card",
            div {
                class: "balance-header",
                h2 { class: "balance-title", "Total Balance" }
                if is_loading {
                    div { class: "loading-spinner" }
                } else {
                    button {
                        class: "refresh-button",
                        onclick: move |_| {
                            // TODO: Implement balance refresh
                        },
                        "↻"
                    }
                }
            }

            div { class: "balance-main" }
            if is_loading {
                div { class: "balance-loading", "Loading..." }
            } else {
                div { class: "balance-amount" }
                span { class: "balance-value", "{format_balance(balance.total())}" }
                span { class: "balance-currency", "NOCK" }
            }

            div { class: "balance-details" }
            div { class: "balance-row" }
            span { class: "balance-label", "Available:" }
            span { class: "balance-amount-small", "{format_balance(balance.available())}" }

            if balance.unconfirmed > 0 {
                div { class: "balance-row" }
                span { class: "balance-label", "Pending:" }
                span { class: "balance-amount-small pending", "{format_balance(balance.unconfirmed)}" }
            }

            if balance.locked > 0 {
                div { class: "balance-row" }
                span { class: "balance-label", "Locked:" }
                span { class: "balance-amount-small locked", "{format_balance(balance.locked)}" }
            }
        }

        style { {BALANCE_CARD_CSS} }
    }
}

fn format_balance(amount: u64) -> String {
    let nock_amount = amount as f64 / 1_000_000.0; // Assuming 6 decimal places
    format!("{:.6}", nock_amount)
}

const BALANCE_CARD_CSS: &str = r#"
.balance-card {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border-radius: 20px;
    padding: 24px;
    color: white;
    box-shadow: 0 10px 30px rgba(102, 126, 234, 0.3);
    margin-bottom: 24px;
    position: relative;
    overflow: hidden;
}

.balance-card::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><defs><pattern id="grain" patternUnits="userSpaceOnUse" width="100" height="100"><circle cx="25" cy="25" r="1" fill="white" opacity="0.1"/><circle cx="75" cy="75" r="1" fill="white" opacity="0.1"/><circle cx="25" cy="75" r="1" fill="white" opacity="0.1"/><circle cx="75" cy="25" r="1" fill="white" opacity="0.1"/></pattern></defs><rect width="100" height="100" fill="url(%23grain)"/></svg>');
    pointer-events: none;
}

.balance-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.balance-title {
    font-size: 16px;
    font-weight: 500;
    margin: 0;
    opacity: 0.9;
}

.refresh-button {
    background: rgba(255, 255, 255, 0.2);
    border: none;
    border-radius: 8px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: white;
    font-size: 16px;
    transition: all 0.2s ease;
}

.refresh-button:hover {
    background: rgba(255, 255, 255, 0.3);
    transform: rotate(180deg);
}

.loading-spinner {
    width: 20px;
    height: 20px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top: 2px solid white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

.balance-main {
    margin-bottom: 20px;
}

.balance-amount {
    display: flex;
    align-items: baseline;
    gap: 8px;
}

.balance-value {
    font-size: 36px;
    font-weight: 700;
    line-height: 1;
}

.balance-currency {
    font-size: 16px;
    font-weight: 500;
    opacity: 0.8;
}

.balance-loading {
    font-size: 24px;
    opacity: 0.7;
    text-align: center;
    padding: 20px 0;
}

.balance-details {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
}

.balance-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.balance-label {
    font-size: 14px;
    opacity: 0.8;
}

.balance-amount-small {
    font-size: 14px;
    font-weight: 600;
}

.balance-amount-small.pending {
    color: #ffd700;
}

.balance-amount-small.locked {
    color: #ff6b6b;
}

@media (max-width: 768px) {
    .balance-card {
        padding: 20px;
        margin-bottom: 20px;
    }
    
    .balance-value {
        font-size: 28px;
    }
}
"#;
