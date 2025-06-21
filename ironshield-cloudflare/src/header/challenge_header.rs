use crate::header::util::deserialize_signature;
use crate::header::util::serialize_signature;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// * `random_nonce`:     The SHA-256 hash of a random number.
/// * `created_time`:     Unix milli timestamp for the challenge.
/// * `expiration_time`:  Unix milli timestamp for the challenge
///                       expiration time. (created_time + 30 ms)
/// * `challenge_params`: Size of target number the hashed nonce should be less than.
/// * `website_id`:       The identifier of the website.
/// * `public_key`:       Ed25519 public key for signature verification.
/// * `signature`:        Ed25519 signature over 
///                       (`random_nonce || created_time || expiration_time
///                       || challenge_params`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronShieldChallenge {
    pub random_nonce:        String,
    pub created_time:        i64,
    pub expiration_time:     i64,
    pub website_id:          String,
    pub challenge_params:    [u8; 32],
    pub public_key:          [u8; 32],
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub challenge_signature: [u8; 64],
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
        website_id:       String,
        challenge_params: [u8; 32],
        public_key:       [u8; 32],
        signature:        [u8; 64],
    ) -> Self {
        Self {
            random_nonce,
            created_time,
            website_id,
            expiration_time: created_time + 30_000,
            challenge_params,
            public_key,
            challenge_signature: signature,
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
            hex::encode(self.challenge_params),
            hex::encode(self.public_key),
            hex::encode(self.challenge_signature)
        )
    }
}