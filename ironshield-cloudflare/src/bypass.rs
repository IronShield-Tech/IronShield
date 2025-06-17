use axum::{
    body::{self},
    http::{header, Response, StatusCode},
};
use worker::*;

use crate::{
    add_cors_headers,
    BYPASS_TOKEN_HEADER,
    BYPASS_TOKEN_VALUE,
    BYPASS_COOKIE_NAME
};

/// Create a redirect response to `skip.ironshield.cloud`.
fn create_redirect_response(headers: &http::HeaderMap) -> Result<Response<body::Body>> {
    add_cors_headers(
        Response::builder()
            .status(StatusCode::FOUND) // 302 Found for redirect
            .header(header::LOCATION, "https://skip.ironshield.cloud")
            .header(header::CONTENT_TYPE, "text/plain"),
        &headers,
    )
        .body(body::Body::from("Redirecting to approved endpoint..."))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to build response: {}", e)))
}

/// Function to check for bypass token in headers
pub fn check_bypass_token(headers: &http::HeaderMap) -> Option<Result<Response<body::Body>>> {
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
    Some(create_redirect_response(headers))
}

/// Function to check for bypass cookie
pub fn check_bypass_cookie(headers: &http::HeaderMap) -> Option<Result<Response<body::Body>>> {
    let cookie_header = headers.get(header::COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;
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
        return Some(create_redirect_response(headers));
    }
    None
}