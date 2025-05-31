use dioxus::prelude::*;

#[component]
pub fn Echo() -> Element {
    let mut response = use_signal(String::new);

    rsx! {
        div {
            class: "echo-container",
            h2 { "Echo Test" }
            input {
                placeholder: "Type something...",
                oninput: move |event| {
                    response.set(api::echo_string(event.value()));
                }
            }
            p { "Response: {response}" }
        }

        style { {ECHO_CSS} }
    }
}

const ECHO_CSS: &str = r#"
.echo-container {
    padding: 20px;
    border: 1px solid #ccc;
    border-radius: 8px;
    margin: 20px 0;
}

.echo-container h2 {
    margin-top: 0;
}

.echo-container input {
    width: 100%;
    padding: 8px;
    margin: 10px 0;
    border: 1px solid #ddd;
    border-radius: 4px;
}
"#;
