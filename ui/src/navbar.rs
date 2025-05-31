use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav {
            class: "navbar",
            div {
                class: "nav-brand",
                Link { to: "/", "🦄 Nockchain" }
            }
            div {
                class: "nav-links",
                Link { to: "/", class: "nav-link", "Wallet" }
                Link { to: "/node", class: "nav-link", "Node" }
                a { href: "#settings", class: "nav-link", "Settings" }
            }
        }

        style { {NAVBAR_CSS} }
    }
}

const NAVBAR_CSS: &str = r#"
.navbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    background: #1a1a1a;
    color: white;
    border-radius: 8px;
    margin-bottom: 20px;
}

.nav-brand {
    font-size: 1.5rem;
    font-weight: 700;
}

.nav-brand a {
    color: white;
    text-decoration: none;
}

.nav-links {
    display: flex;
    gap: 20px;
}

.nav-link {
    color: white;
    text-decoration: none;
    padding: 8px 16px;
    border-radius: 6px;
    transition: background-color 0.2s;
}

.nav-link:hover {
    background: rgba(255, 255, 255, 0.1);
}

@media (max-width: 768px) {
    .navbar {
        flex-direction: column;
        gap: 15px;
    }
    
    .nav-links {
        gap: 15px;
    }
}
"#;
