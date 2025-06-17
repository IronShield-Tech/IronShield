use axum::{
    body::{self},
    http::{header, Method as AxumMethod, Request, Response, StatusCode},
};
use chrono::Utc;
use hex;
use ironshield_core;
use rand;
use worker::*;

// Using placeholders during development to avoid linter errors
// These will be correctly populated at runtime by wrangler
#[cfg(not(target_arch = "wasm32"))]
const WASM_BINARY: &[u8] = &[];
#[cfg(not(target_arch = "wasm32"))]
const WASM_JS_BINDINGS: &[u8] = &[];
#[cfg(not(target_arch = "wasm32"))]
const CHALLENGE_TEMPLATE: &str = "";
#[cfg(not(target_arch = "wasm32"))]
const CHALLENGE_CSS: &str = "";
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

// For builds with wrangler - fixed paths with correct relative paths
#[cfg(target_arch = "wasm32")]
const WASM_BINARY: &[u8] = include_bytes!("../../assets/wasm/ironshield_wasm_bg.wasm");
#[cfg(target_arch = "wasm32")]
const WASM_JS_BINDINGS: &[u8] = include_bytes!("../../assets/wasm/ironshield_wasm.js");
#[cfg(target_arch = "wasm32")]
const CHALLENGE_TEMPLATE: &str = include_str!("../../assets/challenge_template.html");
#[cfg(target_arch = "wasm32")]
const CHALLENGE_CSS: &str = include_str!("../../assets/challenge.css");
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

// --- Constants ---
const POW_DIFFICULTY: usize = 4; // Number of leading zeros required in the hash
const CHALLENGE_HEADER: &str = "X-IronShield-Challenge";
const NONCE_HEADER: &str = "X-IronShield-Nonce";
const TIMESTAMP_HEADER: &str = "X-IronShield-Timestamp";
const DIFFICULTY_HEADER: &str = "X-IronShield-Difficulty";
const MAX_CHALLENGE_AGE_SECONDS: i64 = 60; // How long a challenge is valid
const BYPASS_TOKEN_HEADER: &str = "X-Ironshield-Token";
const BYPASS_TOKEN_VALUE: &str = "test_approved";
const BYPASS_COOKIE_NAME: &str = "ironshield_token";
const ALLOWED_ORIGINS: [&str; 3] = [
    "http://localhost:8787",
    "https://skip.ironshield.cloud",
    "https://ironshield.cloud",
];

// Simple placeholder for successful access
async fn protected_content() -> &'static str {
    "Access Granted: Checksum Approved."
}

// Function to generate the challenge page that uses WebAssembly
fn generate_challenge_page(
    challenge_string: &str,
    timestamp: i64,
    headers: &axum::http::HeaderMap,
) -> Result<Response<body::Body>> {
    console_log!(
        "Issuing WebAssembly challenge with timestamp: {}",
        timestamp
    );

    // Create meta tags for all parameters
    let difficulty_meta_tag: String = format!(
        "<meta name=\"x-ironshield-difficulty\" content=\"{}\">",
        POW_DIFFICULTY
    );
    let timestamp_meta_tag: String = format!(
        "<meta name=\"x-ironshield-timestamp\" content=\"{}\">",
        timestamp
    );
    let challenge_meta_tag: String = format!(
        "<meta name=\"x-ironshield-challenge\" content=\"{}\">",
        challenge_string
    );

    // Replace placeholders in the template, and add our meta tags after the viewport meta
    let html_content = CHALLENGE_TEMPLATE
        .replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">",
                &format!("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    {}\n    {}\n    {}", 
                         difficulty_meta_tag, timestamp_meta_tag, challenge_meta_tag))
        // Keep these replacements for header names
        .replace("X-Challenge", CHALLENGE_HEADER)
        .replace("X-Nonce", NONCE_HEADER)
        .replace("X-Timestamp", TIMESTAMP_HEADER);

    add_cors_headers(
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html")
            .header(DIFFICULTY_HEADER, POW_DIFFICULTY.to_string())
            .header(TIMESTAMP_HEADER, timestamp.to_string())
            .header(CHALLENGE_HEADER, challenge_string),
        headers,
    )
    .body(body::Body::from(html_content))
    .map_err(|e: http::Error| {
        Error::RustError(format!("Failed to build challenge response: {}", e))
    })
}

