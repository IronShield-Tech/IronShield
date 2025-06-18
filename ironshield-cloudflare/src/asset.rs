use axum::body;
use http::{header, Response, StatusCode};
use worker::{console_log, Error};
use crate::challenge::CHALLENGE_CSS;

/// Using placeholders during development to avoid linter errors,
/// These will be correctly populated at runtime by wrangler
#[cfg(not(target_arch = "wasm32"))]
const WASM_BINARY: &[u8] = &[];
#[cfg(not(target_arch = "wasm32"))]
const WASM_JS_BINDINGS: &[u8] = &[];
#[cfg(not(target_arch = "wasm32"))]
const POW_WORKER_JS: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const WASM_POW_WORKER_JS: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const CHALLENGE_MAIN_JS: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const UI_MANAGER_JS: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const WORKER_POOL_MANAGER_JS: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const API_CLIENT_JS: &str = "";

/// For builds with wrangler, fixed paths with correct relative paths
#[cfg(target_arch = "wasm32")]
const WASM_BINARY: &[u8] = include_bytes!("../../assets/wasm/ironshield_wasm_bg.wasm");
#[cfg(target_arch = "wasm32")]
const WASM_JS_BINDINGS: &[u8] = include_bytes!("../../assets/wasm/ironshield_wasm.js");
#[cfg(target_arch = "wasm32")]
const POW_WORKER_JS: &str = include_str!("../../assets/pow_worker.js");
#[cfg(target_arch = "wasm32")]
const WASM_POW_WORKER_JS: &str = include_str!("../../assets/wasm_pow_worker.js");
#[cfg(target_arch = "wasm32")]
const CHALLENGE_MAIN_JS: &str = include_str!("../../assets/challenge_main.js");
#[cfg(target_arch = "wasm32")]
const UI_MANAGER_JS: &str = include_str!("../../assets/ui_manager.js");
#[cfg(target_arch = "wasm32")]
const WORKER_POOL_MANAGER_JS: &str = include_str!("../../assets/worker_pool_manager.js");
#[cfg(target_arch = "wasm32")]
const API_CLIENT_JS: &str = include_str!("../../assets/api_client.js");

/// Function to serve the WebAssembly binary
async fn serve_wasm_file() -> worker::Result<Response<body::Body>> {
    console_log!("Serving WebAssembly binary...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/wasm")
        // Add cache control headers to help with caching
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        // Add CORS headers to ensure it can be loaded from different origins if needed
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        // Add streaming-friendly headers
        .header(header::ACCEPT_RANGES, "bytes")
        // Add a content-encoding header to indicate no compression
        // This is important for streaming as compressed responses need to be fully downloaded first
        .header(header::CONTENT_ENCODING, "identity")
        .body(body::Body::from(WASM_BINARY.to_vec()))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve WebAssembly: {}", e)))
}

/// Function to serve the JavaScript bindings for the WebAssembly module
async fn serve_wasm_js_file() -> worker::Result<Response<body::Body>> {
    console_log!("Serving WebAssembly JavaScript bindings...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        // Add CORS headers
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(body::Body::from(WASM_JS_BINDINGS.to_vec()))
        .map_err(|e: http::Error| {
            Error::RustError(format!("Failed to serve WebAssembly JS bindings: {}", e))
        })
}

/// Function to serve the challenge CSS file
async fn serve_challenge_css() -> worker::Result<Response<body::Body>> {
    console_log!("Serving challenge CSS...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(CHALLENGE_CSS))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve challenge CSS: {}", e)))
}

/// Function to serve the PoW worker JavaScript file
async fn serve_pow_worker_js() -> worker::Result<Response<body::Body>> {
    console_log!("Serving PoW worker JS...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(POW_WORKER_JS))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve PoW worker: {}", e)))
}

/// Function to serve the WASM PoW worker JavaScript file
async fn serve_wasm_pow_worker_js() -> worker::Result<Response<body::Body>> {
    console_log!("Serving WASM PoW worker JS...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(WASM_POW_WORKER_JS))
        .map_err(|e: http::Error| {
            Error::RustError(format!("Failed to serve WASM PoW worker: {}", e))
        })
}

/// Function to serve the main challenge JavaScript file
async fn serve_challenge_main_js() -> worker::Result<Response<body::Body>> {
    serve_javascript_file("main challenge JS", CHALLENGE_MAIN_JS)
}

/// Function to serve the UI Manager JavaScript file
async fn serve_ui_manager_js() -> worker::Result<Response<body::Body>> {
    serve_javascript_file("UI manager JS", UI_MANAGER_JS)
}

/// Function to serve the Worker Pool Manager JavaScript file
async fn serve_worker_pool_manager_js() -> worker::Result<Response<body::Body>> {
    serve_javascript_file("Worker pool manager JS", WORKER_POOL_MANAGER_JS)
}

/// Function to serve the API-Client JavaScript file
async fn serve_api_client_js() -> worker::Result<Response<body::Body>> {
    serve_javascript_file("API client JS", API_CLIENT_JS)
}

/// Generic function to serve JavaScript files
fn serve_javascript_file(log_name: &str, content: &'static str) -> worker::Result<Response<body::Body>> {
    console_log!("Serving {}...", log_name);
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(content))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve {}: {}", log_name, e)))
}

/// Function to handle asset requests
pub(crate) async fn handle_asset_request(path: &str) -> Option<worker::Result<Response<body::Body>>> {
    match path {
        // WASM files
        "/ironshield_wasm_bg.wasm" | "/assets/wasm/ironshield_wasm_bg.wasm" => {
            console_log!("Request for WebAssembly binary received.");
            Some(serve_wasm_file().await)
        }
        "/ironshield_wasm.js" | "/assets/wasm/ironshield_wasm.js" => {
            console_log!("Request for WebAssembly JS bindings received.");
            Some(serve_wasm_js_file().await)
        }
        // CSS
        "/challenge.css" | "/assets/challenge.css" => {
            console_log!("Request for challenge CSS received.");
            Some(serve_challenge_css().await)
        }
        // JavaScript files
        "/pow_worker.js" | "/assets/pow_worker.js" => {
            console_log!("Request for PoW worker JS received.");
            Some(serve_pow_worker_js().await)
        }
        "/wasm_pow_worker.js" | "/assets/wasm_pow_worker.js" => {
            console_log!("Request for WASM PoW worker JS received.");
            Some(serve_wasm_pow_worker_js().await)
        }
        "/challenge_main.js" | "/assets/challenge_main.js" => {
            console_log!("Request for main challenge JS received.");
            Some(serve_challenge_main_js().await)
        }
        "/ui_manager.js" | "/assets/ui_manager.js" => {
            console_log!("Request for UI manager JS received.");
            Some(serve_ui_manager_js().await)
        }
        "/worker_pool_manager.js" | "/assets/worker_pool_manager.js" => {
            console_log!("Request for Worker pool manager JS received.");
            Some(serve_worker_pool_manager_js().await)
        }
        "/api_client.js" | "/assets/api_client.js" => {
            console_log!("Request for API client JS received.");
            Some(serve_api_client_js().await)
        }
        // Return None if not an asset request
        _ => None,
    }
}