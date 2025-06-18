use axum::body;
use chrono::Utc;
use http::{header, Request, Response, StatusCode};
use worker::{console_log, Body, Error};
use crate::cors::add_cors_headers;
use crate::http_handler::protected_content;
use crate::constant::{CHALLENGE_HEADER, NONCE_HEADER, TIMESTAMP_HEADER, DIFFICULTY_HEADER, BYPASS_COOKIE_NAME, BYPASS_TOKEN_VALUE};

/// Serves a multithreaded challenge template for WebAssembly if supported,
/// denoted by the ` not ` function.
#[cfg(not(target_arch = "wasm32"))]
const        CHALLENGE_TEMPLATE:  &str = "";
/// Serves the corresponding challenge page for multithreaded WebAssembly 
/// if supported, denoted by the `not` function.
#[cfg(not(target_arch = "wasm32"))]
pub const         CHALLENGE_CSS:  &str = "";
/// Serves a challenge template that is not multithreaded if not supported,
/// denoted by the lack of the `not` function.
#[cfg(target_arch = "wasm32")]
pub const    CHALLENGE_TEMPLATE:  &str = include_str!("../../assets/challenge_template.html");
/// Serves the corresponding challenge page for the legacy challenge template.
#[cfg(target_arch = "wasm32")]
pub const         CHALLENGE_CSS:  &str = include_str!("../../assets/challenge.css");

/// Number of leading zeros required in the hash.
const            POW_DIFFICULTY: usize = 4;
/// How long a challenge is valid.
const MAX_CHALLENGE_AGE_SECONDS:   i64 = 60;

/// Function to issue a new challenge.
pub(crate) async fn issue_new_challenge(headers: &http::HeaderMap) -> worker::Result<Response<body::Body>> {
    let challenge: String = hex::encode(&rand::random::<[u8; 16]>());
    let timestamp_ms: i64 = Utc::now().timestamp_millis();
    generate_challenge_page(&challenge, timestamp_ms, &headers)
}

/// Function to generate the challenge page that uses WebAssembly.
pub(crate) fn generate_challenge_page(
    challenge_string: &str,
    timestamp: i64,
    headers: &http::HeaderMap,
) -> worker::Result<Response<body::Body>> {
    console_log!(
        "Issuing WebAssembly challenge with timestamp: {}",
        timestamp
    );

    // Create meta-tags for all parameters
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

    // Replace placeholders in the template and add our meta-tags after the viewport meta.
    let html_content = CHALLENGE_TEMPLATE
        .replace("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">",
                 &format!("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n    {}\n    {}\n    {}",
                          difficulty_meta_tag, timestamp_meta_tag, challenge_meta_tag))
        // Keep these replacements for header names.
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

/// Function to verify the submitted solution.
pub(crate) fn verify_solution(req: &Request<Body>) -> bool {
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
//                  // Optionally check if the timestamp is too far in the future as well?
//                  if timestamp_millis > now_millis + 5000 { // e.g., 5 seconds tolerance
//                      console_log!("Challenge timestamp is in the future.");
//                      return false;
//                  }
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

            // 3. Verify the solution using our core library
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

/// Function to handle solution verification and return the appropriate response.
pub(crate) async fn handle_solution_verification(
    req: &Request<Body>,
    headers: &http::HeaderMap,
) -> worker::Result<Response<body::Body>> {
    // Early return for failed verification
    if !verify_solution(&req) {
        let response = add_cors_headers(
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header(header::CONTENT_TYPE, "text/plain"),
            &headers,
        )
            .body(body::Body::from(
                "Proof of Work verification failed. Please try again.",
            ));

        return response.map_err(|e: http::Error| {
            Error::RustError(format!("Failed to build response: {}", e))
        });
    }

    // Verification successful - prepare success response
    let cookie_value = format!(
        "{}={}; Max-Age=900; HttpOnly; Secure; Path=/; SameSite=Lax",
        BYPASS_COOKIE_NAME,
        BYPASS_TOKEN_VALUE
    );

    #[allow(unused_variables)]
    let content = protected_content().await;

    let response = add_cors_headers(
        Response::builder()
            .status(StatusCode::OK)
            .header(header::SET_COOKIE, cookie_value)
            .header(header::CONTENT_TYPE, "application/json"),
        &headers,
    )
        .body(body::Body::from(
            "{\"success\":true,\"message\":\"Verification successful.\",\"redirectUrl\":\"https://skip.ironshield.cloud\"}"
        ));

    response.map_err(|e: http::Error| {
        Error::RustError(format!("Failed to build response: {}", e))
    })
}