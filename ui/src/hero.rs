use dioxus::prelude::*;

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            class: "hero",
            div {
                class: "hero-content",
                h1 { "🦄 Nockchain Wallet" }
                p { "A secure, self-sovereign wallet with built-in full node support" }
                div {
                    class: "hero-features",
                    div { class: "feature", "🔐 Secure Key Management" }
                    div { class: "feature", "⚡ Built-in Full Node" }
                    div { class: "feature", "🌐 Cross-Platform" }
                }
            }
        }

        style { {HERO_CSS} }
    }
}

const HERO_CSS: &str = r#"
.hero {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 60px 20px;
    text-align: center;
    border-radius: 12px;
    margin-bottom: 30px;
}

.hero-content h1 {
    font-size: 3rem;
    margin: 0 0 20px 0;
    font-weight: 700;
}

.hero-content p {
    font-size: 1.2rem;
    margin: 0 0 30px 0;
    opacity: 0.9;
}

.hero-features {
    display: flex;
    justify-content: center;
    gap: 30px;
    flex-wrap: wrap;
}

.feature {
    background: rgba(255, 255, 255, 0.1);
    padding: 15px 20px;
    border-radius: 8px;
    backdrop-filter: blur(10px);
    font-weight: 500;
}

@media (max-width: 768px) {
    .hero-content h1 {
        font-size: 2rem;
    }
    
    .hero-features {
        flex-direction: column;
        align-items: center;
    }
}
"#;
