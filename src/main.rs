mod components;
mod protocol;

use dioxus::prelude::*;
use components::virtual_list::VirtualList;
use components::feed_item::FeedItem;
use components::feed::Feed;
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

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        
        div {
            class: "app-container",
            style: "padding: 20px; font-family: Arial, sans-serif;",
            
            
            Feed {}
        }


    }
}