// Function to verify the submitted solution
fn verify_solution(req: &Request<worker::Body>) -> bool {
    console_log!("Verifying checksum...");

    let headers: &http::HeaderMap = req.headers();
    let challenge_opt: Option<&str> = headers.get(CHALLENGE_HEADER).and_then(|v| v.to_str().ok());
    let nonce_opt: Option<&str> = headers.get(NONCE_HEADER).and_then(|v| v.to_str().ok());
    let timestamp_opt: Option<&str> = headers.get(TIMESTAMP_HEADER).and_then(|v| v.to_str().ok());
    let difficulty_opt: Option<&str> = headers.get(DIFFICULTY_HEADER).and_then(|v| v.to_str().ok());

    match (challenge_opt, nonce_opt, timestamp_opt, difficulty_opt) {
        (Some(challenge), Some(nonce_str), Some(timestamp_str), Some(difficulty_str)) => {
            // 1. Verify timestamp freshness
            match timestamp_str.parse::<i64>() {
                Ok(timestamp_millis) => {
                    let now_millis: i64 = Utc::now().timestamp_millis();
                    if now_millis.saturating_sub(timestamp_millis)
                        > MAX_CHALLENGE_AGE_SECONDS * 1000
                    {
                        console_log!(
                            "Challenge timestamp expired. Now: {}, Provided: {}",
                            now_millis,
                            timestamp_millis
                        );
                        return false;
                    }
                    // Optionally check if timestamp is too far in the future as well?
                    // if timestamp_millis > now_millis + 5000 { // e.g., 5 seconds tolerance
                    //     console_log!("Challenge timestamp is in the future.");
                    //     return false;
                    // }
                }
                Err(_) => {
                    console_log!(
                        "Invalid timestamp format (expected Unix ms). Received: {}",
                        timestamp_str
                    );
                    return false;
                }
            }

            // 2. Parse difficulty
            let difficulty: usize = match difficulty_str.parse::<usize>() {
                Ok(d) => d,
                Err(_) => {
                    console_log!("Invalid difficulty format.");
                    return false;
                }
            };

            // 3. Verify solution using our core library
            let result: bool = ironshield_core::verify_solution(challenge, nonce_str, difficulty);

            if result {
                console_log!("Checksum verification successful!");
                true
            } else {
                console_log!("Checksum verification failed.");
                false
            }
        }
        _ => {
            console_log!("Missing required PoW headers.");
            false // Missing headers
        }
    }
}

// Function to serve the WebAssembly binary
async fn serve_wasm_file() -> Result<Response<body::Body>> {
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
        // Add content-encoding header to indicate no compression
        // This is important for streaming as compressed responses need to be fully downloaded first
        .header(header::CONTENT_ENCODING, "identity")
        .body(body::Body::from(WASM_BINARY.to_vec()))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve WebAssembly: {}", e)))
}

// Function to serve the JavaScript bindings for the WebAssembly module
async fn serve_wasm_js_file() -> Result<Response<body::Body>> {
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

// Function to serve the challenge CSS file
async fn serve_challenge_css() -> Result<Response<body::Body>> {
    console_log!("Serving challenge CSS...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(CHALLENGE_CSS))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve challenge CSS: {}", e)))
}

// Function to serve the PoW worker JavaScript file
async fn serve_pow_worker_js() -> Result<Response<body::Body>> {
    console_log!("Serving PoW worker JS...");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body::Body::from(POW_WORKER_JS))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to serve PoW worker: {}", e)))
}

