mod bypass;
mod challenge;
mod cors;
mod difficulty;
mod http_handler;
mod asset;

use axum::{
    body::{self},
    http::{Method as AxumMethod, Request, Response},
};
use worker::*;

use asset::handle_asset_request;
use bypass::{check_bypass_cookie, check_bypass_token};
use cors::add_cors_headers;
use http_handler::{
    handle_get_request, handle_options_request, handle_unsupported_method,
    has_proof_of_work_headers,
};

// --- Constants ---
const CHALLENGE_HEADER: &str = "X-IronShield-Challenge";
const NONCE_HEADER: &str = "X-IronShield-Nonce";
const TIMESTAMP_HEADER: &str = "X-IronShield-Timestamp";
const DIFFICULTY_HEADER: &str = "X-IronShield-Difficulty";
pub const BYPASS_TOKEN_HEADER: &str = "X-Ironshield-Token";
pub const BYPASS_TOKEN_VALUE: &str = "test_approved";
pub const BYPASS_COOKIE_NAME: &str = "ironshield_token";
const ALLOWED_ORIGINS: [&str; 3] = [
    "http://localhost:8787",
    "https://skip.ironshield.cloud",
    "https://ironshield.cloud",
];

/// Main Worker entry point
#[event(fetch)]
pub async fn main(req: Request<Body>, _env: Env, _ctx: Context) -> Result<Response<body::Body>> {
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

    // Check for Proof of Work headers
    let has_pow_headers = has_proof_of_work_headers(&headers);

    // Route based on HTTP method
    match *req.method() {
        AxumMethod::GET => handle_get_request(&req, &headers, has_pow_headers).await,
        AxumMethod::OPTIONS => handle_options_request(&headers),
        _ => handle_unsupported_method(&headers),
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