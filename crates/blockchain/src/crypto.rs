//! Cryptographic functions for contracts and signatures

use blake3;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::Address;

/// Hash data using Blake3
pub fn hash_blake3(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// Ed25519 signature (64 bytes)
#[derive(Debug, Clone, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct TransactionSignature([u8; 64]);

impl TransactionSignature {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Get as bytes
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Convert to Ed25519 signature for verification
    pub fn to_signature(&self) -> Result<Signature, ed25519_dalek::SignatureError> {
        Signature::try_from(self.0.as_slice())
    }
}

// Manual Serialize/Deserialize for [u8; 64]
impl Serialize for TransactionSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for TransactionSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = [u8; 64];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a 64-byte array")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() == 64 {
                    let mut bytes = [0u8; 64];
                    bytes.copy_from_slice(v);
                    Ok(bytes)
                } else {
                    Err(E::custom(format!("expected 64 bytes, got {}", v.len())))
                }
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut bytes = [0u8; 64];
                for i in 0..64 {
                    bytes[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(bytes)
            }
        }

        deserializer
            .deserialize_bytes(BytesVisitor)
            .map(TransactionSignature)
    }
}

/// A keypair for signing transactions
#[derive(Clone)]
pub struct Keypair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Keypair {
    /// Generate a new random keypair
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes);

        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create from a 32-byte seed (for deterministic key generation)
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Get the address derived from the public key
    /// Address = first 20 bytes of blake3(public_key)
    pub fn address(&self) -> Address {
        let public_key_bytes = self.verifying_key.as_bytes();
        let hash = blake3::hash(public_key_bytes);
        Address::from_slice(&hash.as_bytes()[..20])
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> TransactionSignature {
        let signature = self.signing_key.sign(message);
        TransactionSignature(signature.to_bytes())
    }

    /// Get the verifying key (public key)
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    /// Get the verifying key as bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

/// Verify a signature against a message and public key
pub fn verify_signature(
    message: &[u8],
    signature: &TransactionSignature,
    public_key: &[u8; 32],
) -> anyhow::Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key)
        .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;

    let sig = signature
        .to_signature()
        .map_err(|e| anyhow::anyhow!("Invalid signature format: {:?}", e))?;

    verifying_key
        .verify(message, &sig)
        .map_err(|_| anyhow::anyhow!("Signature verification failed"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_blake3() {
        let data = b"hello world";
        let hash1 = hash_blake3(data);
        let hash2 = hash_blake3(data);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, [0u8; 32]);
    }

    #[test]
    fn test_keypair_generation() {
        let keypair = Keypair::generate();
        let address = keypair.address();

        // Address should be 20 bytes
        assert_eq!(address.as_bytes().len(), 20);
    }

    #[test]
    fn test_sign_and_verify() {
        let keypair = Keypair::generate();
        let message = b"Hello, blockchain!";

        // Sign
        let signature = keypair.sign(message);

        // Verify
        let result = verify_signature(message, &signature, &keypair.public_key_bytes());
        assert!(result.is_ok(), "Signature verification should succeed");
    }

    #[test]
    fn test_verify_wrong_message() {
        let keypair = Keypair::generate();
        let message = b"Hello, blockchain!";
        let wrong_message = b"Wrong message";

        // Sign correct message
        let signature = keypair.sign(message);

        // Verify with wrong message
        let result = verify_signature(wrong_message, &signature, &keypair.public_key_bytes());
        assert!(result.is_err(), "Verification should fail with wrong message");
    }

    #[test]
    fn test_verify_wrong_key() {
        let keypair1 = Keypair::generate();
        let keypair2 = Keypair::generate();
        let message = b"Hello, blockchain!";

        // Sign with keypair1
        let signature = keypair1.sign(message);

        // Verify with keypair2's public key
        let result = verify_signature(message, &signature, &keypair2.public_key_bytes());
        assert!(
            result.is_err(),
            "Verification should fail with wrong public key"
        );
    }

    #[test]
    fn test_deterministic_keys() {
        let seed = [42u8; 32];

        let keypair1 = Keypair::from_seed(&seed);
        let keypair2 = Keypair::from_seed(&seed);

        // Same seed should produce same keys
        assert_eq!(
            keypair1.public_key_bytes(),
            keypair2.public_key_bytes(),
            "Same seed should produce same keys"
        );
        assert_eq!(
            keypair1.address(),
            keypair2.address(),
            "Same seed should produce same address"
        );
    }
}
