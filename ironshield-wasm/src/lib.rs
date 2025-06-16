#[allow(unused_imports)]
use hex;
use ironshield_core;
#[allow(unused_imports)]
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

// Support for threading in WASM
#[cfg(feature = "parallel")]
use wasm_bindgen_rayon::init_thread_pool;

/// Export functions to JavaScript.
#[wasm_bindgen]
pub fn solve_pow_challenge(challenge: &str, difficulty: usize) -> Result<JsValue, JsValue> {
    // Set panic hook for better error messages.
    console_error_panic_hook::set_once();

    // Get the required number of leading zeros.
    #[allow(unused_variables)]
    let target_prefix: String = "0".repeat(difficulty);

    // Solve the challenge using our core library.
    let (nonce, hash) = ironshield_core::find_solution(challenge, difficulty)
        .map_err(|e: String| JsValue::from_str(&format!("Error solving challenge: {}", e)))?;

    // Create the solution result.
    let solution_result: SolutionResult = SolutionResult {
        nonce_str: nonce.to_string(), // Convert to string to avoid BigInt issues.
        nonce,                        // Keep the u64 version for compatibility.
        hash: hash.clone(),
        hash_prefix: hash[..10].to_string(),
    };

    // Use serde-wasm-bindgen to convert to JsValue.
    match serde_wasm_bindgen::to_value(&solution_result) {
        Ok(js_value) => Ok(js_value),
        Err(err) => Err(JsValue::from_str(&format!(
            "Error serializing result: {:?}",
            err
        ))),
    }
}

/// Initialize the thread pool for parallel processing
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    // Initialize the thread pool with the specified number of threads
    init_thread_pool(num_threads)
        .map_err(|e| JsValue::from_str(&format!("Error initializing thread pool: {}", e)))?;
    Ok(())
}

/// Solve the challenge using parallel processing
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub fn solve_pow_challenge_parallel(
    challenge: &str,
    difficulty: usize,
    num_threads: usize,
) -> Result<JsValue, JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    // Solve the challenge using parallel processing
    let (nonce, hash) = ironshield_core::find_solution_parallel(challenge, difficulty, num_threads)
        .map_err(|e| JsValue::from_str(&format!("Error solving challenge in parallel: {}", e)))?;

    // Create the solution result
    let solution_result = SolutionResult {
        nonce_str: nonce.to_string(),
        nonce,
        hash: hash.clone(),
        hash_prefix: hash[..10].to_string(),
    };

    // Use serde-wasm-bindgen to convert to JsValue
    match serde_wasm_bindgen::to_value(&solution_result) {
        Ok(js_value) => Ok(js_value),
        Err(err) => Err(JsValue::from_str(&format!(
            "Error serializing parallel result: {:?}",
            err
        ))),
    }
}

// Add a function to detect if threads are supported
#[wasm_bindgen]
pub fn are_threads_supported() -> bool {
    #[cfg(feature = "parallel")]
    return true;

    #[cfg(not(feature = "parallel"))]
    return false;
}

// Add a simple validation function that accepts a string nonce
#[wasm_bindgen]
pub fn verify_pow_solution(challenge: &str, nonce_value: &str, difficulty: usize) -> bool {
    ironshield_core::verify_solution(challenge, nonce_value, difficulty)
}

// Add a logging helper for debugging
#[wasm_bindgen]
pub fn console_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

// Data structure to return the solution
#[derive(serde::Serialize)]
struct SolutionResult {
    nonce_str: String, // String representation for JavaScript
    nonce: u64,        // Original u64 value (may cause issues in JS)
    hash: String,
    hash_prefix: String,
}