// Function to serve the WASM PoW worker JavaScript file
async fn serve_wasm_pow_worker_js() -> Result<Response<body::Body>> {
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

// Function to serve the main challenge JavaScript file
async fn serve_challenge_main_js() -> Result<Response<body::Body>> {
    serve_javascript_file("main challenge JS", CHALLENGE_MAIN_JS)
}

// Function to serve the UI Manager JavaScript file
async fn serve_ui_manager_js() -> Result<Response<body::Body>> {
    serve_javascript_file("UI manager JS", UI_MANAGER_JS)
}

// Function to serve the Worker Pool Manager JavaScript file
async fn serve_worker_pool_manager_js() -> Result<Response<body::Body>> {
    serve_javascript_file("Worker pool manager JS", WORKER_POOL_MANAGER_JS)
}

// Function to serve the API Client JavaScript file
async fn serve_api_client_js() -> Result<Response<body::Body>> {
    serve_javascript_file("API client JS", API_CLIENT_JS)
}

// Generic function to serve JavaScript files
fn serve_javascript_file(log_name: &str, content: &'static str) -> Result<Response<body::Body>> {
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

// Helper function to handle CORS for responses
fn add_cors_headers(
    builder: axum::http::response::Builder,
    request_headers: &axum::http::HeaderMap,
) -> axum::http::response::Builder {
    let mut builder: http::response::Builder = builder;

    // Get the origin header from the request
    let origin: &str = request_headers
        .get(header::ORIGIN)
        .and_then(|v: &http::HeaderValue| v.to_str().ok())
        .unwrap_or("");

    // Check if the origin is allowed
    let is_allowed_origin: bool = ALLOWED_ORIGINS.contains(&origin) || origin.is_empty();

    // Set appropriate Access-Control-Allow-Origin header
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    } else {
        // Fallback to wildcard if no origin is specified or it's not in our allowed list
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    }

    // Add other CORS headers
    builder = builder
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, X-IronShield-Challenge, X-IronShield-Nonce, X-IronShield-Timestamp, X-IronShield-Difficulty, X-Ironshield-Token")
        .header(header::VARY, "Origin"); // Important for caching

    // Only add credentials header if we have a specific origin (not wildcard)
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
    }

    builder
}

// Function to handle asset requests
async fn handle_asset_request(path: &str) -> Option<Result<Response<body::Body>>> {
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
        _ => None
    }
}

/// Function to check for bypass token in headers
fn check_bypass_token(headers: &axum::http::HeaderMap) -> Option<Result<Response<body::Body>>> {
    let token = headers.get(BYPASS_TOKEN_HEADER)?;
    
    if token
        .to_str()
        .map(|t| t == BYPASS_TOKEN_VALUE)
        .unwrap_or(false)
    {
        return None;
    }

    console_log!("Bypass token found and valid, skipping PoW verification");
    // Perform a direct redirect to skip.ironshield.cloud
    Some(
        add_cors_headers(
            Response::builder()
                .status(StatusCode::FOUND) // 302 Found for redirect
                .header(header::LOCATION, "https://skip.ironshield.cloud")
                .header(header::CONTENT_TYPE, "text/plain"),
            &headers,
        )
            .body(body::Body::from("Redirecting to approved endpoint..."))
            .map_err(|e: http::Error| {
                Error::RustError(format!("Failed to build response: {}", e))
            })
    )

}

/// Function to check for bypass cookie
fn check_bypass_cookie(headers: &axum::http::HeaderMap) -> Option<Result<Response<body::Body>>> {
    let      cookie_header = headers.get(header::COOKIE)?;
    let         cookie_str = cookie_header.to_str().ok()?;
    let cookies: Vec<&str> = cookie_str.split(';').collect();
    
    for cookie in cookies {
        let cookie_parts: Vec<&str> = cookie.trim().split('=').collect();
        
        if cookie_parts.len() != 2 {
            continue;
        }

        if cookie_parts[0] != BYPASS_COOKIE_NAME {
            continue;
        }

        if cookie_parts[1] != BYPASS_TOKEN_VALUE {
            continue;
        }

        console_log!("Bypass cookie found and valid, skipping PoW verification");
        // Perform a direct redirect to skip.ironshield.cloud
        return Some(
            add_cors_headers(
                Response::builder()
                    .status(StatusCode::FOUND) // 302 Found for redirect
                    .header(header::LOCATION, "https://skip.ironshield.cloud")
                    .header(header::CONTENT_TYPE, "text/plain"),
                &headers,
            )
                .body(body::Body::from("Redirecting to approved endpoint..."))
                .map_err(|e: http::Error| {
                    Error::RustError(format!("Failed to build response: {}", e))
                })
        );
    }
    None
}


