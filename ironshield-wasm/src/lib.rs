use wasm_bindgen::prelude::*;
use ironshield_core;
#[allow(unused_imports)]
use sha2::{Sha256, Digest};
#[allow(unused_imports)]
use hex;

// Export functions to JavaScript
#[wasm_bindgen]
pub fn solve_pow_challenge(challenge: &str, difficulty: usize) -> Result<JsValue, JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Get required number of leading zeros
    #[allow(unused_variables)]
    let target_prefix = "0".repeat(difficulty);
    
    // Solve the challenge using our core library
    let (nonce, hash) = ironshield_core::find_solution(challenge, difficulty)
        .map_err(|e| JsValue::from_str(&format!("Error solving challenge: {}", e)))?;
    
    // Create the solution result
    let solution_result = SolutionResult {
        nonce_str: nonce.to_string(), // Convert to string to avoid BigInt issues
        nonce,                         // Keep the u64 version for compatibility
        hash: hash.clone(),
        hash_prefix: hash[..10].to_string(),
    };
    
    // Use serde-wasm-bindgen to convert to JsValue
    match serde_wasm_bindgen::to_value(&solution_result) {
        Ok(js_value) => Ok(js_value),
        Err(err) => Err(JsValue::from_str(&format!("Error serializing result: {:?}", err))),
    }
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