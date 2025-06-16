//! # IronShield WebAssembly Module
//! 
//! Interface for solving proof-of-work challenges 
//! in a WebAssembly environment.

use wasm_bindgen::prelude::*;
use serde::Serialize;

#[cfg(feature = "parallel")]
use wasm_bindgen_rayon::init_thread_pool;

/// Result structure for proof-of-work solutions.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionResult {
    /// String representation of nonce (safe for JavaScript)
    nonce: String,
    /// The computed hash.
    hash: String,
    /// First 10 characters of the hash for quick verification.
    hash_prefix: String,
}

impl SolutionResult {
    fn new(nonce: u64, hash: String) -> Self {
        let hash_prefix = if hash.len() >= 10 {
            hash[..10].to_string()
        } else {
            hash.clone()
        };

        Self {
            nonce: nonce.to_string(),
            hash,
            hash_prefix,
        }
    }
}

/// Convert errors to JsValue with consistent formatting.
fn map_error(context: &str, error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&format!("{}: {}", context, error))
}

/// Serialize a result to JsValue with error handling.
fn serialize_result(result: &SolutionResult) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(result)
        .map_err(|e| map_error("Serialization failed", e))
}

/// Solve a proof-of-work challenge.
/// 
/// # Arguments
/// * `challenge` - The challenge string to solve.
/// * `difficulty` - Number of leading zeros required in the hash.
/// 
/// # Returns
/// A JavaScript object containing the solution details.
#[wasm_bindgen]
pub fn solve_pow_challenge(challenge: &str, difficulty: usize) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let (nonce, hash) = ironshield_core::find_solution(challenge, difficulty)
        .map_err(|e| map_error("Challenge solving failed", e))?;

    let result = SolutionResult::new(nonce, hash);
    serialize_result(&result)
}

/// Initialize thread pool for parallel processing.
/// 
/// # Arguments
/// * `num_threads` - Number of threads to initialize.
/// 
/// # Note
/// Only available when compiled with the "parallel" feature.
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    init_thread_pool(num_threads)
        .map_err(|e| map_error("Thread pool initialization failed", e))
}

/// Solve a proof-of-work challenge using parallel processing
/// 
/// # Arguments
/// * `challenge` - The challenge string to solve
/// * `difficulty` - Number of leading zeros required in the hash
/// * `num_threads` - Number of threads to use for parallel processing
/// 
/// # Returns
/// A JavaScript object containing the solution details
/// 
/// # Note
/// Only available when compiled with the "parallel" feature
#[wasm_bindgen]
#[cfg(feature = "parallel")]
pub fn solve_pow_challenge_parallel(
    challenge: &str,
    difficulty: usize,
    num_threads: usize,
) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let (nonce, hash) = ironshield_core::find_solution_parallel(challenge, difficulty, num_threads)
        .map_err(|e| map_error("Parallel challenge solving failed", e))?;

    let result = SolutionResult::new(nonce, hash);
    serialize_result(&result)
}

/// Check if parallel processing is supported.
/// 
/// # Returns
/// `true` if compiled with parallel support, `false` otherwise.
#[wasm_bindgen]
pub fn are_threads_supported() -> bool {
    cfg!(feature = "parallel")
}

/// Verify a proof-of-work solution.
/// 
/// # Arguments
/// * `challenge` - The original challenge string.
/// * `nonce_value` - The nonce value as a string.
/// * `difficulty` - The required difficulty level.
/// 
/// # Returns
/// `true` if the solution is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_pow_solution(challenge: &str, nonce_value: &str, difficulty: usize) -> bool {
    ironshield_core::verify_solution(challenge, nonce_value, difficulty)
}

/// Log a message to the browser console.
/// 
/// # Arguments
/// * `message` - The message to log.
#[wasm_bindgen]
pub fn console_log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}