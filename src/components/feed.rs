use dioxus::prelude::*;
use dioxus::html::geometry::PixelsVector2D;

#[derive(PartialEq, Props, Clone)]
pub struct FeedProps {
}

// Scroll logic hook
fn use_scroll_management(
    items: Signal<Vec<String>>,
    is_loading_top: Signal<bool>,
    is_loading_bottom: Signal<bool>,
    scroll_lock: Signal<bool>,
    locked_scroll_position: Signal<f64>,
    scroll_element: Signal<Option<std::rc::Rc<MountedData>>>
) -> impl Fn(Event<ScrollData>) {
    move |evt: Event<ScrollData>| {
        let scroll_top = evt.data().scroll_top() as f64;
        let scroll_height = evt.data().scroll_height() as f64;
        let client_height = evt.data().client_height() as f64;
        
        // Handle scroll lock enforcement
        if scroll_lock() {
            handle_scroll_lock(scroll_top, locked_scroll_position(), scroll_element());
            return;
        }
        
        println!("Scroll detected: top={scroll_top}, height={scroll_height}, client={client_height}");
        
        // Handle top scroll trigger
        if scroll_top <= 0.0 && !is_loading_top() {
            handle_top_scroll_trigger(
                items, 
                is_loading_top, 
                scroll_lock, 
                locked_scroll_position, 
                scroll_element(),
                scroll_top
            );
        }
        
        // Handle bottom scroll trigger
        if scroll_height - scroll_top - client_height < 200.0 && !is_loading_bottom() {
            handle_bottom_scroll_trigger(items, is_loading_bottom);
        }
    }
}

// Handle scroll lock enforcement
fn handle_scroll_lock(
    current_scroll_top: f64,
    locked_position: f64,
    scroll_element: Option<std::rc::Rc<MountedData>>
) {
    if (current_scroll_top - locked_position).abs() > 1.0 {
        println!("Scroll locked! Forcing position back to {locked_position} (was at {current_scroll_top})");
        if let Some(element) = scroll_element {
            let _ = spawn(async move {
                let _ = element.scroll(
                    PixelsVector2D::new(0.0, locked_position),
                    ScrollBehavior::Instant
                ).await;
            });
        }
    }
}

// Handle top scroll loading logic
fn handle_top_scroll_trigger(
    mut items: Signal<Vec<String>>,
    mut is_loading_top: Signal<bool>,
    mut scroll_lock: Signal<bool>,
    mut locked_scroll_position: Signal<f64>,
    scroll_element: Option<std::rc::Rc<MountedData>>,
    original_scroll_position: f64,
) {
    println!("User reached absolute top (<=0) - loading older items...");
    is_loading_top.set(true);
    locked_scroll_position.set(original_scroll_position);
    scroll_lock.set(true);
    
    spawn(async move {
        println!("Starting load process from scroll position: {original_scroll_position}");
        println!("Scroll position LOCKED at: {}", locked_scroll_position());
        
        // Add new items
        let mut new_items = items().clone();
        for i in 1..=3 {
            new_items.insert(0, format!("Older Item {}", new_items.len() + i));
        }
        items.set(new_items);
        
        // Wait for DOM updates
        wait_for_dom_updates().await;
        
        // Restore scroll position
        if let Some(element) = scroll_element {
            restore_scroll_position(element, locked_scroll_position).await;
        }
        
        println!("Preparing to release scroll lock after extended period...");
    });
}

// Handle bottom scroll loading logic
fn handle_bottom_scroll_trigger(
    mut items: Signal<Vec<String>>,
    mut is_loading_bottom: Signal<bool>,
) {
    println!("User near bottom - loading newer items automatically...");
    is_loading_bottom.set(true);
    
    // Add newer items to the end
    let mut new_items = items().clone();
    for i in 1..=3 {
        new_items.push(format!("Bottom Item {}", new_items.len() + i));
    }
    items.set(new_items);
}

