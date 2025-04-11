use axum::{
    body::{self},
    http::{Request, Response, StatusCode, header, Method as AxumMethod},
};
use worker::*;
use sha2::{Sha256, Digest};
use hex;
use chrono::{Utc, Duration, DateTime};

// Include the WebAssembly client module
mod pow_client;

// Make the WebAssembly binary and its JS bindings available to include
const WASM_BINARY: &[u8] = include_bytes!("../wasm/pow_wasm_bg.wasm");
const WASM_JS_BINDINGS: &[u8] = include_bytes!("../wasm/pow_wasm.js");
const CHALLENGE_TEMPLATE: &str = include_str!("../assets/challenge_template.html");

// --- Constants ---
const POW_DIFFICULTY: usize = 4; // Number of leading zeros required in the hash
const CHALLENGE_HEADER: &str = "X-IronShield-Challenge";
const NONCE_HEADER: &str = "X-IronShield-Nonce";
const TIMESTAMP_HEADER: &str = "X-IronShield-Timestamp";
const DIFFICULTY_HEADER: &str = "X-IronShield-Difficulty"; // New header for difficulty
const MAX_CHALLENGE_AGE_SECONDS: i64 = 60; // How long a challenge is valid

// Simple placeholder for successful access
async fn protected_content() -> &'static str {
    "Access Granted: Checksum Approved."
}

// Function to generate the challenge page that uses WebAssembly
fn generate_challenge_page(challenge_string: &str, timestamp: DateTime<Utc>) -> Result<Response<body::Body>> {
    console_log!("Issuing WebAssembly challenge...");

    // Create a meta tag for the difficulty to be read by client-side JavaScript
    let difficulty_meta_tag = format!("<meta name=\"x-ironshield-difficulty\" content=\"{}\">", POW_DIFFICULTY);
    
    // Replace placeholders in the template, and add our difficulty meta tag after the viewport meta
    let html_content = CHALLENGE_TEMPLATE
        .replace("CHALLENGE_PLACEHOLDER", challenge_string)
        .replace("TIMESTAMP_PLACEHOLDER", &timestamp.to_rfc3339())
        .replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">",
                &format!("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    {}", difficulty_meta_tag))
        // Keep these replacements for header names
        .replace("X-Challenge", CHALLENGE_HEADER)
        .replace("X-Nonce", NONCE_HEADER)
        .replace("X-Timestamp", TIMESTAMP_HEADER);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .header(DIFFICULTY_HEADER, POW_DIFFICULTY.to_string()) // Add difficulty header
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
            match DateTime::parse_from_rfc3339(timestamp_str) {
                 Ok(timestamp_with_offset) => {
                    let timestamp_utc = timestamp_with_offset.with_timezone(&Utc);
                    let now = Utc::now();
                    if now.signed_duration_since(timestamp_utc) > Duration::seconds(MAX_CHALLENGE_AGE_SECONDS) {
                        console_log!("Challenge timestamp expired.");
                        return false;
                    }
                 }
                 Err(_) => {
                    console_log!("Invalid timestamp format.");
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

            // 3. Verify nonce format
            match nonce_str.parse::<u64>() {
                Ok(nonce) => {
                    // 4. Recompute hash
                    let data_to_hash = format!("{}:{}", challenge, nonce);
                    let mut hasher = Sha256::new();
                    hasher.update(data_to_hash.as_bytes());
                    let hash_bytes = hasher.finalize();
                    let hash_hex = hex::encode(hash_bytes);

                    // 5. Check difficulty using the received difficulty
                    let target_prefix = "0".repeat(difficulty);
                    if hash_hex.starts_with(&target_prefix) {
                        console_log!("Checksum verification successful (Hash: {}..., Difficulty: {}).", &hash_hex[..8], difficulty);
                        true
                    } else {
                        console_log!("Checksum verification failed (Hash: {}..., Difficulty: {}).", &hash_hex[..8], difficulty);
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
        .body(body::Body::from(WASM_BINARY.to_vec()))
        .map_err(|e| Error::RustError(format!("Failed to serve WebAssembly: {}", e)))
}

// Function to serve the JavaScript bindings for the WebAssembly module
async fn serve_wasm_js_file() -> Result<Response<body::Body>> {
    console_log!("Serving WebAssembly JavaScript bindings...");
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .body(body::Body::from(WASM_JS_BINDINGS.to_vec()))
        .map_err(|e| Error::RustError(format!("Failed to serve WebAssembly JS bindings: {}", e)))
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
            return serve_wasm_file().await;
        }
        "/pow_wasm.js" => {
            return serve_wasm_js_file().await;
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
                let timestamp = Utc::now();
                
                // Return the challenge page
                return generate_challenge_page(&challenge, timestamp);
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
