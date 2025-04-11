use wasm_bindgen::prelude::*;
use sha2::{Sha256, Digest};
use hex;

// Export functions to JavaScript
#[wasm_bindgen]
pub fn solve_pow_challenge(challenge: &str, difficulty: usize) -> Result<JsValue, JsValue> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Get required number of leading zeros
    let target_prefix = "0".repeat(difficulty);
    
    // Solve the challenge
    let (nonce, hash) = find_solution(challenge, &target_prefix)
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

// Internal function to find a solution
fn find_solution(challenge: &str, target_prefix: &str) -> Result<(u64, String), String> {
    let max_attempts = 10000000;
    
    for nonce in 0..max_attempts {
        let data_to_hash = format!("{}:{}", challenge, nonce);
        let mut hasher = Sha256::new();
        hasher.update(data_to_hash.as_bytes());
        let hash_bytes = hasher.finalize();
        let hash = hex::encode(hash_bytes);
        
        if hash.starts_with(target_prefix) {
            return Ok((nonce, hash));
        }
        
        // Occasionally yield to avoid blocking UI
        if nonce % 1000 == 0 {
            // In real implementation, we'd use js_sys::Promise here
            // but for simplicity we'll just continue
        }
    }
    
    Err("Could not find solution within attempt limit".into())
}

// Data structure to return the solution
#[derive(serde::Serialize)]
struct SolutionResult {
    nonce_str: String, // String representation for JavaScript
    nonce: u64,        // Original u64 value (may cause issues in JS)
    hash: String,
    hash_prefix: String,
}

// Add a simple validation function that accepts a string nonce
#[wasm_bindgen]
pub fn verify_pow_solution(challenge: &str, nonce_value: &str, difficulty: usize) -> bool {
    // Parse the nonce from string
    match nonce_value.parse::<u64>() {
        Ok(nonce) => {
            let target_prefix = "0".repeat(difficulty);
            let data_to_hash = format!("{}:{}", challenge, nonce);
            
            let mut hasher = Sha256::new();
            hasher.update(data_to_hash.as_bytes());
            let hash_bytes = hasher.finalize();
            let hash = hex::encode(hash_bytes);
            
            hash.starts_with(&target_prefix)
        },
        Err(_) => false
    }
}

// Add a logging helper for debugging
#[wasm_bindgen]
pub fn console_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
} 