// Main Worker entry point
#[event(fetch)]
pub async fn main(
    req: Request<worker::Body>,
    _env: Env,
    _ctx: worker::Context,
) -> Result<Response<body::Body>> {
    // Optionally, set a panic hook for better error messages in the browser console.
    utils::set_panic_hook();
    
    if let Some(asset_response) = handle_asset_request(req.uri().path()).await {
        return asset_response;
    }
    
    let headers = req.headers();
    
    if let Some(response) = check_bypass_token(&headers) {
        return response;
    }
    
    if let Some(response) = check_bypass_cookie(&headers) {
        return response;
    }
    
    // Existing logic for handling GET requests (challenge/verification)
    let has_pow_headers = headers.contains_key(CHALLENGE_HEADER)
        && headers.contains_key(NONCE_HEADER)
        && headers.contains_key(TIMESTAMP_HEADER)
        && headers.contains_key(DIFFICULTY_HEADER);

    match *req.method() {
        AxumMethod::GET => {
            if has_pow_headers {
                // Verify Proof of Work if verification headers are present
                if verify_solution(&req) {
                    // If verification is successful, create a response that sets the bypass cookie
                    let cookie_value: String = format!(
                        "{}={}; Max-Age=900; HttpOnly; Secure; Path=/; SameSite=Lax", // Added Max-Age=900 for 15 minutes
                        BYPASS_COOKIE_NAME,
                        BYPASS_TOKEN_VALUE // Still using the insecure test value for now
                    );
                    // Return protected content if verification succeeds
                    #[allow(unused_variables)]
                    let content = protected_content().await;
                    return add_cors_headers(Response::builder()
                        .status(StatusCode::OK) // Use 200 OK instead of 302 redirect
                        .header(header::SET_COOKIE, cookie_value)
                        .header(header::CONTENT_TYPE, "application/json"), &headers)
                        .body(body::Body::from(format!("{{\"success\":true,\"message\":\"Verification successful.\",\"redirectUrl\":\"https://skip.ironshield.cloud\"}}")))
                        .map_err(|e: http::Error| Error::RustError(format!("Failed to build response: {}", e)));
                } else {
                    // Reject if verification fails
                    return add_cors_headers(
                        Response::builder()
                            .status(StatusCode::FORBIDDEN)
                            .header(header::CONTENT_TYPE, "text/plain"),
                        &headers,
                    )
                    .body(body::Body::from(
                        "Proof of Work verification failed. Please try again.",
                    ))
                    .map_err(|e: http::Error| {
                        Error::RustError(format!("Failed to build response: {}", e))
                    });
                }
            }
            // No verification headers - issue challenge
            // Generate a fresh challenge
            let challenge: String = hex::encode(&rand::random::<[u8; 16]>());
            let timestamp_ms: i64 = Utc::now().timestamp_millis();

            // Return the challenge page
            return generate_challenge_page(&challenge, timestamp_ms, &headers);
        }
        AxumMethod::OPTIONS => {
            // Handle CORS preflight requests
            console_log!("Handling OPTIONS request for CORS preflight");
            return add_cors_headers(
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::ACCESS_CONTROL_MAX_AGE, "86400"),
                &headers,
            ) // 24 hours
            .body(body::Body::from(""))
            .map_err(|e: http::Error| {
                Error::RustError(format!("Failed to build OPTIONS response: {}", e))
            });
        }
        // Reject any other methods
        _ => add_cors_headers(
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::CONTENT_TYPE, "text/plain"),
            &headers,
        )
        .body(body::Body::from("Method not allowed"))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to build response: {}", e))),
    }
}

// Utility functions
mod utils {
    pub fn set_panic_hook() {
        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function to get better error messages if the code panics.
        console_error_panic_hook::set_once();
    }
}
