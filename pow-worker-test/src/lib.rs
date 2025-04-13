use axum::{
    body::{self},
    http::{Request, Response, StatusCode, header, Method as AxumMethod},
};
use worker::*;
use hex;
use chrono::{Utc, Duration, DateTime};

// Include the WebAssembly client module
mod pow_client;

// Make the WebAssembly binary and its JS bindings available to include
const WASM_BINARY: &[u8] = include_bytes!("../wasm/pow_wasm_bg.wasm");
const WASM_JS_BINDINGS: &[u8] = include_bytes!("../wasm/pow_wasm.js");
const CHALLENGE_TEMPLATE: &str = include_str!("../assets/challenge_template.html");
const CHALLENGE_CSS: &str = include_str!("../assets/challenge.css");
const POW_WORKER_JS: &str = include_str!("../assets/pow_worker.js");
const CHALLENGE_MAIN_JS: &str = include_str!("../assets/challenge_main.js");

// --- Constants ---
const POW_DIFFICULTY: usize = 4; // Number of leading zeros required in the hash
const CHALLENGE_HEADER: &str = "X-IronShield-Challenge";
const NONCE_HEADER: &str = "X-IronShield-Nonce";
const TIMESTAMP_HEADER: &str = "X-IronShield-Timestamp";
const DIFFICULTY_HEADER: &str = "X-IronShield-Difficulty";
const MAX_CHALLENGE_AGE_SECONDS: i64 = 60; // How long a challenge is valid

// Simple placeholder for successful access
async fn protected_content() -> &'static str {
    "Access Granted: Checksum Approved."
}

// Function to generate the challenge page that uses WebAssembly
fn generate_challenge_page(challenge_string: &str, timestamp: i64) -> Result<Response<body::Body>> {
    console_log!("Issuing WebAssembly challenge with timestamp: {}", timestamp);

    // Create meta tags for all parameters
    let difficulty_meta_tag = format!("<meta name=\"x-ironshield-difficulty\" content=\"{}\">", POW_DIFFICULTY);
    let timestamp_meta_tag = format!("<meta name=\"x-ironshield-timestamp\" content=\"{}\">", timestamp);
    let challenge_meta_tag = format!("<meta name=\"x-ironshield-challenge\" content=\"{}\">", challenge_string);
    
    // Replace placeholders in the template, and add our meta tags after the viewport meta
    let html_content = CHALLENGE_TEMPLATE
        .replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">",
                &format!("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    {}\n    {}\n    {}", 
                         difficulty_meta_tag, timestamp_meta_tag, challenge_meta_tag))
        // Keep these replacements for header names
        .replace("X-Challenge", CHALLENGE_HEADER)
        .replace("X-Nonce", NONCE_HEADER)
        .replace("X-Timestamp", TIMESTAMP_HEADER);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .header(DIFFICULTY_HEADER, POW_DIFFICULTY.to_string())
        .header(TIMESTAMP_HEADER, timestamp.to_string())
        .header(CHALLENGE_HEADER, challenge_string)
        .body(body::Body::from(html_content))
        .map_err(|e| Error::RustError(format!("Failed to build challenge response: {}", e)))
}

