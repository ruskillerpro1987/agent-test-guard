//! Cryptographic primitives: BLAKE3 keyed hashing and Ed25519 signing/verification

pub use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use ed25519_dalek::{Signer, Verifier};
use thiserror::Error;

/// Cryptographic errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CryptoError {
    /// Invalid signature format or bytes.
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid public key format or bytes.
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Signature verification failed.
    #[error("Signature verification failed")]
    VerificationFailed,

    /// Invalid hex string.
    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    /// Byte sequence has unexpected length.
    #[error("Invalid length: expected {expected}, got {got}")]
    InvalidLength { expected: usize, got: usize },
}

/// Encodes a byte slice into a lowercase hexadecimal string.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

/// Converts an ASCII hex byte into its 4-bit nibble value.
fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Decodes a hexadecimal string into a byte vector.
fn hex_decode(hex_str: &str) -> Result<Vec<u8>, CryptoError> {
    let bytes = hex_str.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err(CryptoError::InvalidHex(format!(
            "odd length hex string: {}",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let hi = hex_val(chunk[0]).ok_or_else(|| {
            CryptoError::InvalidHex(format!("invalid hex character: '{}'", chunk[0] as char))
        })?;
        let lo = hex_val(chunk[1]).ok_or_else(|| {
            CryptoError::InvalidHex(format!("invalid hex character: '{}'", chunk[1] as char))
        })?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

/// Decodes a hexadecimal string into a fixed-size byte array of length `N`.
fn decode_fixed_hex<const N: usize>(hex_str: &str) -> Result<[u8; N], CryptoError> {
    let decoded = hex_decode(hex_str)?;
    if decoded.len() != N {
        return Err(CryptoError::InvalidLength {
            expected: N,
            got: decoded.len(),
        });
    }
    let mut arr = [0u8; N];
    arr.copy_from_slice(&decoded);
    Ok(arr)
}

/// Computes the 32-byte BLAKE3 hash of `data`.
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Computes the BLAKE3 hash of `data` as a lowercase hex string.
pub fn blake3_hash_hex(data: &[u8]) -> String {
    hex_encode(&blake3_hash(data))
}

/// Computes the 32-byte keyed BLAKE3 hash of `data` using a 32-byte key.
pub fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> [u8; 32] {
    *blake3::keyed_hash(key, data).as_bytes()
}

/// Computes the keyed BLAKE3 hash of `data` as a lowercase hex string.
pub fn blake3_keyed_hash_hex(key: &[u8; 32], data: &[u8]) -> String {
    hex_encode(&blake3_keyed_hash(key, data))
}

/// Derives a 32-byte subkey from context and key material using BLAKE3 KDF.
pub fn derive_key(context: &str, key_material: &[u8]) -> [u8; 32] {
    blake3::derive_key(context, key_material)
}

fn fill_random_bytes(buf: &mut [u8]) {
    #[cfg(windows)]
    {
        #[link(name = "advapi32")]
        extern "system" {
            #[link_name = "SystemFunction036"]
            fn rtl_gen_random(buffer: *mut u8, length: u32) -> u8;
        }
        // SAFETY: `buf` is a valid, writable byte slice of length `buf.len()`,
        // and SystemFunction036 (RtlGenRandom) safely fills it with cryptographically secure random bytes.
        unsafe {
            rtl_gen_random(buf.as_mut_ptr(), buf.len() as u32);
        }
    }
    #[cfg(not(windows))]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            let _ = f.read_exact(buf);
        }
    }
}

/// Generates a new random Ed25519 signing and verifying keypair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut seed = [0u8; 32];
    fill_random_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Signs `message` with `signing_key` and returns the Ed25519 signature.
pub fn sign(signing_key: &SigningKey, message: &[u8]) -> Signature {
    signing_key.sign(message)
}

/// Signs `message` with `signing_key` and returns the signature as a hex string.
pub fn sign_hex(signing_key: &SigningKey, message: &[u8]) -> String {
    hex_encode(&sign(signing_key, message).to_bytes())
}

/// Verifies `signature` over `message` using `verifying_key`.
pub fn verify(
    verifying_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> Result<(), CryptoError> {
    verifying_key
        .verify(message, signature)
        .map_err(|_| CryptoError::VerificationFailed)
}

/// Verifies raw byte signature over `message` against raw 32-byte public key.
pub fn verify_raw(
    public_key_bytes: &[u8; 32],
    message: &[u8],
    signature_bytes: &[u8; 64],
) -> Result<(), CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
    let signature = Signature::from_bytes(signature_bytes);
    verify(&verifying_key, message, &signature)
}

/// Verifies hex-encoded signature over `message` against hex-encoded public key.
pub fn verify_hex(
    public_key_hex: &str,
    message: &[u8],
    signature_hex: &str,
) -> Result<(), CryptoError> {
    let verifying_key = verifying_key_from_hex(public_key_hex)?;
    let signature_bytes = decode_fixed_hex::<64>(signature_hex)?;
    let signature = Signature::from_bytes(&signature_bytes);
    verify(&verifying_key, message, &signature)
}

/// Encodes `verifying_key` to a 64-character lowercase hex string.
pub fn verifying_key_to_hex(verifying_key: &VerifyingKey) -> String {
    hex_encode(verifying_key.as_bytes())
}

/// Encodes `signing_key` to a 64-character lowercase hex string.
pub fn signing_key_to_hex(signing_key: &SigningKey) -> String {
    hex_encode(signing_key.as_bytes())
}

/// Decodes a 64-character hex string into a `VerifyingKey`.
pub fn verifying_key_from_hex(hex_str: &str) -> Result<VerifyingKey, CryptoError> {
    let bytes = decode_fixed_hex::<32>(hex_str)?;
    VerifyingKey::from_bytes(&bytes).map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))
}

/// Decodes a 64-character hex string into a `SigningKey`.
pub fn signing_key_from_hex(hex_str: &str) -> Result<SigningKey, CryptoError> {
    let bytes = decode_fixed_hex::<32>(hex_str)?;
    Ok(SigningKey::from_bytes(&bytes))
}
