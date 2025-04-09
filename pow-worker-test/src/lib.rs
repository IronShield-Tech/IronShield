use axum::{
    body::{self},
    http::{Request, Response, StatusCode, header, Method as AxumMethod},
    routing::get,
    Router,
};
use tower_service::Service;
use worker::*;
use sha2::{Sha256, Digest};
use hex;
use chrono::{Utc, Duration, DateTime};

// --- Constants ---
const POW_DIFFICULTY: usize = 4; // Number of leading zeros required in the hash
const CHALLENGE_HEADER: &str = "X-IronShield-Challenge";
const NONCE_HEADER: &str = "X-IronShield-Nonce";
const TIMESTAMP_HEADER: &str = "X-IronShield-Timestamp";
const MAX_CHALLENGE_AGE_SECONDS: i64 = 60; // How long a challenge is valid

// Simple placeholder for successful access
async fn protected_content() -> &'static str {
    "Access Granted: Checksum Approved."
}

// Function to generate the challenge page
fn generate_challenge_page(challenge_string: &str, timestamp: DateTime<Utc>) -> Result<Response<body::Body>> {
    console_log!("Issuing checksum challenge...");

    // Simple HTML with embedded JS for the challenge
    // language=HTML
    let html_content = format!(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>IronShield Challenge</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body {{ font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background-color: #f0f0f0; }}
                .container {{ text-align: center; padding: 20px; border-radius: 8px; background-color: #fff; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
                #status {{ margin-top: 15px; font-weight: bold; }}
                progress {{ width: 100%; margin-top: 10px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h2>Security Check</h2>
                <p>Please wait while we verify your connection. This may take a few seconds.</p>
                <progress id="progress" max="100" value="0"></progress>
                <div id="status">Initializing...</div>
            </div>

            <script>
                async function solveChallenge() {{
                    const challenge = "{challenge}";
                    const timestamp = "{timestamp}"; // ISO 8601 format
                    const difficulty = {difficulty};
                    const targetPrefix = "0".repeat(difficulty);
                    const statusDiv = document.getElementById('status');
                    const progressBar = document.getElementById('progress');
                    statusDiv.textContent = 'Solving challenge...';

                    let nonce = 0;
                    let hash = '';
                    let attempts = 0;
                    const maxAttempts = 1000000; // Safety break
                    const startTime = Date.now();

                    while (nonce < maxAttempts) {{
                        const dataToHash = `${{challenge}}:${{nonce}}`;
                        const encoder = new TextEncoder();
                        const data = encoder.encode(dataToHash);
                        const hashBuffer = await crypto.subtle.digest('SHA-256', data);
                        const hashArray = Array.from(new Uint8Array(hashBuffer));
                        hash = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');

                        if (hash.startsWith(targetPrefix)) {{
                            statusDiv.textContent = `Challenge solved! (Nonce: ${{nonce}}, Hash: ${{hash.substring(0, 10)}}...)`;
                            progressBar.value = 100;
                            // Send the solution back via headers
                            fetch(window.location.href, {{
                                method: 'GET', // Or match the original request method if needed
                                headers: {{
                                    '{challenge_header}': challenge,
                                    '{nonce_header}': nonce.toString(),
                                    '{timestamp_header}': timestamp
                                }}
                            }})
                            .then(response => {{
                                if (response.ok) {{
                                    // Replace current page content with the response from the server
                                    return response.text().then(html => {{
                                        document.open();
                                        document.write(html);
                                        document.close();
                                    }});
                                }} else {{
                                    statusDiv.textContent = `Verification failed (Status: ${{response.status}}). Please try refreshing.`;
                                    progressBar.value = 0;
                                }}
                            }})
                            .catch(error => {{
                                console.error('Error sending verification:', error);
                                statusDiv.textContent = 'Error sending verification. Please check console.';
                                progressBar.value = 0;
                            }});
                            return; // Exit loop
                        }}

                        nonce++;
                        attempts++;

                        // Update progress roughly - very approximate
                        if (attempts % 1000 === 0) {{
                            // Avoid getting stuck if it takes too long
                            if (Date.now() - startTime > 30000) {{ // 30 seconds timeout
                                statusDiv.textContent = 'Challenge timed out. Please refresh.';
                                progressBar.value = 0;
                                return;
                            }}
                        progressBar.value = Math.min(99, (attempts / 50000) * 100); // Heuristic progress
                        statusDiv.textContent = `Solving challenge... Attempt: ${{attempts}}`;
                        await new Promise(resolve => setTimeout(resolve, 0)); // Yield to browser
                        }}
                    }}

                    statusDiv.textContent = 'Could not solve the challenge within limits. Please refresh.';
                    progressBar.value = 0;
                }}

                // Start solving immediately
                solveChallenge();
            </script>
        </body>
        </html>
        "#,
        challenge = challenge_string,
        timestamp = timestamp.to_rfc3339(), // Use standard format
        difficulty = POW_DIFFICULTY,
        challenge_header = CHALLENGE_HEADER,
        nonce_header = NONCE_HEADER,
        timestamp_header = TIMESTAMP_HEADER
    );

    Response::builder()
        .status(StatusCode::OK) // Use OK for the challenge page itself
        .header(header::CONTENT_TYPE, "text/html")
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

    match (challenge_opt, nonce_opt, timestamp_opt) {
        (Some(challenge), Some(nonce_str), Some(timestamp_str)) => {
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


            // 2. Verify nonce format
            match nonce_str.parse::<u64>() {
                Ok(nonce) => {
                    // 3. Recompute hash
                    let data_to_hash = format!("{}:{}", challenge, nonce);
                    let mut hasher = Sha256::new();
                    hasher.update(data_to_hash.as_bytes());
                    let hash_bytes = hasher.finalize();
                    let hash_hex = hex::encode(hash_bytes);

                    // 4. Check difficulty
                    let target_prefix = "0".repeat(POW_DIFFICULTY);
                    if hash_hex.starts_with(&target_prefix) {
                        console_log!("Checksum verification successful (Hash: {}...).", &hash_hex[..8]);
                        true
                    } else {
                        console_log!("Checksum verification failed (Hash: {}...).", &hash_hex[..8]);
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

// Main Worker entry point
#[event(fetch)]
pub async fn main(req: Request<worker::Body>, _env: Env, _ctx: worker::Context) -> Result<Response<body::Body>> {
    // Optionally set panic hook for better error messages in browser console
    utils::set_panic_hook();

    let headers = req.headers();
    let has_pow_headers = headers.contains_key(CHALLENGE_HEADER)
        && headers.contains_key(NONCE_HEADER)
        && headers.contains_key(TIMESTAMP_HEADER);

    if req.method() == &AxumMethod::GET && has_pow_headers {
        // Attempt verification
        if verify_solution(&req) {
            // If verified, proceed to the actual Axum router/handler
            let mut router = Router::new().route("/", get(protected_content));
            router.call(req).await
                  .map_err(|_| worker::Error::RustError("Infallible error from router.call".to_string()))
        } else {
             // Verification failed
             Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(body::Body::from("Checksum verification failed."))
                .map_err(|e| Error::RustError(format!("Failed to build forbidden response: {}", e)))
        }
    } else if req.method() == &AxumMethod::GET {
         // Issue challenge - generate a unique challenge string (timestamp + maybe IP/randomness)
         let now = Utc::now();
         // TODO: Include client IP or other info if available and desired
         // let client_ip = req.headers().get("CF-Connecting-IP").map(|v| v.to_str().unwrap_or("unknown")).unwrap_or("unknown");
         // let challenge_string = format!("{}:{}", client_ip, now.timestamp());
         let challenge_string = format!("challenge_{}", now.timestamp_millis()); // Simple timestamp-based challenge for now

         generate_challenge_page(&challenge_string, now)
    } else {
         // For non-GET requests or other scenarios, return Method Not Allowed or handle differently
         Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(body::Body::from("Method Not Allowed"))
            .map_err(|e| Error::RustError(format!("Failed to build method not allowed response: {}", e)))
    }
}

// Need this utility function if not already present from template
mod utils {
     pub fn set_panic_hook() {
         // When the `console_error_panic_hook` feature is enabled, we can call the
         // `set_panic_hook` function at least once during initialization, and then
         // we will get better error messages if our code ever panics.
         //
         // For more details see
         // https://github.com/rustwasm/console_error_panic_hook#readme
         console_error_panic_hook::set_once();
     }
 }
