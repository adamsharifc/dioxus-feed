use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct VirtualListProps {
    pub items: Vec<String>,
    pub container_height: usize,
    pub item_height: usize,
}

#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let items = &props.items;
    let container_height = props.container_height;
    let item_height = props.item_height;

    // Calculate visible count dynamically
    let visible_count = container_height / item_height;

    // Track scroll position using onscroll event
    let mut scroll_top = use_signal(|| 0usize);

    // Calculate layout
    let total_content_height = items.len() * item_height;

    // Calculate visible range with buffer
    let start_index = scroll_top() / item_height;
    let end_index = (start_index + visible_count + 2).min(items.len()); // +2 for buffer

    rsx! {
        div {
            style: "
                width: 800px;
                height: {container_height}px;
                overflow: auto;
                border: 1px solid red;
                background: #fff;
                position: relative;
            ",
            
            onscroll: move |evt| {
                let data = evt.data();
                let current_scroll_top = data.scroll_top() as usize;
                scroll_top.set(current_scroll_top);
            },

            // Content container with full height for scrollbar
            div {
                style: "height: {total_content_height}px; position: relative;",
                
                // Only render visible items (virtualization)
                for i in start_index..end_index {
                    if i < items.len() {
                        div {
                            key: "{i}",
                            style: "
                                position: absolute;
                                top: {i * item_height}px;
                                width: 100%;
                                height: {item_height}px;
                                border: 1px solid red;
                                display: flex;
                                align-items: center;
                                background: #fff;
                            ",

                            img {
                                src: "https://picsum.photos/id/{i}/100/100",
                                width: "100",
                                height: "100",
                                alt: "{items[i]}",
                                style: "margin-right: 16px; border: 1px solid #ccc;"
                            },

                            div {
                                "{items[i]}"
                            }
                        }
                    }
                }
            }
        }
    }
}
