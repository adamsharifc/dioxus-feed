use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct FeedProps {
}

#[component]
pub fn Feed(props: FeedProps) -> Element {
    let mut poll_count = use_signal(|| 0u32);
    let polling = use_signal(|| true);
    let older_requested = use_signal(|| false);
    let items = use_signal(|| vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string(), "Item 4".to_string(), "Item 5".to_string()]);
    let scroll_debug = use_signal(|| 0.0f64);

    let mut items_for_poll = items.clone();
    use_future(move || async move {
        loop {
            if polling() {
                poll_count += 1;
                println!("new items published");
                // Append a new item to the bottom
                let mut new_items = items_for_poll().clone();
                let next_num = new_items.len() + 1;
                new_items.push(format!("New Item {}", next_num));
                items_for_poll.set(new_items);
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });

    let onscroll = {
        let mut older_requested = older_requested.clone();
        let mut items = items.clone();
        let mut scroll_debug = scroll_debug.clone();
        move |evt: Event<ScrollData>| {
            let scroll_top = evt.data().scroll_top() as f64;
            let scroll_height = evt.data().scroll_height() as f64;
            let client_height = evt.data().client_height() as f64;
            scroll_debug.set(scroll_top);
            println!("Scroll Debug: scroll_top={scroll_top}, scroll_height={scroll_height}, client_height={client_height}");
            if scroll_top <= 0.0 && *older_requested.read() == false {
                println!("older items requested");
                older_requested.set(true);
                // Simulate loading older items (no overflow)
                let mut new_items = items().clone();
                let mut oldest_num = 1;
                if let Some(first) = new_items.first() {
                    if let Some(num) = first.split_whitespace().last().and_then(|n| n.parse::<usize>().ok()) {
                        oldest_num = num;
                    }
                }
                for i in 1..=5 {
                    new_items.insert(0, format!("Older Item {}", oldest_num.saturating_sub(i)));
                }
                items.set(new_items);
            }
        }
    };

    rsx! {
        div {
            h1 { "Feed Component" }
            div { "ScrollTop: {scroll_debug}" }
            div {
                style: "height: 300px; overflow-y: scroll; border: 1px solid #ccc;",
                onscroll: onscroll,
                for item in items().iter() {
                    div { style: "padding: 16px; border-bottom: 1px solid #eee;", "{item}" }
                }
            }
        }
    }
}