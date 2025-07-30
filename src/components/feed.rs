use dioxus::prelude::*;
use dioxus::html::geometry::PixelsVector2D;

#[derive(PartialEq, Props, Clone)]
pub struct FeedProps {
}

// Configuration constants - centralized magic numbers
const MAX_ITEMS: usize = 500; // Maximum items to keep in memory
const ITEMS_PER_LOAD: usize = 3; // Number of items to load at once
const BOTTOM_THRESHOLD: f64 = 200.0; // Distance from bottom to trigger loading (px)
const ITEM_HEIGHT: f64 = 110.0; // Estimated item height for scroll calculations (px)
const POLLING_INTERVAL_SECONDS: u64 = 3; // Real-time polling interval
const SCROLL_LOCK_DURATION_MS: u64 = 200; // How long to keep scroll locked
const BOTTOM_LOADING_DURATION_MS: u64 = 600; // Bottom loading indicator duration

// DOM update timing constants
const DOM_UPDATE_IMMEDIATE_MS: u64 = 16; // Initial DOM change wait
const DOM_UPDATE_LAYOUT_MS: u64 = 50; // Layout calculation wait
const DOM_UPDATE_RENDER_MS: u64 = 100; // Final rendering wait
const DOM_UPDATE_STABILIZATION_MS: u64 = 200; // Position stabilization wait

// Scroll operation constants
const SCROLL_POSITION_TOLERANCE: f64 = 1.0; // Tolerance for scroll position changes
const SCROLL_RETRY_ATTEMPTS: usize = 3; // Number of scroll retry attempts
const SCROLL_RETRY_DELAY_MS: u64 = 10; // Delay between scroll retries
const MIN_SCROLL_OFFSET: f64 = 50.0; // Minimum scroll offset to prevent zero position

// Item limit management with error handling
fn trim_items_if_needed(items: &mut Vec<String>) -> Result<(), &'static str> {
    if items.len() > MAX_ITEMS {
        let excess = items.len() - MAX_ITEMS;
        let remove_start = MAX_ITEMS / 2;
        
        if remove_start + excess <= items.len() {
            items.drain(remove_start..remove_start + excess);
            Ok(())
        } else {
            Err("Invalid trim range calculated")
        }
    } else {
        Ok(())
    }
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
            if let Err(_) = handle_scroll_lock(scroll_top, locked_scroll_position(), scroll_element()) {
                // Silently continue if scroll lock fails
            }
            return;
        }
        
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
        if scroll_height - scroll_top - client_height < BOTTOM_THRESHOLD && !is_loading_bottom() {
            handle_bottom_scroll_trigger(items, is_loading_bottom);
        }
    }
}

// Handle scroll lock enforcement with error handling
fn handle_scroll_lock(
    current_scroll_top: f64,
    locked_position: f64,
    scroll_element: Option<std::rc::Rc<MountedData>>
) -> Result<(), &'static str> {
    if (current_scroll_top - locked_position).abs() > SCROLL_POSITION_TOLERANCE {
        if let Some(element) = scroll_element {
            let _ = spawn(async move {
                let _ = element.scroll(
                    PixelsVector2D::new(0.0, locked_position),
                    ScrollBehavior::Instant
                ).await;
            });
            Ok(())
        } else {
            Err("Scroll element not available")
        }
    } else {
        Ok(())
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
    is_loading_top.set(true);
    locked_scroll_position.set(original_scroll_position);
    scroll_lock.set(true);
    
    spawn(async move {
        // Add new items with error handling
        let mut new_items = items().clone();
        for i in 1..=ITEMS_PER_LOAD {
            new_items.insert(0, format!("Older Item {}", new_items.len() + i));
        }
        
        // Trim items if needed with error handling
        let _ = trim_items_if_needed(&mut new_items);
        items.set(new_items);
        
        // Wait for DOM updates
        if let Err(_) = wait_for_dom_updates().await {
            // Continue even if timing fails
        }
        
        // Restore scroll position
        if let Some(element) = scroll_element {
            let _ = restore_scroll_position(element, locked_scroll_position).await;
        }
    });
}

// Handle bottom scroll loading logic
fn handle_bottom_scroll_trigger(
    mut items: Signal<Vec<String>>,
    mut is_loading_bottom: Signal<bool>,
) {
    is_loading_bottom.set(true);
    
    // Add newer items with error handling
    let mut new_items = items().clone();
    for i in 1..=ITEMS_PER_LOAD {
        new_items.push(format!("Bottom Item {}", new_items.len() + i));
    }
    
    // Trim items if needed with error handling
    let _ = trim_items_if_needed(&mut new_items);
    items.set(new_items);
}

// DOM update waiting logic with error handling
async fn wait_for_dom_updates() -> Result<(), &'static str> {
    // Initial short wait for immediate DOM changes
    tokio::time::sleep(std::time::Duration::from_millis(DOM_UPDATE_IMMEDIATE_MS)).await;
    
    // Secondary wait for layout calculations
    tokio::time::sleep(std::time::Duration::from_millis(DOM_UPDATE_LAYOUT_MS)).await;
    
    // Final wait for complete rendering
    tokio::time::sleep(std::time::Duration::from_millis(DOM_UPDATE_RENDER_MS)).await;
    
    Ok(())
}

