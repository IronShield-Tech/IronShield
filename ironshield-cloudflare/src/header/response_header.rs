use crate::header::util::deserialize_signature;
use crate::header::util::serialize_signature;
use serde::{Deserialize, Serialize};

/// * `challenge_signature`: The Ed25519 signature of the challenge.
///                          Used to verify the integrity of each 
///                          *different* challenge attempt, this 
///                          ensures that signatures may not be 
///                          reused across different challenges.
/// * `solution`             The solution to the challenge provided
///                          to the client, by the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronShieldChallengeResponse {
    #[serde(
        serialize_with = "serialize_signature",
        deserialize_with = "deserialize_signature"
    )]
    pub challenge_signature: [u8; 64],
    pub solution:            i64,
}

impl IronShieldChallengeResponse {
    pub fn new(
        challenge_signature: [u8; 64], 
        solution: i64
    ) -> Self {
        Self {
            challenge_signature,
            solution,
        }
    }

    /// Concatenates the token data into a string.
    /// 
    /// Concatenates:
    /// - `challenge_signature` as a lowercase hex string.Add commentMore actions
    /// - `valid_for`:          as a string.
    pub fn concat_struct(&self) -> String {
        format!(
            "{}|{}",
            // Use of hex::encode to convert the public key to a hex string
            // "Encodes data as hex string using lowercase characters."
            // Requirement of `format!`.
            hex::encode(self.challenge_signature),
            self.solution
        )
    }
}