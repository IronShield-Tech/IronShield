//! # Core functionality for the IronShield proof-of-work system.
//! 
//! This module contains shared code that can be used in both
//! the server-side (Cloudflare Workers) and client-side (WASM) implementations

use hex;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use sha2::{Digest, Sha256};
pub use ironshield_types::*; // Re-export types from ironshield-types


const MAX_ATTEMPTS:   u64 = 10_000_000; // Maximum number of nonce values to try before giving up.
const CHUNK_SIZE:   usize = 10_000; // Number of nonce values processed in each parallel chunk.
const MAX_ATTEMPTS_SINGLE_THREADED: i64 = 100_000_000; // Maximum number of nonce values to try in the new algorithm before giving up.

/// Find a solution for the given challenge and difficulty level
/// using sequential search.
/// 
/// This function searches for a nonce value that, when combined 
/// with the challenge string and hashed with SHA-256, produces a hash
/// starting with the required number of leading zeros.
/// 
/// # Arguments
/// * `challenge` - The challenge string to hash (typically server-provided).
/// * `difficulty` - Number of leading zeros required in the hash (higher = more difficult).
///
/// # Returns
/// * `Ok((nonce, hash))` - The successful nonce value and resulting hash.
/// * `Err(message)` - Error if no solution is found within `MAX_ATTEMPTS`.
///
/// # Performance
/// Sequential search is suitable for single-threaded environments like WASM.
pub fn find_solution(challenge: &str, difficulty: usize) -> Result<(u64, String), String> {
    let target_prefix = "0".repeat(difficulty);

    for nonce in 0..MAX_ATTEMPTS {
        let hash = calculate_hash(challenge, nonce);

        if hash.starts_with(&target_prefix) {
            return Ok((nonce, hash));
        }
    }

    Err("Could not find solution within attempt limit".into())
}


/// Find a solution using parallel processing
/// 
/// Something Ethan is working on. 
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

/// Calculate the SHA-256 hash for a given challenge and nonce combination.
///
/// The input format is "challenge:nonce" (e.g., "hello_world:12345").
///
/// # Arguments
/// * `challenge` - The challenge string.
/// * `nonce` - The nonce value to try.
///
/// # Returns
/// * Hexadecimal string representation of the SHA-256 hash (64 chars long).
pub fn calculate_hash(challenge: &str, nonce: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", challenge, nonce).as_bytes());
    hex::encode(hasher.finalize())
}

/// Verify that a given nonce produces a valid solution for the challenge.
///
/// # Arguments
/// * `challenge` - The original challenge string.
/// * `nonce_str` - The proposed nonce as a string (will be parsed to u64).
/// * `difficulty` - Required number of leading zeros in the hash.
///
/// # Returns
/// * `true` - If the nonce produces a hash meeting the difficulty requirement
/// * `false` - If nonce is invalid, hash doesn't meet the requirement, or parsing fails.
///
/// # Safety
/// This function handles invalid nonce strings gracefully by returning false.
pub fn verify_solution(challenge: &str, nonce_str: &str, difficulty: usize) -> bool {
    nonce_str
        .parse::<u64>()
        .map(|nonce| {
            let hash = calculate_hash(challenge, nonce);
            hash.starts_with(&"0".repeat(difficulty))
        })
        .unwrap_or(false)
}

