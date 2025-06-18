use http::header;

pub const ALLOWED_ORIGINS: [&str; 3] = [
    "http://localhost:8787",
    "https://skip.ironshield.cloud",
    "https://ironshield.cloud",
];

/// 
/// 
/// # Arguments 
/// 
/// * `builder`: 
/// * `request_headers`: 
/// 
/// returns: Builder 
/// 
/// # Examples 
/// 
/// ```
/// 
/// ```
// Helper function to handle CORS for responses
pub fn add_cors_headers(
    builder: http::response::Builder,
    request_headers: &http::HeaderMap,
) -> http::response::Builder {
    let mut builder: http::response::Builder = builder;

    // Get the origin header from the request
    let origin: &str = request_headers
        .get(header::ORIGIN)
        .and_then(|v: &http::HeaderValue| v.to_str().ok())
        .unwrap_or("");

    // Check if the origin is allowed
    let is_allowed_origin: bool = crate::ALLOWED_ORIGINS.contains(&origin) || origin.is_empty();

    // Set the appropriate Access-Control-Allow-Origin header
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    } else {
        // Fallback to wildcard if no origin is specified, or it's not in our allowed list
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    }

    // Add other CORS headers
    builder = builder
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, POST, OPTIONS")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Content-Type, X-IronShield-Challenge, X-IronShield-Nonce, X-IronShield-Timestamp, X-IronShield-Difficulty, X-Ironshield-Token")
        .header(header::VARY, "Origin"); // Important for caching

    // Only add a credential header if we have a specific origin (not wildcard)
    if is_allowed_origin && !origin.is_empty() {
        builder = builder.header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");
    }

    builder
}