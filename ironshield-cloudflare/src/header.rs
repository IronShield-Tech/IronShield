use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronShieldChallenge {
    /// The SHA-256 hash of a random number.
    pub random_nonce:     String,
    /// Unix milli timestamp for the challenge
    /// creation time.
    pub created_time:     i64,
    /// Unix milli timestamp for the challenge
    /// expiration time. (created_time + 30 ms)
    pub expiration_time:  i64,
    /// Challenge difficulty parameter or target
    /// number of leading zeros in the hash.
    pub challenge_params: u8,
    /// Ed25519 public key for signature 
    /// verification.
    pub public_key:       [u8; 32],
    /// Ed25519 signature over 
    /// (`random_nonce || created_time || 
    /// expires_time || challenge_params`).
    #[serde(
        serialize_with = "serialize_signature", 
        deserialize_with = "deserialize_signature"
    )]
    pub signature:        [u8; 64],
}

/// Converts the 64-byte Ed25519 signature array 
/// into bytes for serialization. 
fn serialize_signature<S>(
    signature: &[u8; 64],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(signature)
}

/// Converts serialized bytes back into a 64-byte
/// Ed25519 signature array, with validation to ensure
/// the length (64 bytes) is correct.
fn deserialize_signature<'de, D>(
    deserializer: D
) -> Result<[u8; 64], D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
    
    if bytes.len() != 64 {
        return Err(Error::custom(format!("Expected 64 bytes, got {}", bytes.len())));
    }
    
    let mut array = [0u8; 64];
    array.copy_from_slice(&bytes);
    Ok(array)
}

impl IronShieldChallenge {
    /// Constructor, this creates a new `IronShieldChallenge` instance.
    /// 
    /// # Arguments 
    /// 
    /// * `random_nonce`:     The SHA-256 hash of a random number.
    /// * `created_time`:     Unix milli timestamp for the challenge
    ///                       creation time.
    /// * `challenge_params`: Unix milli timestamp for the challenge
    ///                       expiration time. (created_time + 30 ms)
    /// * `public_key`:       Ed25519 public key for signature
    /// * `signature`:        Ed25519 signature over (`random_nonce || 
    ///                       created_time || expires_time || 
    ///                       challenge_params`).
    pub fn new(
        random_nonce:     String,
        created_time:     i64,
        challenge_params: u8,
        public_key:       [u8; 32],
        signature:        [u8; 64],
    ) -> Self {
        Self {
            random_nonce,
            created_time,
            expiration_time: created_time + 30_000,
            challenge_params,
            public_key,
            signature,
        }
    }
    
    /// Check if the challenge has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expiration_time
    }
    
    /// Returns the remaining time until the 
    /// expiration in milliseconds.
    pub fn time_until_expiration(&self) -> i64 {
        self.expiration_time - Utc::now().timestamp_millis()
    }
    
    /// Serializes the signable data for signature verification.
    /// Serializes the signable data (for signature verification)
    /// 
    /// Concatenates:
    /// - `random_nonce`     as bytes
    /// - `created_time`     as 8 bytes, big-endian.
    /// - `expiration_time`  as 8 bytes, big-endian.
    /// - `challenge_params` as a single byte.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.random_nonce.as_bytes());
        data.extend_from_slice(&self.created_time.to_be_bytes());
        data.extend_from_slice(&self.expiration_time.to_be_bytes());
        data.push(self.challenge_params);
        data
    }
}