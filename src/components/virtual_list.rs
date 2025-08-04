use dioxus::prelude::*;
use dioxus::html::geometry::PixelsVector2D;

// Feed item data structure for virtual list
#[derive(Clone, PartialEq, Debug)]
pub struct VirtualFeedItem {
    pub id: String,
    pub content: String,
    pub image_url: String,
}

impl VirtualFeedItem {
    pub fn new(id: String, content: String, image_name: &str) -> Self {
        Self {
            id,
            content,
            image_url: format!("myprotocol/assets/images/{}", image_name),
        }
    }
    
    pub fn new_with_random_image(id: String, content: String) -> Self {
        let image = get_random_image_for_id(&id);
        Self::new(id, content, image)
    }
}

// Available images for random selection
const AVAILABLE_IMAGES: &[&str] = &[
    "sample1.svg",
    "sample2.svg", 
    "sample3.svg",
    "sample4.svg",
    "sample5.svg",
    "sample6.svg",
    "sample7.avif",
    "sample8.avif",
    "sample9.avif",
    "sample10.avif",
    "sample11.avif",
    "sample12.avif",
    "sample13.avif"
];

// Random image selector with better uniqueness
fn get_random_image_for_id(id: &str) -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    
    let hash = hasher.finish();
    let index = (hash as usize) % AVAILABLE_IMAGES.len();
    AVAILABLE_IMAGES[index]
}

// Virtual list configuration
const ITEM_HEIGHT: f64 = 320.0; // Height per item including padding
const CONTAINER_HEIGHT: f64 = 600.0; // Viewport height
const BUFFER_SIZE: usize = 5; // Extra items to render outside viewport
const LOAD_THRESHOLD: f64 = 200.0; // Distance from edge to trigger loading
const ITEMS_PER_LOAD: usize = 5; // Items to load at once
const POLLING_INTERVAL_MS: u64 = 5000; // 5 seconds for new items

#[derive(PartialEq, Props, Clone)]
pub struct VirtualListProps {
    pub on_load_more_top: Option<EventHandler<()>>,
    pub on_load_more_bottom: Option<EventHandler<()>>,
}