/// Find a solution for the given IronShieldChallenge using single-threaded computation.
/// 
/// This function implements a proof-of-work algorithm that finds a nonce value such that
/// when concatenated with the challenge's random_nonce and hashed with SHA-256, the 
/// resulting hash (interpreted as a [u8; 32]) is numerically less than the challenge_param.
/// 
/// The algorithm:
/// 1. Takes the random_nonce from the challenge (as bytes)
/// 2. Iterates through nonce values (starting from 0)
/// 3. For each nonce: hashes random_nonce_bytes + nonce_bytes using multiple hasher updates
/// 4. Compares the hash [u8; 32] with challenge_param [u8; 32] using byte-wise comparison
/// 5. Returns the first nonce where hash < challenge_param
/// 
/// 
/// # Arguments
/// * `challenge` - The IronShieldChallenge struct containing random_nonce and challenge_param
/// 
/// # Returns
/// * `Ok(IronShieldChallengeResponse)` - Contains the successful nonce and signature
/// * `Err(String)` - Error message if no solution found within MAX_ATTEMPTS_SINGLE_THREADED
/// 
/// # Example
/// The challenge contains:
/// - random_nonce: "abc123def456" (hex string)
/// - challenge_param: [0x00, 0x00, 0xFF, ...] (target threshold)
/// 
/// The function will find nonce N such that:
/// SHA256(hex::decode("abc123def456") + N.to_le_bytes()) < challenge_param
pub fn find_solution_single_threaded(
    challenge: &IronShieldChallenge,
) -> Result<IronShieldChallengeResponse, String> {
    
    // Parse the random_nonce from hex string to bytes
    let random_nonce_bytes: Vec<u8> = hex::decode(&challenge.random_nonce)
        .map_err(|e: hex::FromHexError| format!("Failed to decode random_nonce hex: {}", e))?;
    
    // Get the target threshold from challenge_param
    let target_threshold: &[u8; 32] = &challenge.challenge_param;
    
    // Iterate through possible nonce values
    for nonce in 0..MAX_ATTEMPTS_SINGLE_THREADED {
        // Convert nonce to little-endian bytes (8 bytes for i64)
        let nonce_bytes: [u8; 8] = nonce.to_le_bytes();
        
        // Calculate the hash of the random_nonce and nonce
        let mut hasher = Sha256::new();
        hasher.update(&random_nonce_bytes);  // First part of the input
        hasher.update(&nonce_bytes);         // Second part of the input
        let hash_result = hasher.finalize();
        
        // Convert hash and use byte-wise comparison with the target threshold
        let hash_bytes: [u8; 32] = hash_result.into();
        if hash_bytes < *target_threshold {
            // Found a valid solution!
            return Ok(IronShieldChallengeResponse::new(
                challenge.challenge_signature, // Copy the challenge signature
                nonce, // The successful nonce value
            ));
        }
    }
    
    // No solution found within the attempt limit
    Err(format!("Could not find solution within {} attempts", MAX_ATTEMPTS_SINGLE_THREADED))
}

