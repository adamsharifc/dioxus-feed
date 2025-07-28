mod components;

use dioxus::prelude::*;
use components::virtual_list::VirtualList;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        VirtualList {}
    }
}

