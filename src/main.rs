mod components;
mod protocol;

use dioxus::prelude::*;
use components::virtual_list::VirtualList;
use protocol::myprotocol::register_myprotocol_handler;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::LaunchBuilder::desktop()
        .launch(App);
}

#[component]
fn App() -> Element {
    // Register the asset handler for myprotocol in a modular way
    register_myprotocol_handler(vec!["*".to_string()]); 

    let items = (0..100).map(|i| format!("Image {}", i)).collect::<Vec<_>>();
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        // div {
        //     h1 { "Testing Custom Protocol" }
            
        //     // Test the custom protocol directly
        //     img { 
        //         src: "/myprotocol/C:/Users/adams/Desktop/photo-1753109910060-ba1fa8fbd094.avif",
        //         width: "200px",
        //     }

        // }

        VirtualList {
            items: items,
            container_height: 960,
            item_height: 100
        }
    }
}

