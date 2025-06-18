use axum::body;
use http::{header, Request, Response, StatusCode};
use worker::{console_log, Body, Error};
use crate::challenge::{handle_solution_verification, issue_new_challenge};
use crate::constant::{CHALLENGE_HEADER, DIFFICULTY_HEADER, NONCE_HEADER, TIMESTAMP_HEADER};
use crate::cors::add_cors_headers;

// Simple placeholder for successful access
pub(crate) async fn protected_content() -> &'static str {
    "Access Granted: Checksum Approved."
}

/// Function to handle GET requests (challenge/verification)
pub(crate) async fn handle_get_request(
    req: &Request<Body>,
    headers: &http::HeaderMap,
    has_pow_headers: bool,
) -> worker::Result<Response<body::Body>> {
    if !has_pow_headers {
        issue_new_challenge(headers).await
    } else {
        handle_solution_verification(req, headers).await
    }
}

/// Function to handle OPTIONS requests (CORS preflight)
pub(crate) fn handle_options_request(headers: &http::HeaderMap) -> worker::Result<Response<body::Body>> {
    console_log!("Handling OPTIONS request for CORS preflight");
    add_cors_headers(
        Response::builder()
            .status(StatusCode::OK)
            .header(header::ACCESS_CONTROL_MAX_AGE, "86400"),
        &headers,
    ) // 24 hours
        .body(body::Body::from(""))
        .map_err(|e: http::Error| {
            Error::RustError(format!("Failed to build OPTIONS response: {}", e))
        })
}

/// Function to handle unsupported HTTP methods
pub(crate) fn handle_unsupported_method(headers: &http::HeaderMap) -> worker::Result<Response<body::Body>> {
    add_cors_headers(
        Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::CONTENT_TYPE, "text/plain"),
        &headers,
    )
        .body(body::Body::from("Method not allowed"))
        .map_err(|e: http::Error| Error::RustError(format!("Failed to build response: {}", e)))
}

/// Function to check if a request has Proof of Work headers
pub(crate) fn has_proof_of_work_headers(headers: &http::HeaderMap) -> bool {
    headers.contains_key(CHALLENGE_HEADER)
        && headers.contains_key(NONCE_HEADER)
        && headers.contains_key(TIMESTAMP_HEADER)
        && headers.contains_key(DIFFICULTY_HEADER)
}