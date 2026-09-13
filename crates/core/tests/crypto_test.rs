use agent_test_guard_core::crypto::{
    blake3_hash, blake3_hash_hex, blake3_keyed_hash, blake3_keyed_hash_hex, derive_key,
    generate_keypair, sign, sign_hex, signing_key_from_hex, signing_key_to_hex, verify, verify_hex,
    verify_raw, verifying_key_from_hex, verifying_key_to_hex, CryptoError, SigningKey,
    VerifyingKey,
};

#[test]
fn test_blake3_hash_when_empty_string_and_arbitrary_data() {
    // Given: empty byte slice and known BLAKE3 test vector
    let empty_data = b"";
    let expected_empty_hex = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    let sample_data = b"agent-test-guard-payload";

    // When: computing raw and hex hashes
    let empty_hex = blake3_hash_hex(empty_data);
    let empty_raw = blake3_hash(empty_data);
    let sample_raw_first = blake3_hash(sample_data);
    let sample_raw_second = blake3_hash(sample_data);
    let sample_hex = blake3_hash_hex(sample_data);

    // Then: matches standard test vector, hex matches raw, and hashes are deterministic
    assert_eq!(empty_hex, expected_empty_hex);
    assert_eq!(empty_raw.len(), 32);
    assert_eq!(sample_raw_first, sample_raw_second);
    assert_ne!(empty_raw, sample_raw_first);
    assert_eq!(sample_hex.len(), 64);
}

#[test]
fn test_blake3_keyed_hash_when_different_keys_and_same_key() {
    // Given: a message and two distinct 32-byte keys
    let message = b"baseline-manifest-data";
    let key_a = [0x42u8; 32];
    let key_b = [0x43u8; 32];

    // When: computing keyed hashes with key_a twice and key_b once
    let hash_a1 = blake3_keyed_hash(&key_a, message);
    let hash_a2 = blake3_keyed_hash(&key_a, message);
    let hash_a1_hex = blake3_keyed_hash_hex(&key_a, message);
    let hash_b = blake3_keyed_hash(&key_b, message);

    // Then: same key is deterministic; different keys produce orthogonal hashes
    assert_eq!(hash_a1, hash_a2);
    assert_ne!(hash_a1, hash_b);
    assert_eq!(hash_a1_hex.len(), 64);
}

#[test]
fn test_derive_key_when_different_contexts_and_same_material() {
    // Given: shared key material and two orthogonal application contexts
    let key_material = b"agent-test-guard-root-entropy-seed";
    let ctx_ratchet = "agent-test-guard:ratchet:v1";
    let ctx_session = "agent-test-guard:session:v1";

    // When: deriving subkeys across identical and distinct contexts
    let subkey_ratchet_1 = derive_key(ctx_ratchet, key_material);
    let subkey_ratchet_2 = derive_key(ctx_ratchet, key_material);
    let subkey_session = derive_key(ctx_session, key_material);

    // Then: derivation is deterministic within context and separated across contexts
    assert_eq!(subkey_ratchet_1, subkey_ratchet_2);
    assert_ne!(subkey_ratchet_1, subkey_session);
    assert_eq!(subkey_ratchet_1.len(), 32);
}

#[test]
fn test_ed25519_sign_and_verify_when_valid_keypair_and_payload() {
    // Given: generated keypair and an authentic payload
    let (signing_key, verifying_key) = generate_keypair();
    let message = b"{\"tests\":42,\"sha256\":\"authentic\"}";

    // When: signing with native and hex APIs, then verifying across all variants
    let signature = sign(&signing_key, message);
    let sig_hex = sign_hex(&signing_key, message);
    let vk_hex = verifying_key_to_hex(&verifying_key);

    let verify_res = verify(&verifying_key, message, &signature);
    let verify_raw_res = verify_raw(&verifying_key.to_bytes(), message, &signature.to_bytes());
    let verify_hex_res = verify_hex(&vk_hex, message, &sig_hex);

    // Then: native, raw bytes, and hex verification all succeed
    assert!(verify_res.is_ok());
    assert!(verify_raw_res.is_ok());
    assert!(verify_hex_res.is_ok());
}

#[test]
fn test_ed25519_verify_fails_when_message_tampered() {
    // Given: valid keypair, signature over authentic message, and 1-bit flipped message
    let (signing_key, verifying_key) = generate_keypair();
    let message = b"audit_count:100;passed:true";
    let signature = sign(&signing_key, message);
    let sig_hex = sign_hex(&signing_key, message);
    let vk_hex = verifying_key_to_hex(&verifying_key);

    let mut tampered_message = message.to_vec();
    tampered_message[0] ^= 0x01;

    // When: attempting verification against tampered message
    let res_typed: Result<(), CryptoError> = verify(&verifying_key, &tampered_message, &signature);
    let res_raw: Result<(), CryptoError> = verify_raw(
        &verifying_key.to_bytes(),
        &tampered_message,
        &signature.to_bytes(),
    );
    let res_hex: Result<(), CryptoError> = verify_hex(&vk_hex, &tampered_message, &sig_hex);

    // Then: all verification attempts reject the tampered message with an error
    assert!(res_typed.is_err());
    assert!(res_raw.is_err());
    assert!(res_hex.is_err());
}

