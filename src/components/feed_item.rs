use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct FeedItemProps {
    pub id: String,
    pub content: String,
    pub image_url: String,
    pub timestamp: u64,
}

#[component]
pub fn FeedItem(props: FeedItemProps) -> Element {
    let mut image_loaded = use_signal(|| false);
    let mut image_error = use_signal(|| false);
    
    rsx! {
        div {
            class: "feed-item-container",            
            // Content section
            div {
                class: "feed-item-content",
                "{props.content}"
            }
            
            // Image section with placeholder
            div {
                class: "feed-item-content-image-container",
                
                // Loading placeholder (shown until image loads)
                if !image_loaded() && !image_error() {
                    div {
                        class: "feed-item-content-loading-placeholder",
                        "Loading image..."
                    }
                }
                
                // Error state
                if image_error() {
                    div {
                        class: "feed-item-content-loading-error",
                        "Failed to load image"
                    }
                }
                
                // Actual image
                img {
                    src: props.image_url.as_str(),
                    alt: "Feed item image",
                    style: format!("
                        opacity: {};
                    ", if image_loaded() { "1" } else { "0" }),
                    onload: move |_| image_loaded.set(true),
                    onerror: move |_| image_error.set(true),
                }
            }
            
            // Metadata section
            div {
                class: "feed-item-content-metadata",
                div {
                    "ID: {props.id}"
                }
                div {
                    "Posted: {props.timestamp}"
                }
            }
        }
    }
}