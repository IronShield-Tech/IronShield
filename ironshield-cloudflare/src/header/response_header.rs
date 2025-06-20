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
}