#[test]
fn test_ed25519_verify_fails_when_wrong_public_key() {
    // Given: two distinct keypairs and signature issued by keypair A
    let (signing_key_a, _verifying_key_a) = generate_keypair();
    let (_signing_key_b, verifying_key_b) = generate_keypair();
    let message = b"unauthorized-key-substitution";
    let signature_a = sign(&signing_key_a, message);
    let sig_hex_a = sign_hex(&signing_key_a, message);
    let vk_hex_b = verifying_key_to_hex(&verifying_key_b);

    // When: verifying keypair A's signature against keypair B's public key
    let res_typed: Result<(), CryptoError> = verify(&verifying_key_b, message, &signature_a);
    let res_raw: Result<(), CryptoError> = verify_raw(
        &verifying_key_b.to_bytes(),
        message,
        &signature_a.to_bytes(),
    );
    let res_hex: Result<(), CryptoError> = verify_hex(&vk_hex_b, message, &sig_hex_a);

    // Then: verification fails across all interfaces
    assert!(res_typed.is_err());
    assert!(res_raw.is_err());
    assert!(res_hex.is_err());
}

#[test]
fn test_ed25519_verify_fails_when_signature_corrupted() {
    // Given: valid keypair, message, and signature with corrupted bytes
    let (signing_key, verifying_key) = generate_keypair();
    let message = b"tamper-evident-signature-check";
    let signature = sign(&signing_key, message);
    let vk_hex = verifying_key_to_hex(&verifying_key);

    let mut corrupted_sig_bytes = signature.to_bytes();
    corrupted_sig_bytes[0] ^= 0xff;

    let valid_sig_hex = sign_hex(&signing_key, message);
    let mut corrupted_sig_hex = valid_sig_hex;
    let replacement = if corrupted_sig_hex.starts_with("aa") {
        "bb"
    } else {
        "aa"
    };
    corrupted_sig_hex.replace_range(0..2, replacement);

    // When: verifying with corrupted signature bytes and hex
    let res_raw: Result<(), CryptoError> =
        verify_raw(&verifying_key.to_bytes(), message, &corrupted_sig_bytes);
    let res_hex: Result<(), CryptoError> = verify_hex(&vk_hex, message, &corrupted_sig_hex);

    // Then: verification fails due to signature corruption
    assert!(res_raw.is_err());
    assert!(res_hex.is_err());
}

#[test]
fn test_hex_roundtrip_when_signing_and_verifying_keys() {
    // Given: freshly generated Ed25519 keypair
    let (signing_key, verifying_key) = generate_keypair();

    // When: converting keys to hex strings and reconstructing back
    let vk_hex = verifying_key_to_hex(&verifying_key);
    let sk_hex = signing_key_to_hex(&signing_key);

    let decoded_vk = verifying_key_from_hex(&vk_hex);
    let decoded_sk = signing_key_from_hex(&sk_hex);

    // Then: hex encoding roundtrip reproduces exact keys
    assert!(matches!(&decoded_vk, Ok(vk) if vk.to_bytes() == verifying_key.to_bytes()));
    assert!(matches!(&decoded_sk, Ok(sk) if sk.to_bytes() == signing_key.to_bytes()));
    assert_eq!(vk_hex.len(), 64);
    assert_eq!(sk_hex.len(), 64);
}

#[test]
fn test_hex_decode_fails_when_invalid_hex_or_wrong_length() {
    // Given: malformed hex strings with invalid characters or incorrect lengths
    let invalid_chars = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
    let odd_length = "abc";
    let too_short = "01020304";
    let message = b"hex-boundary-input";

    // When: attempting to decode keys and verify signatures with malformed hex
    let res_vk_chars: Result<VerifyingKey, CryptoError> = verifying_key_from_hex(invalid_chars);
    let res_vk_odd: Result<VerifyingKey, CryptoError> = verifying_key_from_hex(odd_length);
    let res_vk_short: Result<VerifyingKey, CryptoError> = verifying_key_from_hex(too_short);

    let res_sk_chars: Result<SigningKey, CryptoError> = signing_key_from_hex(invalid_chars);
    let res_sk_odd: Result<SigningKey, CryptoError> = signing_key_from_hex(odd_length);
    let res_sk_short: Result<SigningKey, CryptoError> = signing_key_from_hex(too_short);

    let res_verify_bad_pk: Result<(), CryptoError> = verify_hex(invalid_chars, message, too_short);
    let res_verify_bad_sig: Result<(), CryptoError> =
        verify_hex("00".repeat(32).as_str(), message, "bad_sig");

    // Then: invalid hex or invalid lengths return CryptoError
    assert!(res_vk_chars.is_err());
    assert!(res_vk_odd.is_err());
    assert!(res_vk_short.is_err());
    assert!(res_sk_chars.is_err());
    assert!(res_sk_odd.is_err());
    assert!(res_sk_short.is_err());
    assert!(res_verify_bad_pk.is_err());
    assert!(res_verify_bad_sig.is_err());
}
