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
    register_myprotocol_handler(vec!["assets".to_string()]); 

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        
        div {
            style: "
            	font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                box-sizing: border-box;
                margin: 0;
                padding: 20px;
                background: #f8fafc;
                min-height: 100vh;
                display: flex;
                justify-content: center;
                align-items: center;
            ",

            div {
                style: "
                    max-width: 800px;
                    width: 100%;
                    background: white;
                    border-radius: 8px;
                    border: 1px solid #e2e8f0;
                    overflow: hidden;
                    padding: 20px;
                    box-sizing: border-box;
                ",
                
                header {
                    class: "feed-header",
                    h1 {
                        class: "feed-title",
                        "Feed"
                    }
                    p {
                        class: "feed-desc",
                        "Latest updates"
                    }
                }
                
                VirtualList {}
            }
        }
    }
}

