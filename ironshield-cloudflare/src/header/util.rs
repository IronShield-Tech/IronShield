use serde::{Deserialize, Deserializer, Serializer};

/// Converts the 64-byte Ed25519 signature array
/// into bytes for serialization.
pub fn serialize_signature<S>(
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
pub fn deserialize_signature<'de, D>(
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