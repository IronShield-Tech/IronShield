use ironshield_core;
use wasm_bindgen::prelude::*;
use hex;
use serde_json;

/// Support for threading in WASM
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
use wasm_bindgen_futures::JsFuture;

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

/// JavaScript-compatible solution result for IronShield challenges
#[derive(serde::Serialize)]
struct IronShieldSolutionResult {
    /// String representation of the solution nonce to avoid JavaScript BigInt precision issues.
    solution_str: String,
    /// Original numeric value for compatibility.
    solution: i64,
    /// Challenge signature preserved from the original challenge.
    challenge_signature_hex: String,
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

/// Creates a standardized IronShield solution result from core library output.
fn create_ironshield_solution_result(response: ironshield_core::IronShieldChallengeResponse) -> IronShieldSolutionResult {
    IronShieldSolutionResult {
        solution_str: response.solution.to_string(),
        solution: response.solution,
        challenge_signature_hex: hex::encode(response.challenge_signature),
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
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    // Create a shared memory thread pool for parallel processing
    let promise = init_thread_pool(num_threads);
    JsFuture::from(promise).await.map(|_| ()).map_err(|e| e)
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
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
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
    #[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
    return true;

    #[cfg(not(all(feature = "parallel", not(feature = "no-parallel"))))]
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

/// Solves IronShield proof-of-work challenges using single-threaded computation.
/// 
/// # Arguments
/// * `challenge_json` - JSON string containing the IronShieldChallenge
/// 
/// # Returns
/// JavaScript object with solution nonce and challenge signature, or error message.
#[wasm_bindgen]
pub fn solve_ironshield_challenge(challenge_json: &str) -> Result<JsValue, JsValue> {
    // Enable better error messages in browser console
    console_error_panic_hook::set_once();

    // Parse the challenge from JSON
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e| JsValue::from_str(&format!("Error parsing challenge JSON: {}", e)))?;

    // Find valid nonce using single-threaded algorithm
    let response = ironshield_core::find_solution_single_threaded(&challenge)
        .map_err(|e| JsValue::from_str(&format!("Error solving IronShield challenge: {}", e)))?;

    // Package result for JavaScript consumption
    let solution_result = create_ironshield_solution_result(response);

    // Convert Rust struct to JavaScript object
    serde_wasm_bindgen::to_value(&solution_result)
        .map_err(|err| JsValue::from_str(&format!("Error serializing IronShield result: {:?}", err)))
}

/// Solves IronShield proof-of-work challenges using optimized multi-threaded computation.
/// 
/// This function provides the fastest possible PoW solving by distributing the work
/// across all available CPU cores with optimal load balancing and early termination.
/// 
/// # Arguments
/// * `challenge_json` - JSON string containing the IronShieldChallenge
/// 
/// # Returns
/// JavaScript object with solution nonce and challenge signature, or error message.
/// 
/// # Performance
/// - **Multi-core scaling**: Near-linear performance improvement with CPU core count
/// - **WASM optimization**: Fully compatible with SharedArrayBuffer and Web Workers
/// - **Early termination**: Stops all threads immediately when solution is found
/// - **Memory efficient**: Minimal overhead compared to single-threaded version
/// 
/// # Note
/// Requires thread pool initialization via `init_threads()` first when using parallel features
#[wasm_bindgen]
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
pub fn solve_ironshield_challenge_multi_threaded(challenge_json: &str) -> Result<JsValue, JsValue> {
    // Enable better error messages in browser console
    console_error_panic_hook::set_once();

    // Parse the challenge from JSON
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e| JsValue::from_str(&format!("Error parsing challenge JSON: {}", e)))?;

    // Find valid nonce using optimized multi-threaded algorithm
    let response = ironshield_core::find_solution_multi_threaded(&challenge)
        .map_err(|e| JsValue::from_str(&format!("Error solving IronShield challenge with multi-threading: {}", e)))?;

    // Package result for JavaScript consumption
    let solution_result = create_ironshield_solution_result(response);

    // Convert Rust struct to JavaScript object
    serde_wasm_bindgen::to_value(&solution_result)
        .map_err(|err| JsValue::from_str(&format!("Error serializing multi-threaded IronShield result: {:?}", err)))
}

/// Verifies an IronShield proof-of-work solution without recomputing.
/// 
/// # Arguments
/// * `challenge_json` - JSON string containing the original IronShieldChallenge.
/// * `solution_nonce` - Proposed solution nonce as i64.
/// 
/// # Returns
/// `true` if the solution is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_ironshield_solution(challenge_json: &str, solution_nonce: i64) -> Result<bool, JsValue> {
    // Parse the challenge from JSON
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e| JsValue::from_str(&format!("Error parsing challenge JSON for verification: {}", e)))?;

    // Verify the solution
    let is_valid = ironshield_core::verify_ironshield_solution(&challenge, solution_nonce);
    Ok(is_valid)
}