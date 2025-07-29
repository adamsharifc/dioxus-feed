use dioxus::desktop::wry::http::Response;
use dioxus::desktop::use_asset_handler;
use tokio::io::AsyncReadExt;
use std::path::{Path, PathBuf};

// Constants
const MYPROTOCOL_PREFIX: &str = "/myprotocol/";

/// Special symbol to allow filesystem-wide access
pub const ALLOW_ALL_FILESYSTEM: &str = "*";

/// Supported image extensions that can be rendered by webview
const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", 
    "jpeg", 
    "png", 
    "gif", 
    "webp", 
    "bmp", 
    "svg", 
    "ico", 
    "tiff", 
    "tif", 
    "avif"
];

/// Custom error type for protocol handling
#[derive(Debug)]
pub enum ProtocolError {
    PathNotAllowed(String),
    UnsupportedExtension(String),
    FileNotFound(String),
    InvalidPath(String),
    IoError(String),
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::PathNotAllowed(path) => write!(f, "Path not allowed: {}", path),
            ProtocolError::UnsupportedExtension(ext) => write!(f, "Unsupported extension: {}", ext),
            ProtocolError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ProtocolError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
            ProtocolError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

/// Register the custom asset handler for the "myprotocol" scheme.
/// 
/// # Parameters
/// * `allowed_directories` - Vector of directory paths that are allowed for file access.
///                          Use `vec!["*".to_string()]` to allow access to entire filesystem 
/// ```
pub fn register_myprotocol_handler(allowed_directories: Vec<String>) {
    use_asset_handler("myprotocol", move |request, responder| {
        let allowed_dirs = allowed_directories.clone();
        tokio::spawn(async move {
            match handle_protocol_request(request.uri().path(), &allowed_dirs).await {
                Ok(response) => responder.respond(response),
                Err(e) => {
                    eprintln!("Protocol error: {}", e);
                    let error_response = create_error_response(&e);
                    responder.respond(error_response);
                }
            }
        });
    });
}

/// Handle the protocol request and return appropriate response
async fn handle_protocol_request(path: &str, allowed_directories: &[String]) -> Result<Response<Vec<u8>>, ProtocolError> {
    // URL decode the path to handle %20 (spaces) and other encoded characters
    let decoded_path = urlencoding::decode(path)
        .map_err(|_| ProtocolError::InvalidPath(path.to_string()))?;
    
    let file_path_str = extract_file_path(&decoded_path)?;    
    let validated_path = validate_file_path(&file_path_str, allowed_directories)?;
    
    load_file_response(&validated_path).await
}

/// Extract the actual file path from the protocol-prefixed path
fn extract_file_path(decoded_path: &str) -> Result<String, ProtocolError> {
    if decoded_path.starts_with(MYPROTOCOL_PREFIX) {
        let clean_path = &decoded_path[MYPROTOCOL_PREFIX.len()..];
        Ok(clean_path.to_string())
    } else {
        Err(ProtocolError::InvalidPath(format!("Path doesn't start with {}", MYPROTOCOL_PREFIX)))
    }
}

/// Validate file path against allowed directories and supported extensions
fn validate_file_path(file_path: &str, allowed_directories: &[String]) -> Result<PathBuf, ProtocolError> {
    let path = Path::new(file_path);
    
    // Check file extension
    validate_file_extension(path)?;
    
    // Check if filesystem-wide access is allowed
    if allowed_directories.len() == 1 && allowed_directories[0] == ALLOW_ALL_FILESYSTEM {
        return Ok(path.to_path_buf());
    }
    
    // Validate against allowed directories
    validate_directory_access(path, allowed_directories)
}

/// Validate that the file has a supported image extension
fn validate_file_extension(path: &Path) -> Result<(), ProtocolError> {
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .ok_or_else(|| ProtocolError::UnsupportedExtension("No extension found".to_string()))?;
    
    if !SUPPORTED_IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Err(ProtocolError::UnsupportedExtension(extension));
    }
    
    Ok(())
}

/// Validate that the path is within allowed directories
fn validate_directory_access(path: &Path, allowed_directories: &[String]) -> Result<PathBuf, ProtocolError> {
    // Convert to absolute path if possible
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| ProtocolError::IoError(e.to_string()))?
            .join(path)
    };
    
    // Canonicalize to prevent directory traversal
    let canonical_path = abs_path.canonicalize()
        .map_err(|_| ProtocolError::FileNotFound(path.display().to_string()))?;
    
    // Check against allowed directories
    for allowed_dir in allowed_directories {
        let allowed_path = if Path::new(allowed_dir).is_absolute() {
            PathBuf::from(allowed_dir)
        } else {
            std::env::current_dir()
                .map_err(|e| ProtocolError::IoError(e.to_string()))?
                .join(allowed_dir)
        };
        
        if let Ok(canonical_allowed) = allowed_path.canonicalize() {
            if canonical_path.starts_with(&canonical_allowed) {
                return Ok(canonical_path);
            }
        }
    }
    
    Err(ProtocolError::PathNotAllowed(canonical_path.display().to_string()))
}

/// Load file and create HTTP response
async fn load_file_response(file_path: &Path) -> Result<Response<Vec<u8>>, ProtocolError> {
    let mut file = tokio::fs::File::open(file_path).await
        .map_err(|_| ProtocolError::FileNotFound(file_path.display().to_string()))?;
    
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await
        .map_err(|e| ProtocolError::IoError(e.to_string()))?;
    
    let mime = mime_guess::from_path(file_path).first_or_octet_stream();
    let response = Response::builder()
        .header("Content-Type", mime.as_ref())
        .body(bytes)
        .map_err(|e| ProtocolError::IoError(e.to_string()))?;
    
    Ok(response)
}

/// Create appropriate error response based on error type
fn create_error_response(error: &ProtocolError) -> Response<Vec<u8>> {
    let (status, message) = match error {
        ProtocolError::FileNotFound(_) => (404, "File not found"),
        ProtocolError::PathNotAllowed(_) => (403, "Access denied"),
        ProtocolError::UnsupportedExtension(_) => (415, "Unsupported media type"),
        ProtocolError::InvalidPath(_) => (400, "Bad request"),
        ProtocolError::IoError(_) => (500, "Internal server error"),
    };
    
    Response::builder()
        .status(status)
        .body(message.as_bytes().to_vec())
        .unwrap_or_else(|_| {
            Response::builder()
                .status(500)
                .body("Error creating error response".as_bytes().to_vec())
                .unwrap()
        })
}
