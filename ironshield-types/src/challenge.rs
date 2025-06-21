use crate::serde_utils::{serialize_signature, deserialize_signature, serialize_32_bytes, deserialize_32_bytes};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// IronShield Challenge structure for the proof-of-work algorithm
/// 
/// * `random_nonce`:     The SHA-256 hash of a random number (hex string).
/// * `created_time`:     Unix milli timestamp for the challenge.
/// * `expiration_time`:  Unix milli timestamp for the challenge expiration time.
/// * `challenge_param`:  Target threshold - hash must be less than this value.
/// * `website_id`:       The identifier of the website.
/// * `public_key`:       Ed25519 public key for signature verification.
/// * `challenge_signature`: Ed25519 signature over the challenge data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronShieldChallenge {
    pub random_nonce:        String,
    pub created_time:        i64,
    pub expiration_time:     i64,
    pub website_id:          String,
    #[serde(
        serialize_with = "serialize_32_bytes",
        deserialize_with = "deserialize_32_bytes"
    )]
    pub challenge_param:     [u8; 32],
    #[serde(
        serialize_with = "serialize_32_bytes",
        deserialize_with = "deserialize_32_bytes"
    )]
    pub public_key:          [u8; 32],
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub challenge_signature: [u8; 64],
}

impl IronShieldChallenge {
    /// Constructor for creating a new IronShieldChallenge instance.
    pub fn new(
        random_nonce:     String,
        created_time:     i64,
        website_id:       String,
        challenge_param:  [u8; 32],
        public_key:       [u8; 32],
        signature:        [u8; 64],
    ) -> Self {
        Self {
            random_nonce,
            created_time,
            website_id,
            expiration_time: created_time + 30_000, // 30 seconds
            challenge_param,
            public_key,
            challenge_signature: signature,
        }
    }

    /// Converts a difficulty value (expected number of attempts) to a challenge_param.
    ///
    /// The difficulty represents the expected number of hash attempts needed to find a valid nonce
    /// where SHA256(random_nonce_bytes + nonce_bytes) < challenge_param.
    ///
    /// Since hash outputs are uniformly distributed over the 256-bit space, the relationship is:
    /// challenge_param = 2^256 / difficulty
    ///
    /// This function calculates this by finding the appropriate byte representation.
    ///
    /// # Arguments
    /// * `difficulty` - Expected number of attempts (must be > 0)
    ///
    /// # Returns
    /// * `[u8; 32]` - The challenge_param bytes in big-endian format
    ///
    /// # Panics
    /// * Panics if difficulty is 0
    ///
    /// # Examples
    /// * difficulty = 1 → challenge_param = [0xFF; 32] (very easy, ~100% chance)
    /// * difficulty = 256 → challenge_param = [0x01, 0x00, 0x00, ...] (MSB in first byte)
    /// * difficulty = 2^8 → challenge_param = [0x00, 0xFF, 0xFF, ...] (MSB in second byte)
    pub fn difficulty_to_challenge_param(difficulty: u64) -> [u8; 32] {
        if difficulty == 0 {
            panic!("Difficulty cannot be zero");
        }
        
        if difficulty == 1 {
            // Special case: difficulty 1 means almost certain success
            return [0xFF; 32];
        }
        
        // For difficulty > 1, we need to calculate 2^256 / difficulty
        // This is equivalent to finding the position where we place the MSB
        
        // Find the highest bit position in difficulty
        let difficulty_bits = 64 - difficulty.leading_zeros() as usize;
        
        // The target value should be approximately 2^256 / difficulty
        // This means the MSB should be at bit position (256 - difficulty_bits)
        let target_bit_position = if difficulty_bits > 256 {
            return [0; 32]; // Impossibly hard
        } else {
            256 - difficulty_bits
        };
        
        let mut result = [0u8; 32];
        
        if target_bit_position >= 256 {
            // Very easy case, return max value
            return [0xFF; 32];
        }
        
        // Calculate which byte and bit within that byte
        let byte_index = target_bit_position / 8;
        let bit_index = target_bit_position % 8;
        
        // Set the MSB
        if byte_index < 32 {
            result[byte_index] = 1u8 << (7 - bit_index);
            
            // Fill all bytes after the MSB byte with 0xFF for a more accurate approximation
            for i in (byte_index + 1)..32 {
                result[i] = 0xFF;
            }
        }
        
        result
    }

