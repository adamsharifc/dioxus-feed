use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct VirtualListProps {
    pub items: Vec<String>,
    pub visible_count: usize,
    pub item_height: f64,
}

#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    // Constants for smooth scrolling
    const FRICTION: f64 = 0.95;
    const MIN_VELOCITY: f64 = 0.1;
    const WHEEL_MULTIPLIER: f64 = 0.5;
    const MAX_VELOCITY: f64 = 50.0;

    let items = &props.items;
    let visible_count = props.visible_count;
    let item_height = props.item_height;

    // Use pixel-based scrolling instead of item index
    let mut scroll_position = use_signal(|| 0.0_f64);
    let mut velocity = use_signal(|| 0.0_f64);
    let mut is_scrolling = use_signal(|| false);

    // Calculate scroll bounds
    let container_height = visible_count as f64 * item_height;
    let total_content_height = items.len() as f64 * item_height;
    let max_scroll = (total_content_height - container_height).max(0.0);

    // Calculate visible items based on scroll position
    let start_index = (scroll_position() / item_height).floor() as usize;
    let end_index = ((start_index + visible_count).min(items.len())); // Buffer items

    rsx! {
        div {
            width: "800px",
            height: "{container_height}px",
            overflow: "hidden",
            border: "1px solid red",
            background: "#fff",
            position: "relative",
            tabindex: "0",
            padding_bottom: "{item_height / 2.0}px", // To ensure the last item is visible

            onwheel: move |evt| {
                evt.prevent_default();

                let delta_y = match evt.data().delta() {
                    dioxus_elements::geometry::WheelDelta::Pixels(p) => p.y,
                    dioxus_elements::geometry::WheelDelta::Lines(l) => l.y * 20.0,
                    dioxus_elements::geometry::WheelDelta::Pages(p) => p.y * 100.0,
                };

                if !is_scrolling() {
                    is_scrolling.set(true);
                }

                let new_velocity = (velocity() + delta_y * WHEEL_MULTIPLIER)
                    .max(-MAX_VELOCITY)
                    .min(MAX_VELOCITY);
                velocity.set(new_velocity);

                if velocity().abs() >= MIN_VELOCITY {
                    let new_position = scroll_position() + velocity();
                    scroll_position.set(new_position.max(0.0).min(max_scroll));
                    velocity.set(velocity() * FRICTION);
                } else {
                    velocity.set(0.0);
                    is_scrolling.set(false);
                }
            },

            div {
                transform: "translateY({-(scroll_position() % item_height)}px)",

                for i in start_index..end_index {
                    if i < items.len() {
                        div {
                            key: "{i}",
                            class: "list-item",
                            border: "1px solid red",
                            display: "flex",
                            align_items: "center",
                            min_height: "{item_height}px",

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