#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    // Core state
    let mut items = use_signal(|| vec![
        VirtualFeedItem::new_with_random_image("initial_1".to_string(), "Welcome to the feed! This is item 1".to_string()),
        VirtualFeedItem::new_with_random_image("initial_2".to_string(), "Here's another item in your feed".to_string()),
        VirtualFeedItem::new_with_random_image("initial_3".to_string(), "Scroll up or down to load more content".to_string()),
        VirtualFeedItem::new_with_random_image("initial_4".to_string(), "Images load asynchronously via custom protocol".to_string()),
        VirtualFeedItem::new_with_random_image("initial_5".to_string(), "Infinite scrolling in both directions".to_string()),
    ]);
    
    // Scroll tracking
    let mut scroll_top = use_signal(|| 0.0);
    let mut scroll_height = use_signal(|| 0.0);
    let mut client_height = use_signal(|| CONTAINER_HEIGHT);
    let mut last_scroll_top = use_signal(|| 0.0);
    let mut scroll_direction = use_signal(|| 0i8); // -1 = up, 0 = none, 1 = down
    
    // Loading states
    let mut is_loading_top = use_signal(|| false);
    let mut is_loading_bottom = use_signal(|| false);
    
    // Scroll element reference
    let mut scroll_element = use_signal(|| None::<std::rc::Rc<MountedData>>);
    
    // Calculate virtual list parameters
    let total_items = items().len();
    let total_height = total_items as f64 * ITEM_HEIGHT;
    let visible_count = (client_height() / ITEM_HEIGHT).ceil() as usize;
    
    // Calculate visible range with buffer
    let start_index = ((scroll_top() / ITEM_HEIGHT) as usize).saturating_sub(BUFFER_SIZE);
    let end_index = (start_index + visible_count + (BUFFER_SIZE * 2)).min(total_items);
    
    // Load more items at top
    let load_more_top = use_callback(move |_| {
        if is_loading_top() {
            return;
        }
        
        is_loading_top.set(true);
        
        spawn(async move {
            // Simulate loading delay
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
            
            let mut current_items = items();
            let mut new_items = Vec::new();
            
            // Add items at the beginning
            for i in 1..=ITEMS_PER_LOAD {
                let item_id = format!("older_{}_{}", current_items.len() + i, chrono::Utc::now().timestamp_millis());
                let content = format!("Older content item {} - loaded from top", current_items.len() + i);
                new_items.push(VirtualFeedItem::new_with_random_image(item_id, content));
            }
            
            // Prepend new items
            new_items.extend(current_items);
            
            // Preserve scroll position by adjusting scroll_top
            let added_height = ITEMS_PER_LOAD as f64 * ITEM_HEIGHT;
            if let Some(element) = scroll_element() {
                let new_scroll_top = scroll_top() + added_height;
                let _ = spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    let _ = element.scroll(
                        PixelsVector2D::new(0.0, new_scroll_top),
                        ScrollBehavior::Instant
                    ).await;
                });
                scroll_top.set(new_scroll_top);
            }
            
            items.set(new_items);
            is_loading_top.set(false);
        });
    });
    
    // Load more items at bottom
    let load_more_bottom = use_callback(move |_| {
        if is_loading_bottom() {
            return;
        }
        
        is_loading_bottom.set(true);
        
        spawn(async move {
            // Simulate loading delay
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            
            let mut current_items = items();
            
            // Add items at the end
            for i in 1..=ITEMS_PER_LOAD {
                let item_id = format!("newer_{}_{}", current_items.len() + i, chrono::Utc::now().timestamp_millis());
                let content = format!("Newer content item {} - loaded from bottom", current_items.len() + i);
                current_items.push(VirtualFeedItem::new_with_random_image(item_id, content));
            }
            
            items.set(current_items);
            is_loading_bottom.set(false);
        });
    });
    
    // Auto-polling for new content
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(POLLING_INTERVAL_MS)).await;
            
            let mut current_items = items();
            let next_num = current_items.len() + 1;
            let item_id = format!("auto_{}_{}", next_num, chrono::Utc::now().timestamp_millis());
            let content = format!("Auto-generated item {} - real-time update", next_num);
            current_items.push(VirtualFeedItem::new_with_random_image(item_id, content));
            
            items.set(current_items);
        }
    });
    
    // Handle scroll events
    let handle_scroll = move |evt: Event<ScrollData>| {
        let current_scroll_top = evt.data().scroll_top() as f64;
        let current_scroll_height = evt.data().scroll_height() as f64;
        let current_client_height = evt.data().client_height() as f64;
        
        // Determine scroll direction
        let previous_scroll = last_scroll_top();
        let direction = if current_scroll_top > previous_scroll {
            1i8 // scrolling down
        } else if current_scroll_top < previous_scroll {
            -1i8 // scrolling up
        } else {
            0i8 // no change
        };
        
        // Update state
        scroll_top.set(current_scroll_top);
        scroll_height.set(current_scroll_height);
        client_height.set(current_client_height);
        last_scroll_top.set(current_scroll_top);
        scroll_direction.set(direction);
        
        // Check if we need to load more items at top (only when scrolling UP)
        if current_scroll_top <= LOAD_THRESHOLD && direction == -1 && !is_loading_top() {
            load_more_top.call(());
        }
        
        // Check if we need to load more items at bottom (only when scrolling DOWN)
        let distance_from_bottom = current_scroll_height - current_scroll_top - current_client_height;
        if distance_from_bottom <= LOAD_THRESHOLD && direction == 1 && !is_loading_bottom() {
            load_more_bottom.call(());
        }
    };

    rsx! {
        div {
            style: format!("
                height: {}px;
                overflow-y: auto;
                background: white;
                position: relative;
                scroll-behavior: smooth;
            ", CONTAINER_HEIGHT),
            
            onscroll: handle_scroll,
            onmounted: move |event| scroll_element.set(Some(event.data())),
            
            // Loading indicator at top
            if is_loading_top() {
                div {
                    style: "
                        position: sticky;
                        top: 0;
                        z-index: 100;
                        background: white;
                        color: #0f172a;
                        text-align: center;
                        padding: 15px;
                        border-bottom: 1px solid #e2e8f0;
                        font-weight: 500;
                    ",
                    "Loading older items..."
                }
            }
            
            // Virtual content container
            div {
                style: format!("height: {}px; position: relative;", total_height),
                
                // Render only visible items
                for i in start_index..end_index {
                    if i < items().len() {
                        VirtualFeedItemComponent {
                            key: "{items()[i].id}",
                            item: items()[i].clone(),
                            top_position: i as f64 * ITEM_HEIGHT,
                        }
                    }
                }
            }
            
            // Loading indicator at bottom
            if is_loading_bottom() {
                div {
                    style: "
                        position: sticky;
                        bottom: 0;
                        z-index: 100;
                        background: white;
                        color: #0f172a;
                        text-align: center;
                        padding: 15px;
                        border-top: 1px solid #e2e8f0;
                        font-weight: 500;
                    ",
                    "Loading newer items..."
                }
            }
            
            // Debug info (hidden by default)
            div {
                style: "
                    position: fixed;
                    top: 10px;
                    right: 10px;
                    background: rgba(0, 0, 0, 0.8);
                    color: white;
                    padding: 10px;
                    border-radius: 5px;
                    font-size: 12px;
                    font-family: monospace;
                    z-index: 1000;
                    display: none;
                ",
                div { "Items: {total_items}" }
                div { "Visible: {start_index}-{end_index}" }
                div { "Scroll: {scroll_top:.0}px" }
                div { "Height: {total_height:.0}px" }
                div { 
                    if scroll_direction() == -1 { "Direction: UP" }
                    else if scroll_direction() == 1 { "Direction: DOWN" }
                    else { "Direction: NONE" }
                }
                div { "Loading T:{is_loading_top()} B:{is_loading_bottom()}" }
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct VirtualFeedItemProps {
    pub item: VirtualFeedItem,
    pub top_position: f64,
}

#[component]
pub fn VirtualFeedItemComponent(props: VirtualFeedItemProps) -> Element {
    let item = &props.item;
    let top_position = props.top_position;
    
    // Image loading state
    let mut image_loaded = use_signal(|| false);
    let mut image_error = use_signal(|| false);
    
    rsx! {
        article {
            style: format!("
                position: absolute;
                top: {}px;
                width: 100%;
                height: {}px;
                background: white;
                border-radius: 8px;
                border: 1px solid #e2e8f0;
                margin-bottom: 16px;
                padding: 20px;
                box-sizing: border-box;
                display: flex;
                flex-direction: column;
                transition: border-color 0.2s ease;
            ", top_position, ITEM_HEIGHT - 16.0),
            
            onmouseenter: |_| {
                // Add hover effect via CSS-in-JS
            },
            
            // Header with timestamp
            header {
                style: "
                    display: flex;
                    align-items: center;
                    margin-bottom: 12px;
                    padding-bottom: 8px;
                    border-bottom: 1px solid #e2e8f0;
                ",
                h3 {
                    style: "
                        margin: 0;
                        font-size: 16px;
                        font-weight: 600;
                        color: #0f172a;
                    ",
                    "Item {item.id}"
                }
            }
            
            // Main content area
            div {
                style: "
                    display: flex;
                    gap: 16px;
                    flex: 1;
                    align-items: flex-start;
                ",
                
                // Image container
                div {
                    style: "
                        flex-shrink: 0;
                        width: 120px;
                        height: 120px;
                        border-radius: 6px;
                        overflow: hidden;
                        background: #f8fafc;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        border: 1px solid #e2e8f0;
                        position: relative;
                    ",
                    
                    if !image_loaded() && !image_error() {
                        div {
                            style: "
                                color: #64748b;
                                font-size: 12px;
                                text-align: center;
                                padding: 10px;
                            ",
                            "Loading..."
                        }
                    }
                    
                    if image_error() {
                        div {
                            style: "
                                color: #ef4444;
                                font-size: 12px;
                                text-align: center;
                                padding: 10px;
                            ",
                            "Failed to load"
                        }
                    }
                    
                    img {
                        src: "{item.image_url}",
                        alt: "Feed item image",
                        style: format!("
                            width: 100%;
                            height: 100%;
                            object-fit: cover;
                            display: {};
                        ", if image_loaded() { "block" } else { "none" }),
                        
                        onload: move |_| {
                            image_loaded.set(true);
                            image_error.set(false);
                        },
                        
                        onerror: move |_| {
                            image_error.set(true);
                            image_loaded.set(false);
                        },
                    }
                }
                
                // Text content
                div {
                    style: "
                        flex: 1;
                        display: flex;
                        flex-direction: column;
                        gap: 8px;
                    ",
                    
                    p {
                        style: "
                            margin: 0;
                            font-size: 14px;
                            line-height: 1.5;
                            color: #475569;
                        ",
                        "{item.content}"
                    }
                }
            }
        }
    }
}