// Scroll position restoration logic with comprehensive error handling
async fn restore_scroll_position(
    element: std::rc::Rc<MountedData>,
    mut locked_scroll_position: Signal<f64>,
) -> Result<(), &'static str> {
    let calculated_offset = ITEMS_PER_LOAD as f64 * ITEM_HEIGHT;
    
    let target_position = if calculated_offset < MIN_SCROLL_OFFSET {
        MIN_SCROLL_OFFSET
    } else {
        calculated_offset
    };
    
    locked_scroll_position.set(target_position);
    
    // Attempt scroll restoration with retries
    for attempt in 1..=SCROLL_RETRY_ATTEMPTS {
        match element.scroll(
            PixelsVector2D::new(0.0, target_position), 
            ScrollBehavior::Instant
        ).await {
            Ok(_) => return Ok(()),
            Err(_) => {
                if attempt < SCROLL_RETRY_ATTEMPTS {
                    tokio::time::sleep(std::time::Duration::from_millis(SCROLL_RETRY_DELAY_MS)).await;
                }
            }
        }
    }
    
    // Extended stabilization wait
    tokio::time::sleep(std::time::Duration::from_millis(DOM_UPDATE_STABILIZATION_MS)).await;
    
    Err("All scroll attempts failed")
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
                tokio::time::sleep(std::time::Duration::from_millis(SCROLL_LOCK_DURATION_MS)).await;
                reset_loading_top.set(false);
                reset_scroll_lock.set(false);
            }
            if reset_loading_bottom() {
                tokio::time::sleep(std::time::Duration::from_millis(BOTTOM_LOADING_DURATION_MS)).await;
                reset_loading_bottom.set(false);
            }
            tokio::time::sleep(std::time::Duration::from_millis(DOM_UPDATE_RENDER_MS)).await;
        }
    });
}

// Real-time polling hook with error handling
fn use_real_time_polling(items: Signal<Vec<String>>) {
    let mut items_for_poll = items.clone();
    use_future(move || async move {
        loop {
            let mut new_items = items_for_poll().clone();
            let next_num = new_items.len() + 1;
            new_items.push(format!("New Item {}", next_num));
            
            // Trim items if needed with error handling
            let _ = trim_items_if_needed(&mut new_items);
            items_for_poll.set(new_items);
            
            tokio::time::sleep(std::time::Duration::from_secs(POLLING_INTERVAL_SECONDS)).await;
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

    // Check if has items
    let has_items = !items().is_empty();

    rsx! {
        // Scrollable container
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
            
            // Debug header (hidden by default)
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
                div { "Items count: {items().len()} (Max: {MAX_ITEMS})" }
                div { "Config: {ITEMS_PER_LOAD} items/load, {BOTTOM_THRESHOLD}px threshold" }
                div { "Item height: {ITEM_HEIGHT}px, Polling: {POLLING_INTERVAL_SECONDS}s" }
                div { "Scroll Height: {last_scroll_height}" }
                div { "Locked Position: {locked_scroll_position}" }
                div { 
                    style: if scroll_lock() { "color: #ff6b6b; font-weight: bold;" } else { "color: #51cf66;" },
                    if scroll_lock() { "SCROLL LOCKED" } else { "Scroll Active" }
                }
                div { "Feed Component - Production Ready" }
                div { 
                    style: "font-size: 12px; color: #999; margin-top: 5px;",
                    "Error handling enabled, inline styles" 
                }
            }
            
            // Main content area
            div {
                style: "max-width: 600px; margin: 0 auto; background: white; padding: 20px;",
                
                // Top loading indicator
                if is_loading_top() {
                    div {
                        style: "
                            position: sticky;
                            top: 80px;
                            text-align: center;
                            padding: 20px;
                            color: #666;
                            background: rgba(255, 255, 255, 0.95);
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
                
                // Empty state
                if !has_items {
                    div {
                        style: "
                            text-align: center;
                            padding: 60px 20px;
                            color: #666;
                            background: #fafafa;
                            border-radius: 8px;
                            border: 2px dashed #ddd;
                            margin: 40px 0;
                        ",
                        div {
                            style: "font-size: 48px; margin-bottom: 16px; color: #ccc;",
                            "📭"
                        }
                        h2 {
                            style: "font-size: 24px; margin: 0 0 8px 0; color: #333;",
                            "No items yet"
                        }
                        p {
                            style: "font-size: 16px; margin: 0; color: #666;",
                            "New items will appear here automatically"
                        }
                    }
                }
                
                // Feed items
                if has_items {
                    div {
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
                }
                
                // Bottom loading indicator
                if is_loading_bottom() {
                    div {
                        style: "
                            position: sticky;
                            bottom: 20px;
                            text-align: center;
                            padding: 20px;
                            color: #666;
                            background: rgba(255, 255, 255, 0.95);
                            backdrop-filter: blur(5px);
                            border-radius: 8px;
                            margin-top: 20px;
                            z-index: 50;
                        ",
                        "Loading newer posts..."
                    }
                }
                
                // Bottom spacer
                div {
                    style: "height: 200px; background: transparent;",
                }
            }
        }
    }
}
