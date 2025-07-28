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

    let onwheel = move |evt: WheelEvent| {
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
    };

    let onkeydown = move |evt: KeyboardEvent| {
        let mut velocity_boost = 0.0;
        match evt.key() {
            Key::ArrowDown => {
                evt.prevent_default();
                velocity_boost = 15.0;
            }
            Key::ArrowUp => {
                evt.prevent_default();
                velocity_boost = -15.0;
            }
            Key::PageDown => {
                evt.prevent_default();
                velocity_boost = 40.0;
            }
            Key::PageUp => {
                evt.prevent_default();
                velocity_boost = -40.0;
            }
            Key::Home => {
                evt.prevent_default();
                scroll_position.set(0.0);
                velocity.set(0.0);
                is_scrolling.set(false);
                return;
            }
            Key::End => {
                evt.prevent_default();
                scroll_position.set(max_scroll);
                velocity.set(0.0);
                is_scrolling.set(false);
                return;
            }
            _ => {}
        }

        if velocity_boost != 0.0 {
            if !is_scrolling() {
                is_scrolling.set(true);
            }
            velocity.set(velocity_boost);
            let new_position = scroll_position() + velocity_boost;
            scroll_position.set(new_position.max(0.0).min(max_scroll));
            velocity.set(velocity() * FRICTION);
        }
    };

    let start_index = (scroll_position() / item_height).floor() as usize;
    let end_index = (start_index + visible_count).min(items.len());

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

            onwheel: onwheel,
            onkeydown: onkeydown,

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
