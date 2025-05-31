use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SendFormProps {
    pub on_send: EventHandler<(String, u64)>, // (address, amount)
}

pub fn SendForm(props: SendFormProps) -> Element {
    rsx! {
        div {
            class: "send-form",
            h3 { "Send Nockchain" }
            form {
                input { placeholder: "Recipient Address" }
                input { placeholder: "Amount" }
                button { "Send" }
            }
        }
    }
}
