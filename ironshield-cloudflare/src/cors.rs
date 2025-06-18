use http::header;

pub const ALLOWED_ORIGINS: [&str; 3] = [
    "http://localhost:8787",
    "https://skip.ironshield.cloud",
    "https://ironshield.cloud",
];

/// Adds CORS (Cross-Origin Resource Sharing) headers to HTTP response 
/// builders for the IronShield service.
/// 
/// The function supports:
/// - Origin validation against a whitelist of allowed domains.
/// - Conditional credential support for trusted origins.
/// - Fallback to wildcard (*) for non-whitelisted origins (without credentials).
/// - Proper cache control headers for CORS preflight responses.
/// 
/// # Arguments 
/// 
/// * `builder`:         An HTTP response builder that will have CORS 
///                      headers added to it. This allows chaining with 
///                      other response configurations before finalizing 
///                      the response.
/// * `request_headers`: The headers from the incoming HTTP request, used 
///                      to extract the Origin header and determine the 
///                      appropriate CORS policy to apply.
/// 
/// # Returns
/// 
/// Returns the modified `http::response::Builder` with all necessary CORS headers added.
/// The builder can then be used to construct the final HTTP response.
/// 
/// # CORS Headers Added
/// 
/// - `Access-Control-Allow-Origin`: Set to the request origin if whitelisted, otherwise "*".
/// - `Access-Control-Allow-Methods`: "GET, POST, OPTIONS".
/// - `Access-Control-Allow-Headers`: Includes IronShield-specific headers for PoW challenges.
/// - `Access-Control-Allow-Credentials`: "true" only for whitelisted origins.
/// - `Vary`: "Origin" for proper caching behavior.
/// 
/// # Examples 
/// 
/// ```rust
/// use axum::http::{Response, StatusCode, HeaderMap};
/// 
/// // Add CORS headers to a success response
/// let response = add_cors_headers(
///     Response::builder().status(StatusCode::OK),
///     &request_headers
/// ).body("Success".into())?;
/// 
/// // Add CORS headers to an error response  
/// let error_response = add_cors_headers(
///     Response::builder().status(StatusCode::FORBIDDEN),
///     &request_headers
/// ).body("Access denied".into())?;
/// ```
pub fn add_cors_headers(
    builder: http::response::Builder,
    request_headers: &http::HeaderMap,
) -> http::response::Builder {
    let mut builder: http::response::Builder = builder;

    // Get the origin header from the request.
    let origin: &str = request_headers
        .get(header::ORIGIN)
        .and_then(|v: &http::HeaderValue| v.to_str().ok())
        .unwrap_or("");

    // Check if the origin is allowed.
    let is_allowed_origin: bool = ALLOWED_ORIGINS.contains(&origin) || origin.is_empty();

    // Set the appropriate Access-Control-Allow-Origin header.
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    } else {
        // Fallback to wildcard if no origin is specified, or it's not in our allowed list.
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    }

    // Add other CORS headers.
    builder = builder
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, X-IronShield-Challenge, X-IronShield-Nonce, X-IronShield-Timestamp, X-IronShield-Difficulty, X-Ironshield-Token")
        .header(header::VARY, "Origin"); // Important for caching.

    // Only add a credential header if we have a specific origin (not wildcard).
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
    }

    builder
}