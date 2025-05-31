use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ReceiveViewProps {
    pub address: String,
}

pub fn ReceiveView(props: ReceiveViewProps) -> Element {
    rsx! {
        div {
            class: "receive-view",
            h3 { "Receive Nockchain" }
            div { class: "qr-code-placeholder", "QR Code Here" }
            div { class: "address", "{props.address}" }
        }
    }
}
