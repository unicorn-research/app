use api::Transaction;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TransactionListProps {
    pub transactions: Vec<Transaction>,
    pub is_loading: bool,
}

pub fn TransactionList(props: TransactionListProps) -> Element {
    rsx! {
        div {
            class: "transaction-list",
            h3 { "Recent Transactions" }
            if props.is_loading {
                div { "Loading transactions..." }
            } else if props.transactions.is_empty() {
                div { class: "empty-state", "No transactions yet" }
            } else {
                for transaction in props.transactions {
                    div {
                        key: "{transaction.id}",
                        class: "transaction-item",
                        div { "{transaction.id}" }
                        div { "{transaction.amount}" }
                    }
                }
            }
        }
    }
}
