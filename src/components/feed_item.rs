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
    rsx! {
        div {
            h2 { "{props.content}" }
            p { "Posted on: {props.timestamp}" }
            img { src: props.image_url.as_str(), alt: "Feed item image" }
        }
    }
}