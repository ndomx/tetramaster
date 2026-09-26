use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        main {
            class: "flex min-h-screen items-center justify-center bg-slate-950 text-slate-100",
            div {
                class: "text-center",
                h1 { class: "text-4xl font-bold", "Tetra Master" }
                p { class: "mt-3 text-slate-400", "Web interface coming soon." }
            }
        }
    }
}
