use chrono::Utc;
use crate::header::util::deserialize_signature;
use crate::header::util::serialize_signature;
use serde::{Deserialize, Serialize};

/// * `challenge_signature`:      The Ed25519 signature of the challenge.
/// * `valid_for`:                The Unix timestamp in unix millis.
/// * `public_key`:               The Ed25519 public key corresponding 
///                               to the central private key (32 bytes).
/// * `authentication_signature`: The signature over (challenge_signature 
///                               || valid_for).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronShieldToken {
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub challenge_signature:      [u8; 64],
    pub valid_for:                i64,
    pub public_key:               [u8; 32],
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub authentication_signature: [u8; 64],
}

impl IronShieldToken {
    pub fn new(
        challenge_signature:      [u8; 64],
        valid_for:                i64,
        public_key:               [u8; 32],
        authentication_signature: [u8; 64],
    ) -> Self {
        Self {
            challenge_signature,
            valid_for,
            public_key,
            authentication_signature,
        }
    }

    /// Check if the challenge has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.valid_for
    }

    /// Get the signable data for authentication signature verification.
    /// 
    /// Concatenates: 
    /// * `challenge_signature` as bytes.
    /// * `valid_for`           as a big-endian 8-byte integer.
    pub fn signable_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.challenge_signature);
        data.extend_from_slice(&self.valid_for.to_be_bytes());
        data
    }
}