// Function to verify the submitted solution
fn verify_solution(req: &Request<worker::Body>) -> bool {
    console_log!("Verifying checksum...");

    let headers = req.headers();
    let challenge_opt = headers.get(CHALLENGE_HEADER).and_then(|v| v.to_str().ok());
    let nonce_opt = headers.get(NONCE_HEADER).and_then(|v| v.to_str().ok());
    let timestamp_opt = headers.get(TIMESTAMP_HEADER).and_then(|v| v.to_str().ok());
    let difficulty_opt = headers.get(DIFFICULTY_HEADER).and_then(|v| v.to_str().ok());

    match (challenge_opt, nonce_opt, timestamp_opt, difficulty_opt) {
        (Some(challenge), Some(nonce_str), Some(timestamp_str), Some(difficulty_str)) => {
            // 1. Verify timestamp freshness
            match timestamp_str.parse::<i64>() {
                 Ok(timestamp_millis) => {
                    let now_millis = Utc::now().timestamp_millis();
                    if now_millis.saturating_sub(timestamp_millis) > MAX_CHALLENGE_AGE_SECONDS * 1000 {
                        console_log!("Challenge timestamp expired. Now: {}, Provided: {}", now_millis, timestamp_millis);
                        return false;
                    }
                    // Optionally check if timestamp is too far in the future as well?
                    // if timestamp_millis > now_millis + 5000 { // e.g., 5 seconds tolerance
                    //     console_log!("Challenge timestamp is in the future.");
                    //     return false;
                    // }
                 }
                 Err(_) => {
                    console_log!("Invalid timestamp format (expected Unix ms). Received: {}", timestamp_str);
                    return false;
                 }
            }

            // 2. Parse difficulty
            let difficulty = match difficulty_str.parse::<usize>() {
                Ok(d) => d,
                Err(_) => {
                    console_log!("Invalid difficulty format.");
                    return false;
                }
            };

            // 3. Verify solution using our Rust code (same as what's in the WASM module)
            match nonce_str.parse::<u64>() {
                Ok(_) => {  // We don't need the parsed nonce here, just checking it's valid
                    // Use the function from pow_client
                    let result = pow_client::verify_pow_solution(challenge, nonce_str, difficulty);
                    
                    if result {
                        console_log!("Checksum verification successful!");
                        true
                    } else {
                        console_log!("Checksum verification failed.");
                        false
                    }
                }
                Err(_) => {
                    console_log!("Invalid nonce format.");
                    false
                }
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
        .body(body::Body::from(WASM_BINARY.to_vec()))
        .map_err(|e| Error::RustError(format!("Failed to serve WebAssembly: {}", e)))
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
        .map_err(|e| Error::RustError(format!("Failed to serve WebAssembly JS bindings: {}", e)))
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
        .map_err(|e| Error::RustError(format!("Failed to serve challenge CSS: {}", e)))
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
        .map_err(|e| Error::RustError(format!("Failed to serve PoW worker JS: {}", e)))
}

// Function to serve the main challenge JavaScript file
async fn serve_challenge_main_js() -> Result<Response<body::Body>> {
    console_log!("Serving main challenge JS...");
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        // Add cache control headers
        .header(header::CACHE_CONTROL, "public, max-age=3600") 
        .body(body::Body::from(CHALLENGE_MAIN_JS))
        .map_err(|e| Error::RustError(format!("Failed to serve main challenge JS: {}", e)))
}

// Main Worker entry point
#[event(fetch)]
pub async fn main(req: Request<worker::Body>, _env: Env, _ctx: worker::Context) -> Result<Response<body::Body>> {
    // Optionally set panic hook for better error messages in browser console
    utils::set_panic_hook();

    let headers = req.headers();
    let has_pow_headers = headers.contains_key(CHALLENGE_HEADER)
        && headers.contains_key(NONCE_HEADER)
        && headers.contains_key(TIMESTAMP_HEADER)
        && headers.contains_key(DIFFICULTY_HEADER);

    // Handle request for WebAssembly files
    match req.uri().path() {
        "/pow_wasm_bg.wasm" => {
            console_log!("Request for WebAssembly binary received. Size: {} bytes", WASM_BINARY.len());
            return serve_wasm_file().await;
        }
        "/pow_wasm.js" => {
            console_log!("Request for WebAssembly JS bindings received. Size: {} bytes", WASM_JS_BINDINGS.len());
            return serve_wasm_js_file().await;
        }
        "/challenge.css" => {
            console_log!("Request for challenge CSS received.");
            return serve_challenge_css().await;
        }
        "/pow_worker.js" => {
            console_log!("Request for PoW worker JS received.");
            return serve_pow_worker_js().await;
        }
        "/challenge_main.js" => {
            console_log!("Request for main challenge JS received.");
            return serve_challenge_main_js().await;
        }
        _ => {}
    }

    match *req.method() {
        AxumMethod::GET => {
            if has_pow_headers {
                // Verify Proof of Work if verification headers are present
                if verify_solution(&req) {
                    // Return protected content if verification succeeds
                    let content = protected_content().await;
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(body::Body::from(content))
                        .map_err(|e| Error::RustError(format!("Failed to build response: {}", e)));
                } else {
                    // Reject if verification fails
                    return Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .header(header::CONTENT_TYPE, "text/plain")
                        .body(body::Body::from("Proof of Work verification failed. Please try again."))
                        .map_err(|e| Error::RustError(format!("Failed to build response: {}", e)));
                }
            } else {
                // No verification headers - issue challenge
                // Generate a fresh challenge
                let challenge = hex::encode(&rand::random::<[u8; 16]>());
                let timestamp_ms = Utc::now().timestamp_millis();
                
                // Return the challenge page
                return generate_challenge_page(&challenge, timestamp_ms);
            }
        },
        // Reject any other methods
        _ => {
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(body::Body::from("Method not allowed"))
                .map_err(|e| Error::RustError(format!("Failed to build response: {}", e)))
        }
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