/// Verify that a solution is valid for a given IronShieldChallenge.
/// 
/// This function uses the same optimized hashing approach as find_solution_single_threaded
/// to ensure consistency and performance.
/// 
/// # Arguments
/// * `challenge` - The original IronShieldChallenge
/// * `nonce` - The proposed solution nonce
/// 
/// # Returns
/// * `true` if the nonce produces a hash less than the challenge_param
/// * `false` if the nonce is invalid or doesn't meet the requirement
pub fn verify_ironshield_solution(challenge: &IronShieldChallenge, nonce: i64) -> bool {
    // Parse the random_nonce from hex string to bytes
    let random_nonce_bytes = match hex::decode(&challenge.random_nonce) {
        Ok(bytes) => bytes,
        Err(_) => return false, // Invalid hex string
    };
    
    // Convert nonce to little-endian bytes
    let nonce_bytes: [u8; 8] = nonce.to_le_bytes();
    
    // Use the same optimized hashing approach as the main function
    let mut hasher = Sha256::new();
    hasher.update(&random_nonce_bytes);  // First part of the input
    hasher.update(&nonce_bytes);         // Second part of the input
    let hash_result = hasher.finalize();
    
    // Convert hash to [u8; 32] for comparison
    let hash_bytes: [u8; 32] = hash_result.into();
    
    // Compare with the challenge parameter
    hash_bytes < challenge.challenge_param
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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

    #[test]
    fn test_ironshield_challenge_creation() {
        let challenge = IronShieldChallenge::new(
            "deadbeef".to_string(),
            1000000,
            "test_website".to_string(),
            [0xFF; 32], // Very high threshold - should be easy to find solution
            [0x00; 32],
            [0x00; 64],
        );
        
        assert_eq!(challenge.random_nonce, "deadbeef");
        assert_eq!(challenge.created_time, 1000000);
        assert_eq!(challenge.expiration_time, 1030000); // +30 seconds
        assert_eq!(challenge.website_id, "test_website");
        assert_eq!(challenge.challenge_param, [0xFF; 32]);
    }

    #[test]
    fn test_find_solution_single_threaded_easy() {
        // Create a challenge with very high threshold (easy to solve)
        let challenge = IronShieldChallenge::new(
            "deadbeef".to_string(),
            1000000,
            "test_website".to_string(),
            [0xFF; 32], // Maximum possible value - should find solution quickly
            [0x00; 32],
            [0x11; 64],
        );
        
        let result = find_solution_single_threaded(&challenge);
        assert!(result.is_ok(), "Should find solution for easy challenge");
        
        let response = result.unwrap();
        assert_eq!(response.challenge_signature, [0x11; 64]);
        assert!(response.solution >= 0, "Solution should be non-negative");
        
        // Verify the solution is actually valid using our optimized verification function
        assert!(verify_ironshield_solution(&challenge, response.solution), 
                "Solution should satisfy the challenge");
    }

    #[test]
    fn test_find_solution_single_threaded_invalid_hex() {
        // Create a challenge with invalid hex string
        let challenge = IronShieldChallenge::new(
            "not_valid_hex!".to_string(), // Invalid hex
            1000000,
            "test_website".to_string(),
            [0xFF; 32],
            [0x00; 32],
            [0x11; 64],
        );
        
        let result = find_solution_single_threaded(&challenge);
        assert!(result.is_err(), "Should fail for invalid hex");
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Failed to decode random_nonce hex"), "Should contain hex decode error");
    }

    #[test]
    fn test_ironshield_challenge_expiration() {
        let past_time = Utc::now().timestamp_millis() - 60000; // 1 minute ago
        let challenge: IronShieldChallenge = IronShieldChallenge::new(
            "deadbeef".to_string(),
            past_time,
            "test_website".to_string(),
            [0xFF; 32],
            [0x00; 32],
            [0x00; 64],
        );
        
        assert!(challenge.is_expired(), "Challenge created in the past should be expired");
        assert!(challenge.time_until_expiration() < 0, "Time until expiration should be negative");
    }

    #[test]
    fn test_serde_serialization() {
        let challenge: IronShieldChallenge = IronShieldChallenge::new(
            "deadbeef".to_string(),
            1000000,
            "test_website".to_string(),
            [0x12; 32],
            [0x34; 32],
            [0x56; 64],
        );
        
        // Test serialization
        let serialized = serde_json::to_string(&challenge).unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization
        let deserialized: IronShieldChallenge = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.random_nonce, challenge.random_nonce);
        assert_eq!(deserialized.challenge_param, challenge.challenge_param);
        assert_eq!(deserialized.public_key, challenge.public_key);
        assert_eq!(deserialized.challenge_signature, challenge.challenge_signature);
    }

    #[test]
    fn test_verify_ironshield_solution() {
        // Create a challenge with reasonable threshold
        let challenge: IronShieldChallenge = IronShieldChallenge::new(
            "cafe1234".to_string(),
            1000000,
            "test_website".to_string(),
            [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Medium threshold
            [0x00; 32],
            [0x22; 64],
        );
        
        // Find a solution
        let result = find_solution_single_threaded(&challenge);
        assert!(result.is_ok(), "Should find solution for reasonable challenge");
        
        let response = result.unwrap();
        
        // Verify using our verification function
        assert!(verify_ironshield_solution(&challenge, response.solution), 
                "Verification function should confirm the solution is valid");
                
        // Verify that an obviously wrong nonce fails (much larger value)
        assert!(!verify_ironshield_solution(&challenge, response.solution + 1000000), 
                "Obviously wrong nonce should fail verification");
                
        // Test with invalid hex in the challenge
        let bad_challenge: IronShieldChallenge = IronShieldChallenge::new(
            "invalid_hex_zzzz".to_string(), // Invalid hex
            1000000,
            "test_website".to_string(),
            [0x80; 32],
            [0x00; 32],
            [0x22; 64],
        );
        assert!(!verify_ironshield_solution(&bad_challenge, 12345), 
                "Challenge with invalid hex should fail verification");
    }

    #[test]
    fn test_performance_optimization_correctness() {
        // This test ensures that our optimization produces the same results
        // as the original Vec-based approach would have
        
        let random_nonce = "deadbeefcafe1234";
        let random_nonce_bytes = hex::decode(random_nonce).unwrap();
        let nonce: i64 = 12345;
        let nonce_bytes = nonce.to_le_bytes();
        
        // Method 1: Optimized approach (multiple hasher updates)
        let mut hasher1 = Sha256::new();
        hasher1.update(&random_nonce_bytes);
        hasher1.update(&nonce_bytes);
        let hash1: [u8; 32] = hasher1.finalize().into();
        
        // Method 2: Traditional approach (Vec concatenation) - for comparison
        let mut input_data = Vec::with_capacity(random_nonce_bytes.len() + 8);
        input_data.extend_from_slice(&random_nonce_bytes);
        input_data.extend_from_slice(&nonce_bytes);
        let mut hasher2 = Sha256::new();
        hasher2.update(&input_data);
        let hash2: [u8; 32] = hasher2.finalize().into();
        
        // Both methods should produce identical results
        assert_eq!(hash1, hash2, "Optimized and traditional methods should produce identical hashes");
    }

    #[test]
    fn test_recommended_attempts() {
        // Test the new recommended_attempts function
        assert_eq!(IronShieldChallenge::recommended_attempts(1000), 3000);
        assert_eq!(IronShieldChallenge::recommended_attempts(50000), 150000);
        assert_eq!(IronShieldChallenge::recommended_attempts(0), 0);
        
        // Test overflow protection
        assert_eq!(IronShieldChallenge::recommended_attempts(u64::MAX), u64::MAX);
    }

    #[test]
    fn test_difficulty_to_challenge_param() {
        // Test that our difficulty conversion works correctly
        
        // Very easy case
        let challenge_param = IronShieldChallenge::difficulty_to_challenge_param(1);
        assert_eq!(challenge_param, [0xFF; 32]);
        
        // Test zero difficulty panics
        let result = std::panic::catch_unwind(|| {
            IronShieldChallenge::difficulty_to_challenge_param(0);
        });
        assert!(result.is_err(), "Zero difficulty should panic");
        
        // Test some practical values produce valid outputs
        let challenge_param_256 = IronShieldChallenge::difficulty_to_challenge_param(256);
        assert_ne!(challenge_param_256, [0; 32]);
        assert_ne!(challenge_param_256, [0xFF; 32]);
        
        let challenge_param_1024 = IronShieldChallenge::difficulty_to_challenge_param(1024);
        assert_ne!(challenge_param_1024, [0; 32]);
        assert_ne!(challenge_param_1024, [0xFF; 32]);
        
        // Test very high difficulty
        let challenge_param_max = IronShieldChallenge::difficulty_to_challenge_param(u64::MAX);
        assert_eq!(challenge_param_max, [0; 32], "Maximum difficulty should produce all zeros");
        
        // Test that the function produces consistent results
        let challenge_param_test = IronShieldChallenge::difficulty_to_challenge_param(1000);
        let challenge_param_test2 = IronShieldChallenge::difficulty_to_challenge_param(1000);
        assert_eq!(challenge_param_test, challenge_param_test2, "Function should be deterministic");
    }
}