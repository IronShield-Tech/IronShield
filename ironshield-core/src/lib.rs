//! # Core functionality for the IronShield proof-of-work system.
//!
//! This module provides SHA-256 based proof-of-work verification functionality.
//!
//! The proof-of-work algorithm finds a nonce value that, when combined with a challenge string,
//! produces an SHA-256 hash starting with a specified number of leading zeros (difficulty).

use hex;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use sha2::{Digest, Sha256};

/// Maximum number of nonce values to try before giving up.
/// This prevents infinite loops and excessive computation.
const MAX_ATTEMPTS:   u64 = 10_000_000;
/// Size of each parallel processing chunk to balance memory 
/// usage and performance.
/// Used in `find_solution_parallel` to divide work between threads.
const CHUNK_SIZE:   usize = 10_000;

// const YIELD_INTERVAL: u64 = 1000;

/// Finds and provides nonce value that produces a hash with 
/// required difficulty.
///
/// Iterates through nonce values sequentially until finding 
/// one that produces a hash starting with `difficulty` numbers
/// of leading zeros when combined with the challenge string. 
/// 
/// # Arguments
/// * `challenge` - The challenge string to hash (typically contains timestamp and user data).
/// * `difficulty` - Number of leading zeros required in the resulting hash.
///
/// # Returns
/// * `Ok((nonce, hash))` - The successful nonce and its corresponding hash.
/// * `Err(String)` - Error message if no solution is found within `MAX_ATTEMPTS`.
pub fn find_solution(challenge: &str, difficulty: usize) -> Result<(u64, String), String> {
    let target_prefix = "0".repeat(difficulty);

    for nonce in 0..MAX_ATTEMPTS {
        let hash = calculate_hash(challenge, nonce);

        if hash.starts_with(&target_prefix) {
            return Ok((nonce, hash));
        }

//      // Occasionally yield to avoid blocking UI.
//      if nonce % YIELD_INTERVAL == 0 {
//          // In real implementation, we'd use js_sys::Promise here.
//      }
    }

    Err("Could not find solution within attempt limit".into())
}

#[cfg(feature = "parallel")]
pub fn find_solution_parallel(
    challenge: &str,
    difficulty: usize,
    num_threads: usize,
) -> Result<(u64, String), String> {
    let target_prefix = "0".repeat(difficulty);

    let result = (0..MAX_ATTEMPTS)
        .step_by(num_threads)
        .collect::<Vec<u64>>()
        .par_chunks(CHUNK_SIZE)
        .find_map_any(|chunk| {
            chunk.iter().find_map(|&start_nonce| {
                (0..num_threads).find_map(|thread_offset| {
                    let nonce = start_nonce + thread_offset as u64;
                    let hash = calculate_hash(challenge, nonce);

                    if hash.starts_with(&target_prefix) {
                        Some((nonce, hash))
                    } else {
                        None
                    }
                })
            })
        });

    result.ok_or_else(|| "Could not find solution within attempt limit".into())
}

/// Computes an SHA-256 hash of the challenge string
/// combined with a nonce value. 
///
/// Creates a hash input in the format `"{challenge}:{nonce}"`
/// and returns the hexadecimal representation of the SHA-256 
/// digest. This is the core hashing function used by both
/// mining and verification processes. 
///
/// # Arguments
/// * `challenge` - The challenge string (contains proof-of-work parameters).
/// * `nonce` - The nonce value to append to the challenge.
///
/// # Returns
/// * Lowercase hexadecimal string representation of the SHA-256 hash.
pub fn calculate_hash(challenge: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", challenge, nonce).as_bytes());
    hex::encode(hasher.finalize())
}

/// Verifies that a given nonce produces a valid proof-of-work solution.
///
/// Parses the nonce string, recalculates the hash, and checks if it meets
/// the difficulty requirement (correct number of leading zeros). Used to
/// validate solutions received from clients.
///
/// # Arguments
/// * `challenge` - The original challenge string used for mining.
/// * `nonce_str` - String representation of the nonce to verify.
/// * `difficulty` - Required number of leading zeros in the hash.
///
/// # Returns
/// * `true` - If the nonce produces a valid hash meeting the difficulty requirement.
/// * `false` - If the nonce is invalid or doesn't meet the difficulty requirement.
pub fn verify_solution(challenge: &str, nonce_str: &str, difficulty: usize) -> bool {
    nonce_str
        .parse::<u64>()
        .map(|nonce| {
            let hash = calculate_hash(challenge, nonce);
            hash.starts_with(&"0".repeat(difficulty))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_calculation() {
        let hash = calculate_hash("test_challenge", 12345);
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_verification() {
        let challenge = "test_challenge";
        let difficulty = 1;

        let (nonce, _) = find_solution(challenge, difficulty).unwrap();

        assert!(verify_solution(challenge, &nonce.to_string(), difficulty));
        assert!(!verify_solution(challenge, "999999", difficulty));
    }
}