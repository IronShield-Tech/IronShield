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

// Function to generate the challenge page that uses WebAssembly
fn generate_challenge_page(challenge_string: &str, timestamp: DateTime<Utc>) -> Result<Response<body::Body>> {
    console_log!("Issuing WebAssembly challenge...");

    // Simple HTML with WebAssembly loader
    // language=HTML
    let html_content = format!(r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>IronShield Challenge</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <style>
                body {{ font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background-color: #f0f0f0; }}
                .container {{ text-align: center; padding: 20px; border-radius: 8px; background-color: #fff; box-shadow: 0 2px 4px rgba(0,0,0,0.1); max-width: 500px; width: 90%; }}
                #status {{ margin-top: 15px; font-weight: bold; }}
                progress {{ width: 100%; margin-top: 10px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h2>Security Check</h2>
                <p>Please wait while we verify your connection. This may take a few seconds.</p>
                <progress id="progress" max="100" value="0"></progress>
                <div id="status">Initializing WebAssembly...</div>
            </div>

            <script type="module">
                // Challenge parameters
                const challenge = "{challenge}";
                const timestamp = "{timestamp}";
                const difficulty = {difficulty};
                
                const statusDiv = document.getElementById('status');
                const progressBar = document.getElementById('progress');
                
                // Function to load and initialize the WebAssembly module
                async function initWasm() {{
                    try {{
                        statusDiv.textContent = 'Loading WebAssembly module...';
                        
                        // Import the WebAssembly module using the JavaScript bindings
                        const wasmModule = await import('/pow_wasm.js');
                        
                        // Initialize the module (this will fetch the .wasm file)
                        await wasmModule.default();
                        
                        // Now we can use the exported functions
                        statusDiv.textContent = 'Solving challenge...';
                        
                        // Track progress
                        let progressInterval = setInterval(() => {{
                            // Just a UI indicator since we can't track actual progress from Wasm
                            const currentValue = progressBar.value;
                            if (currentValue < 90) {{
                                progressBar.value = currentValue + 1;
                            }}
                        }}, 200);
                        
                        // Start solving the challenge
                        try {{
                            const startTime = Date.now();
                            
                            // Call the Rust function via the JavaScript bindings
                            const result = await wasmModule.solve_pow_challenge(challenge, difficulty);
                            const duration = Date.now() - startTime;
                            
                            clearInterval(progressInterval);
                            progressBar.value = 100;
                            
                            // Verify the solution with our Rust code
                            const isValid = wasmModule.verify_pow_solution(challenge, result.nonce_str, difficulty);
                            
                            if (isValid) {{
                                statusDiv.textContent = `Challenge solved! (Nonce: ${{result.nonce_str}}, Hash: ${{result.hash_prefix}}...)`;
                                
                                // Send the solution back via headers
                                fetch(window.location.href, {{
                                    method: 'GET',
                                    headers: {{
                                        '{challenge_header}': challenge,
                                        '{nonce_header}': result.nonce_str,
                                        '{timestamp_header}': timestamp
                                    }}
                                }})
                                .then(response => {{
                                    if (response.ok) {{
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
                            }} else {{
                                statusDiv.textContent = 'Invalid solution generated. Please refresh.';
                                progressBar.value = 0;
                            }}
                        }} catch (error) {{
                            clearInterval(progressInterval);
                            console.error('Error solving challenge:', error);
                            statusDiv.textContent = `Error solving challenge: ${{error.message}}`;
                            progressBar.value = 0;
                        }}
                    }} catch (error) {{
                        console.error('Error initializing WebAssembly:', error);
                        statusDiv.textContent = `Failed to initialize WebAssembly: ${{error.message}}`;
                        progressBar.value = 0;
                    }}
                }}
                
                // Start WebAssembly initialization
                initWasm();
            </script>
        </body>
        </html>
        "#,
        challenge = challenge_string,
        timestamp = timestamp.to_rfc3339(),
        difficulty = POW_DIFFICULTY,
        challenge_header = CHALLENGE_HEADER,
        nonce_header = NONCE_HEADER,
        timestamp_header = TIMESTAMP_HEADER
    );

    Response::builder()
        .status(StatusCode::OK)
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
        && headers.contains_key(TIMESTAMP_HEADER);

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