// DOM update waiting logic
async fn wait_for_dom_updates() {
    println!("Waiting for DOM updates...");
    
    // Initial short wait for immediate DOM changes
    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    
    // Secondary wait for layout calculations
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    
    // Final wait for complete rendering
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

// Scroll position restoration logic
async fn restore_scroll_position(
    element: std::rc::Rc<MountedData>,
    mut locked_scroll_position: Signal<f64>,
) {
    let estimated_item_height = 110.0; // min-height: 80px + margins + padding
    let added_items = 3.0;
    let calculated_offset = added_items * estimated_item_height;
    
    let target_position = if calculated_offset < 50.0 {
        println!("Calculated offset too small ({calculated_offset}), using minimum of 50px");
        50.0
    } else {
        calculated_offset
    };
    
    println!("Will restore scroll position to: {target_position}px after unlock");
    locked_scroll_position.set(target_position);
    
    // Attempt scroll restoration with retries
    let mut scroll_success = false;
    for attempt in 1..=3 {
        let scroll_result = element.scroll(
            PixelsVector2D::new(0.0, target_position), 
            ScrollBehavior::Instant
        ).await;
        
        if scroll_result.is_ok() {
            println!("Target scroll position set successfully on attempt {attempt}");
            scroll_success = true;
            break;
        } else {
            println!("Scroll attempt {attempt} failed, retrying...");
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    
    if !scroll_success {
        println!("All scroll attempts failed, position may reset to 0");
    }
    
    // Extended stabilization wait
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
}

// Loading state management hook
fn use_loading_state_management(
    is_loading_top: Signal<bool>,
    is_loading_bottom: Signal<bool>,
    scroll_lock: Signal<bool>,
) {
    let mut reset_loading_top = is_loading_top.clone();
    let mut reset_loading_bottom = is_loading_bottom.clone();
    let mut reset_scroll_lock = scroll_lock.clone();
    
    use_future(move || async move {
        loop {
            if reset_loading_top() {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                reset_loading_top.set(false);
                reset_scroll_lock.set(false);
                println!("Top loading completed after extended lock, scroll lock released");
            }
            if reset_loading_bottom() {
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                reset_loading_bottom.set(false);
                println!("Bottom loading completed");
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
}

// Real-time polling hook
fn use_real_time_polling(items: Signal<Vec<String>>) {
    let mut items_for_poll = items.clone();
    use_future(move || async move {
        loop {
            println!("Polling: Adding new item to feed...");
            let mut new_items = items_for_poll().clone();
            let next_num = new_items.len() + 1;
            new_items.push(format!("New Item {}", next_num));
            items_for_poll.set(new_items);
            
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });
}

#[component]
pub fn Feed(props: FeedProps) -> Element {
    // Core state management
    let items = use_signal(|| vec![
        "Item 1".to_string(), 
        "Item 2".to_string(), 
        "Item 3".to_string(), 
        "Item 4".to_string(), 
        "Item 5".to_string()
    ]);
    let is_loading_top = use_signal(|| false);
    let is_loading_bottom = use_signal(|| false);
    let scroll_debug = use_signal(|| 0.0f64);
    let scroll_lock = use_signal(|| false);
    let last_scroll_height = use_signal(|| 0.0f64);
    let locked_scroll_position = use_signal(|| 0.0f64);
    let mut scroll_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    // Initialize hooks
    use_real_time_polling(items);
    use_loading_state_management(is_loading_top, is_loading_bottom, scroll_lock);
    
    // Create scroll handler
    let handle_scroll = use_scroll_management(
        items,
        is_loading_top,
        is_loading_bottom,
        scroll_lock,
        locked_scroll_position,
        scroll_element,
    );
    
    // Update debug info in scroll handler
    let mut scroll_debug_handler = scroll_debug.clone();
    let mut last_scroll_height_handler = last_scroll_height.clone();
    let enhanced_handle_scroll = move |evt: Event<ScrollData>| {
        scroll_debug_handler.set(evt.data().scroll_top() as f64);
        last_scroll_height_handler.set(evt.data().scroll_height() as f64);
        handle_scroll(evt);
    };

    rsx! {
        // Scrollable container - this is where the magic happens (like React's window scroll)
        div {
            style: format!("
                height: 98vh;
                overflow-y: {};
                background: #f5f5f5;
                padding: 0;
                margin: 0;
                scroll-behavior: {};
            ", 
                if scroll_lock() { "hidden" } else { "auto" },
                if scroll_lock() { "none" } else { "smooth" }
            ),
            onscroll: enhanced_handle_scroll,
            onmounted: move |event| scroll_element.set(Some(event.data())),
            
            // Sticky header for debugging info  
            div {
                style: "
                    position: sticky;
                    top: 0;
                    background: rgba(255, 255, 255, 0.95);
                    backdrop-filter: blur(10px);
                    padding: 10px 20px;
                    border-bottom: 1px solid #eee;
                    z-index: 100;
                    font-size: 14px;
                    color: #666;
                    display: none;
                ",
                div { "ScrollTop: {scroll_debug}" }
                div { "Items count: {items().len()}" }
                div { "Scroll Height: {last_scroll_height}" }
                div { "Locked Position: {locked_scroll_position}" }
                div { 
                    style: if scroll_lock() { "color: #ff6b6b; font-weight: bold;" } else { "color: #51cf66;" },
                    if scroll_lock() { "SCROLL PHYSICALLY LOCKED!" } else { "Scroll Active" }
                }
                div { "Feed Component - True Scroll Locking" }
                div { 
                    style: "font-size: 12px; color: #999; margin-top: 5px;",
                    "Scroll to top - container will be locked during load!" 
                }
            }
            
            // Main content area
            div {
                style: "max-width: 600px; margin: 0 auto; background: white; padding: 20px;",
                
                // Top loading indicator - sticky at top of content
                if is_loading_top() {
                    div {
                        style: "
                            position: sticky;
                            top: 80px;
                            text-align: center;
                            padding: 20px;
                            color: #666;
                            background: rgba(255, 255, 255, 0.9);
                            backdrop-filter: blur(5px);
                            border-radius: 8px;
                            margin-bottom: 20px;
                            z-index: 50;
                            border: 2px solid #ff6b6b;
                        ",
                        div { "Loading older posts..." }
                        div { 
                            style: "font-size: 12px; color: #999; margin-top: 5px;",
                            "Scroll position locked during load"
                        }
                    }
                }
                
                // Feed items container
                div {
                    class: "feed",
                    for item in items().iter() {
                        div { 
                            style: "
                                padding: 20px;
                                border-bottom: 1px solid #eee;
                                border: 1px solid #ddd;
                                margin: 15px 0;
                                background: white;
                                border-radius: 8px;
                                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                                min-height: 80px;
                                display: flex;
                                align-items: center;
                                font-size: 16px;
                            ", 
                            "{item}" 
                        }
                    }
                }
                
                // Bottom loading indicator - sticky at bottom
                if is_loading_bottom() {
                    div {
                        style: "
                            position: sticky;
                            bottom: 20px;
                            text-align: center;
                            padding: 20px;
                            color: #666;
                            background: rgba(255, 255, 255, 0.9);
                            backdrop-filter: blur(5px);
                            border-radius: 8px;
                            margin-top: 20px;
                            z-index: 50;
                        ",
                        "Loading newer posts..."
                    }
                }
                
                // Bottom spacer to ensure scroll space
                div {
                    style: "height: 200px; background: transparent;",
                }
            }
        }
    }
}
