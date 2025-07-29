use dioxus::desktop::wry::http::Response;
use dioxus::desktop::use_asset_handler;
use tokio::io::AsyncReadExt;

/// Register the custom asset handler for the "myprotocol" scheme.
pub fn register_myprotocol_handler() {
    use_asset_handler("myprotocol", |request, responder| {
        println!("Asset handler triggered!");
        tokio::spawn(async move {
            let path = request.uri().path();
            println!("Original path: {}", path);

            // URL decode the path to handle %20 (spaces) and other encoded characters
            let decoded_path = urlencoding::decode(path).unwrap_or_else(|_| path.into());
            println!("Decoded path: {}", decoded_path);

            let file_path = if decoded_path.starts_with("/myprotocol/") {
                let clean_path = &decoded_path[12..];
                println!("Clean path after removing /myprotocol/ prefix: {}", clean_path);
                clean_path.to_string()
            } else if decoded_path.starts_with("myprotocol/") {
                let clean_path = &decoded_path[11..];
                println!("Clean path after removing myprotocol/ prefix: {}", clean_path);
                clean_path.to_string()
            } else {
                println!("Path doesn't start with myprotocol/ or /myprotocol/");
                decoded_path.to_string()
            };

            // Check if it's a Windows absolute path (e.g., "C:/Users/...")
            let final_file_path = if file_path.len() >= 3 && file_path.chars().nth(1) == Some(':') {
                // Windows absolute path like "C:/Users/file.jpg"
                println!("Detected Windows absolute path");
                file_path
            } else if file_path.starts_with("./") || file_path.starts_with("../") {
                // Relative path
                println!("Detected relative path");
                file_path
            } else {
                // Unix absolute path or filename
                println!("Detected filename or Unix path");
                file_path
            };

            println!("Final file path: '{}'", final_file_path);

            match tokio::fs::File::open(&final_file_path).await {
                Ok(mut file) => {
                    println!("File opened successfully");
                    let mut bytes = Vec::new();
                    match file.read_to_end(&mut bytes).await {
                        Ok(_) => {
                            println!("File read successfully, {} bytes", bytes.len());
                            let mime = mime_guess::from_path(&final_file_path).first_or_octet_stream();
                            let response = Response::builder()
                                .header("Content-Type", mime.as_ref())
                                .body(bytes)
                                .unwrap();
                            responder.respond(response);
                            return;
                        }
                        Err(e) => println!("Error reading file: {}", e),
                    }
                }
                Err(e) => println!("Error opening file: {}", e),
            }

            // 404 response for missing files
            println!("Sending 404 response");
            let response = Response::builder()
                .status(404)
                .body("File not found".as_bytes().to_vec())
                .unwrap();
            responder.respond(response);
        });
    });
}
