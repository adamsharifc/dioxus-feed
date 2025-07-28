use dioxus::prelude::*;

#[component]
pub fn VirtualList() -> Element {

    const LINES_TO_PIXELS: f64 = 20.0;
    const PAGES_TO_PIXELS: f64 = 100.0;

    let items: Vec<String> = (0..100).map(|i| format!("Item {}", i)).collect();
    let visible_count = 10;
    let mut start_index = use_signal(|| 0);

    rsx! {

        div {
            width: "400px",
            height: "300px",
            overflow: "hidden",
            border: "2px solid #ccc",
            padding: "16px",
            background: "#fafafa",
            onwheel: move |evt| {
                let delta_y = match evt.data().delta() {
                    dioxus_elements::geometry::WheelDelta::Pixels(p) => p.y,
                    dioxus_elements::geometry::WheelDelta::Lines(l) => l.y * LINES_TO_PIXELS,
                    dioxus_elements::geometry::WheelDelta::Pages(p) => p.y * PAGES_TO_PIXELS,
                };
                
                if delta_y > 0.0 {
                    // Scroll down
                    start_index.with_mut(|idx| {
                        *idx = (*idx + 1).min(items.len().saturating_sub(visible_count));
                    });
                } else if delta_y < 0.0 {
                    // Scroll up
                    start_index.with_mut(|idx| {
                        *idx = idx.saturating_sub(1);
                    });
                }

                // Log wheel event data
                println!("Delta: {:?}, Coordinates: {:?}", 
                    evt.data().delta(),
                    evt.data().coordinates()
                );
            },

            // Render visible items
            for i in start_index()..(start_index() + visible_count).min(items.len()) {
                div {
                    key: "{i}",
                    style: "padding: 8px; border-bottom: 1px solid #eee;",
                    "{items[i]}"
                }
            }
        }
    }
}
