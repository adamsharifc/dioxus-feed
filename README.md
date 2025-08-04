# Dioxus Feed

> ⚠️ **Caution:**  
> This project uses an alpha release of Dioxus (`dioxus = "0.7.0-alpha.3"`.  
> The alpha version is required because the latest stable release does not provide scroll data for the `onscroll` event.  
> An alternative approach using the `onwheel` event and up/down arrow keys was explored, but ultimately abandoned due to lack of robust support.  
> The alpha release provides the necessary `onscroll` event support for virtual scrolling.

A high-performance feed application built with Dioxus featuring virtual scrolling and custom asset loading.

## Features

- **Virtual List**: Efficient rendering of large datasets with virtualization
- **Infinite Scroll**: Bidirectional loading (scroll up/down to load more items)
- **Custom Protocol**: Asset loading via `myprotocol/` for local images
- **Real-time Updates**: Auto-polling for new content
- **Responsive Design**: Clean, flat UI design

## Architecture

```
src/
├─ main.rs              # Application entry point and layout
├─ components/
│  ├─ mod.rs           # Component module exports
│  ├─ feed.rs          # Feed container component
│  ├─ feed_item.rs     # Individual feed item component
│  └─ virtual_list.rs  # Virtual scrolling implementation
└─ protocol/
   ├─ mod.rs           # Protocol module exports
   └─ myprotocol.rs    # Custom asset protocol handler
```

## Key Components

### Virtual List
- Renders only visible items for performance
- Configurable buffer size and item heights
- Scroll direction detection prevents unwanted loading
- Preserves scroll position when adding items at top

### Custom Protocol
- Handles `myprotocol/` URLs for local asset loading
- Provides access to assets directory
- Enables seamless image loading in feed items

## Running the Application

### Development
```bash
dx serve
```