    /// Check if the challenge has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expiration_time
    }

    /// Returns the remaining time until expiration in milliseconds.
    pub fn time_until_expiration(&self) -> i64 {
        self.expiration_time - Utc::now().timestamp_millis()
    }

    /// Returns the recommended number of attempts to expect for a given difficulty.
    /// 
    /// This provides users with a realistic expectation of how many attempts they might need.
    /// Since the expected value is equal to the difficulty, we return 3x the difficulty
    /// to give users a reasonable upper bound for planning purposes.
    /// 
    /// # Arguments
    /// * `difficulty` - The target difficulty (expected number of attempts)
    /// 
    /// # Returns
    /// * `u64` - Recommended number of attempts (3x the difficulty)
    /// 
    /// # Examples
    /// * difficulty = 1000 → recommended_attempts = 3000
    /// * difficulty = 50000 → recommended_attempts = 150000
    pub fn recommended_attempts(difficulty: u64) -> u64 {
        difficulty.saturating_mul(3)
    }

    /// Concatenates the challenge data into a string.
    ///
    /// Concatenates:
    /// - `random_nonce`     as a string.
    /// - `created_time`     as i64.
    /// - `expiration_time`  as i64.
    /// - `website_id`       as a string.
    /// - `public_key`       as a lowercase hex string.
    /// - `challenge_params` as a lowercase hex string.
    pub fn concat_struct(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.random_nonce,
            self.created_time,
            self.expiration_time,
            self.website_id,
            // We need to encode the byte arrays for format! to work.
            hex::encode(self.challenge_param),
            hex::encode(self.public_key),
            hex::encode(self.challenge_signature)
        )
    }

    /// Creates an `IronShieldChallenge` from a concatenated string.
    ///
    /// This function reverses the operation of
    /// `IronShieldChallenge::concat_struct`.
    /// Expects a string in the format:
    /// "random_nonce|created_time|expiration_time|website_id|challenge_params|public_key|challenge_signature"
    ///
    /// # Arguments
    ///
    /// * `concat_str`: The concatenated string to parse, typically
    ///                 generated by `concat_struct()`.
    ///
    /// # Returns
    ///
    /// * `Result<Self, String>`: A result containing the parsed
    ///                           `IronShieldChallenge` or an 
    ///                           error message if parsing fails.
    pub fn from_concat_struct(concat_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = concat_str.split('|').collect();

        if parts.len() != 7 {
            return Err(format!("Expected 7 parts, got {}", parts.len()));
        }

        let random_nonce = parts[0].to_string();

        let created_time = parts[1].parse::<i64>()
            .map_err(|_| "Failed to parse created_time as i64")?;

        let expiration_time = parts[2].parse::<i64>()
            .map_err(|_| "Failed to parse expiration_time as i64")?;

        let website_id = parts[3].to_string();

        let challenge_param_bytes = hex::decode(parts[4])
            .map_err(|_| "Failed to decode challenge_params hex string")?;
        let challenge_param: [u8; 32] = challenge_param_bytes
            .try_into()
            .map_err(|_| "Challenge params must be exactly 32 bytes")?;

        let public_key_bytes = hex::decode(parts[5])
            .map_err(|_| "Failed to decode public_key hex string")?;
        let public_key: [u8; 32] = public_key_bytes.try_into()
            .map_err(|_| "Public key must be exactly 32 bytes")?;

        let signature_bytes = hex::decode(parts[6])
            .map_err(|_| "Failed to decode challenge_signature hex string")?;
        let challenge_signature: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| "Signature must be exactly 64 bytes")?;

        Ok(Self {
            random_nonce,
            created_time,
            expiration_time,
            website_id,
            challenge_param,
            public_key,
            challenge_signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_to_challenge_param() {
        // Test very easy case
        let challenge_param: [u8; 32] = IronShieldChallenge::difficulty_to_challenge_param(1);
        assert_eq!(challenge_param, [0xFF; 32]);
        
        // Test difficulty 2 - should have MSB at bit position 254
        let challenge_param: [u8; 32] = IronShieldChallenge::difficulty_to_challenge_param(2);
        let expected: [u8; 32] = {
            let mut arr: [u8; 32] = [0u8; 32];
            arr[0] = 0x80; // MSB in first byte, bit 7
            for i in 1..32 {
                arr[i] = 0xFF;
            }
            arr
        };
        assert_eq!(challenge_param, expected);
        
        // Test difficulty 256 - should have MSB at bit position 247
        let challenge_param = IronShieldChallenge::difficulty_to_challenge_param(256);
        let expected = {
            let mut arr = [0u8; 32];
            arr[1] = 0x80; // MSB in second byte, bit 7
            for i in 2..32 {
                arr[i] = 0xFF;
            }
            arr
        };
        assert_eq!(challenge_param, expected);
    }
    
    #[test]
    fn test_recommended_attempts() {
        // Test recommended_attempts function
        assert_eq!(IronShieldChallenge::recommended_attempts(1000), 3000);
        assert_eq!(IronShieldChallenge::recommended_attempts(50000), 150000);
        assert_eq!(IronShieldChallenge::recommended_attempts(0), 0);
        
        // Test overflow protection
        assert_eq!(IronShieldChallenge::recommended_attempts(u64::MAX), u64::MAX);
    }
    
    #[test]
    fn test_difficulty_to_challenge_param_edge_cases() {
        // Test that zero difficulty panics
        let result = std::panic::catch_unwind(|| {
            IronShieldChallenge::difficulty_to_challenge_param(0);
        });
        assert!(result.is_err());
        
        // Test very high difficulty
        let challenge_param = IronShieldChallenge::difficulty_to_challenge_param(u64::MAX);
        // Should return all zeros (impossible)
        assert_eq!(challenge_param, [0; 32]);
        
        // Test some practical difficulty values
        let challenge_param = IronShieldChallenge::difficulty_to_challenge_param(10000);
        // Should produce a valid non-zero, non-max challenge_param
        assert_ne!(challenge_param, [0; 32]);
        assert_ne!(challenge_param, [0xFF; 32]);
    }
} 