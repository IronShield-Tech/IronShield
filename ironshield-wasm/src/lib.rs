use ironshield_core;
use wasm_bindgen::prelude::*;

/// Support for threading in WASM
#[cfg(feature = "parallel")]
use wasm_bindgen_rayon::init_thread_pool;

/// JavaScript-compatible solution result containing proof-of-work data
#[derive(serde::Serialize)]
struct SolutionResult {
    /// String representation to avoid JavaScript BigInt precision issues.
    nonce_str: String,
    /// Original numeric value for compatibility.
    nonce: u64,
    /// Complete SHA-256 hash as a hex string.
    hash: String,
    /// First 10 characters for quick visual verification.
    hash_prefix: String,
}

/// Creates a standardized solution result from core library output.
fn create_solution_result(nonce: u64, hash: String) -> SolutionResult {
    SolutionResult {
        nonce_str: nonce.to_string(),
        nonce,
        hash: hash.clone(),
        hash_prefix: hash[..10].to_string(),
    }
}

/// Solves proof-of-work challenges using single-threaded computation.
/// 
/// # Arguments
/// * `challenge` - Base string to append nonce to for hashing.
/// * `difficulty` - Number of leading zeros required in hash.
/// 
/// # Returns
/// JavaScript object with nonce and hash, or error message.
#[wasm_bindgen]
pub fn solve_pow_challenge(challenge: &str, difficulty: usize) -> Result<JsValue, JsValue> {
    // Enable better error messages in browser console
    console_error_panic_hook::set_once();

    // Find valid nonce using brute force search
    let (nonce, hash) = ironshield_core::find_solution(challenge, difficulty)
        .map_err(|e| JsValue::from_str(&format!("Error solving challenge: {}", e)))?;

    // Package result for JavaScript consumption
    let solution_result = create_solution_result(nonce, hash);

    // Convert Rust struct to JavaScript object
    serde_wasm_bindgen::to_value(&solution_result)
        .map_err(|err| JsValue::from_str(&format!("Error serializing result: {:?}", err)))
}

/// Initializes WebAssembly thread pool for parallel proof-of-work
/// 
/// # Arguments
/// * `num_threads` - Number of worker threads to spawn
/// 
/// # Note
/// Only available when compiled with a "parallel" feature flag
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    // Create a shared memory thread pool for parallel processing
    init_thread_pool(num_threads)
        .map_err(|e| JsValue::from_str(&format!("Error initializing thread pool: {}", e)))
}

/// Solves proof-of-work challenges using multithreaded parallel computation
/// 
/// # Arguments
/// * `challenge` - Base string to append nonce to for hashing
/// * `difficulty` - Number of leading zeros required in hash
/// * `num_threads` - Number of parallel workers to use
/// 
/// # Returns
/// JavaScript object with nonce and hash, or error message
/// 
/// # Note
/// Requires thread pool initialization via `init_threads()` first
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub fn solve_pow_challenge_parallel(
    challenge: &str,
    difficulty: usize,
    num_threads: usize,
) -> Result<JsValue, JsValue> {
    // Enable better error messages in browser console
    console_error_panic_hook::set_once();

    // Distribute nonce search across multiple threads
    let (nonce, hash) = ironshield_core::find_solution_parallel(challenge, difficulty, num_threads)
        .map_err(|e| JsValue::from_str(&format!("Error solving challenge in parallel: {}", e)))?;

    // Package result for JavaScript consumption
    let solution_result = create_solution_result(nonce, hash);

    // Convert Rust struct to JavaScript object
    serde_wasm_bindgen::to_value(&solution_result)
        .map_err(|err| JsValue::from_str(&format!("Error serializing parallel result: {:?}", err)))
}

/// Checks if parallel processing is available in the current build.
/// 
/// # Returns
/// `true` if compiled with a "parallel" feature, `false` otherwise.
#[wasm_bindgen]
pub fn are_threads_supported() -> bool {
    #[cfg(feature = "parallel")]
    return true;

    #[cfg(not(feature = "parallel"))]
    return false;
}

/// Verifies a proof-of-work solution without recomputing.
/// 
/// # Arguments
/// * `challenge` - Original challenge string.
/// * `nonce_value` - Proposed solution nonce as string.
/// * `difficulty` - Required number of leading zeros.
/// 
/// # Returns
/// `true` if a solution is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_pow_solution(challenge: &str, nonce_value: &str, difficulty: usize) -> bool {
    ironshield_core::verify_solution(challenge, nonce_value, difficulty)
}

/// Outputs debug message to browser console.
/// 
/// # Arguments
/// * `s` - Message string to log.
/// 
/// # Note
/// Useful for debugging WASM execution from JavaScript.
#[wasm_bindgen]
pub fn console_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}