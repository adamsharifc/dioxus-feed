use dioxus::prelude::*;
use dioxus::html::geometry::PixelsVector2D;

#[derive(PartialEq, Props, Clone)]
pub struct FeedProps {
}

#[component]
pub fn Feed(props: FeedProps) -> Element {
    // Core state management
    let mut items = use_signal(|| vec![
        "Item 1".to_string(), 
        "Item 2".to_string(), 
        "Item 3".to_string(), 
        "Item 4".to_string(), 
        "Item 5".to_string()
    ]);
    let mut is_loading_top = use_signal(|| false);
    let mut is_loading_bottom = use_signal(|| false);
    let mut scroll_debug = use_signal(|| 0.0f64);
    let mut scroll_lock = use_signal(|| false);
    let mut last_scroll_height = use_signal(|| 0.0f64);
    let mut locked_scroll_position = use_signal(|| 0.0f64);
    
    // Mount reference for scroll control
    let mut scroll_element = use_signal(|| None::<std::rc::Rc<MountedData>>);

    // Real-time polling for new items (like React version)
    let mut items_for_poll = items.clone();
    use_future(move || async move {
        loop {
            println!("📡 Polling: Adding new item to feed...");
            let mut new_items = items_for_poll().clone();
            let next_num = new_items.len() + 1;
            new_items.push(format!("New Item {}", next_num));
            items_for_poll.set(new_items);
            
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    });

    // Async delay for resetting loading states
    let mut reset_loading_top = is_loading_top.clone();
    let mut reset_loading_bottom = is_loading_bottom.clone();
    let mut reset_scroll_lock = scroll_lock.clone();
    
    use_future(move || async move {
        loop {
            if reset_loading_top() {
                // Extended duration for better scroll lock visibility
                tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                reset_loading_top.set(false);
                reset_scroll_lock.set(false);
                println!("✅ Top loading completed after extended lock, scroll lock released");
            }
            if reset_loading_bottom() {
                tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                reset_loading_bottom.set(false);
                println!("✅ Bottom loading completed");
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // Scroll handler - equivalent to React's handleScroll function
    let handle_scroll = {
        let mut items_handler = items.clone();
        let mut is_loading_top_handler = is_loading_top.clone();
        let mut is_loading_bottom_handler = is_loading_bottom.clone();
        let mut scroll_debug_handler = scroll_debug.clone();
        let mut scroll_lock_handler = scroll_lock.clone();
        let mut last_scroll_height_handler = last_scroll_height.clone();
        let mut locked_position_handler = locked_scroll_position.clone();
        let scroll_element_ref = scroll_element.clone();
        
        move |evt: Event<ScrollData>| {
            let scroll_top = evt.data().scroll_top() as f64;
            let scroll_height = evt.data().scroll_height() as f64;
            let client_height = evt.data().client_height() as f64;
            
            scroll_debug_handler.set(scroll_top);
            last_scroll_height_handler.set(scroll_height);
            
            // If scroll is locked, force position back to locked position
            if scroll_lock_handler() {
                let locked_pos = locked_position_handler();
                if (scroll_top - locked_pos).abs() > 1.0 {
                    println!("Scroll locked! Forcing position back to {locked_pos} (was at {scroll_top})");
                    if let Some(element) = scroll_element_ref() {
                        let _ = spawn(async move {
                            let _ = element.scroll(
                                PixelsVector2D::new(0.0, locked_pos),
                                ScrollBehavior::Instant
                            ).await;
                        });
                    }
                }
                return;
            }
            
            println!("Scroll detected: top={scroll_top}, height={scroll_height}, client={client_height}");
            
            // TRIGGER 1: User scrolled to the absolute top (EXACT MATCH to vanilla JS)
            if scroll_top <= 0.0 && !is_loading_top_handler() {
                println!("User reached absolute top (<=0) - loading older items...");
                is_loading_top_handler.set(true);
                
                // Lock scroll at current position
                locked_position_handler.set(scroll_top);
                scroll_lock_handler.set(true);
                
                let scroll_el = scroll_element_ref.clone();
                let mut items_clone = items_handler.clone();
                let mut scroll_lock_clone = scroll_lock_handler.clone();
                let mut locked_pos_clone = locked_position_handler.clone();
                let original_scroll_position = scroll_top;
                
                spawn(async move {
                    println!("Starting load process from scroll position: {original_scroll_position}");
                    println!("Scroll position LOCKED at: {}", locked_pos_clone());
                    
                    // STEP 1: Add items to the list
                    let mut new_items = items_clone().clone();
                    for i in 1..=3 {
                        new_items.insert(0, format!("Older Item {}", new_items.len() + i));
                    }
                    items_clone.set(new_items);
                    
                    // STEP 2: Robust DOM update waiting with multiple timing checks
                    println!("Waiting for DOM updates...");
                    
                    // Initial short wait for immediate DOM changes
                    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                    
                    // Secondary wait for layout calculations
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    
                    // Final wait for complete rendering
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    
                    // STEP 3: Calculate new scroll position while locked
                    if let Some(element) = scroll_el() {
                        // Each item has calculated height based on actual styling
                        let estimated_item_height = 110.0; // min-height: 80px + margins + padding
                        let added_items = 3.0;
                        let calculated_offset = added_items * estimated_item_height;
                        
                        // Ensure we don't scroll to position 0 accidentally
                        let target_position = if calculated_offset < 50.0 {
                            println!("Calculated offset too small ({calculated_offset}), using minimum of 50px");
                            50.0
                        } else {
                            calculated_offset
                        };
                        
                        println!("🔧 Will restore scroll position to: {target_position}px after unlock");
                        
                        // Update locked position to target position for smooth unlock
                        locked_pos_clone.set(target_position);
                        
                        // Set the actual scroll position (this will be the unlock position)
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
                        
                        // Extended stabilization wait to verify position holds
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                    
                    println!("Preparing to release scroll lock after extended period...");
                    
                    // Note: scroll lock will be released by the separate use_future above after 2 seconds
                    // This ensures we can observe the lock in action
                });
            }
            
            // TRIGGER 2: User scrolled near the bottom (load newer items) - AUTOMATIC!
            if scroll_height - scroll_top - client_height < 200.0 && !is_loading_bottom_handler() {
                println!("User near bottom - loading newer items automatically...");
                is_loading_bottom_handler.set(true);
                
                // Add newer items to the end (like React version)
                let mut new_items = items_handler().clone();
                for i in 1..=3 {
                    new_items.push(format!("Bottom Item {}", new_items.len() + i));
                }
                items_handler.set(new_items);
            }
        }
    };

    rsx! {
        // Scrollable container - this is where the magic happens (like React's window scroll)
        div {
            style: format!("
                height: 100vh;
                overflow-y: {};
                background: #f5f5f5;
                padding: 0;
                margin: 0;
                scroll-behavior: {};
            ", 
                if scroll_lock() { "hidden" } else { "auto" },
                if scroll_lock() { "none" } else { "smooth" }
            ),
            onscroll: handle_scroll,
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
                ",
                div { "ScrollTop: {scroll_debug}" }
                div { "Items count: {items().len()}" }
                div { "Scroll Height: {last_scroll_height}" }
                div { "Locked Position: {locked_scroll_position}" }
                div { 
                    style: if scroll_lock() { "color: #ff6b6b; font-weight: bold;" } else { "color: #51cf66;" },
                    if scroll_lock() { "SCROLL PHYSICALLY LOCKED!" } else { "Scroll Active" }
                }
                div { "🔍 Feed Component - True Scroll Locking" }
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
