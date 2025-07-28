mod components;

use dioxus::prelude::*;
use components::virtual_list::VirtualList;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let items = (0..100).map(|i| format!("Image {}", i)).collect::<Vec<_>>();
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        VirtualList {
            items: items,
            visible_count: 5,
            item_height: 100.0
        }
    }
}

