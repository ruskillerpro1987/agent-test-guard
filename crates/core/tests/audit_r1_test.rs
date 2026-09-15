// crates/core/tests/audit_r1_test.rs
//! Independent Forensic Audit Suite - R1: Core Cryptographic Verification

use agent_test_guard_core::canonical::{
    canonicalize_json_str,
};
use agent_test_guard_core::crypto::{
    blake3_hash_hex, blake3_keyed_hash_hex, derive_key,
    generate_keypair, sign_hex, signing_key_from_hex, verify_hex,
    verifying_key_to_hex,
};
use agent_test_guard_core::ratchet::{verify_ratchet, RatchetError, RatchetOptions};
use agent_test_guard_core::schema::{
    BaselineConfig, BaselineManifest, RatchetState, TestFileBaseline, TestItemBaseline,
    TestLanguage, TestTier,
};

// =========================================================================
// 1. Ed25519 vs RFC 8032 §7.1 Official Test Vectors
// =========================================================================

#[test]
fn test_ed25519_rfc8032_test_vector_1_empty_message() {
    // RFC 8032 Section 7.1 - TEST 1
    let secret_hex = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    let expected_pubkey_hex = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let message = b"";
    let expected_sig_hex = "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";

    let signing_key = signing_key_from_hex(secret_hex).expect("valid secret key hex");
    let pubkey = signing_key.verifying_key();
    let pubkey_hex = verifying_key_to_hex(&pubkey);
    assert_eq!(pubkey_hex, expected_pubkey_hex);

    let sig_hex = sign_hex(&signing_key, message);
    assert_eq!(sig_hex, expected_sig_hex);

    let verify_result = verify_hex(expected_pubkey_hex, message, expected_sig_hex);
    assert!(verify_result.is_ok(), "RFC 8032 Test 1 verification failed: {:?}", verify_result);
}

#[test]
fn test_ed25519_rfc8032_test_vector_2_single_byte_message() {
    // RFC 8032 Section 7.1 - TEST 2
    let secret_hex = "4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb";
    let expected_pubkey_hex = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    let message = &[0x72u8]; // 'r'
    let expected_sig_hex = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";

    let signing_key = signing_key_from_hex(secret_hex).expect("valid secret key hex");
    let pubkey = signing_key.verifying_key();
    let pubkey_hex = verifying_key_to_hex(&pubkey);
    assert_eq!(pubkey_hex, expected_pubkey_hex);

    let sig_hex = sign_hex(&signing_key, message);
    assert_eq!(sig_hex, expected_sig_hex);

    let verify_result = verify_hex(expected_pubkey_hex, message, expected_sig_hex);
    assert!(verify_result.is_ok(), "RFC 8032 Test 2 verification failed: {:?}", verify_result);
}

#[test]
fn test_ed25519_rfc8032_test_vector_3_two_byte_message() {
    // RFC 8032 Section 7.1 - TEST 3
    let secret_hex = "c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7";
    let expected_pubkey_hex = "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";
    let message = &[0xafu8, 0x82u8];
    let expected_sig_hex = "6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290ae67f760984dc6594a7c15e9716ed28dc027beceea1ec40a";

    let signing_key = signing_key_from_hex(secret_hex).expect("valid secret key hex");
    let pubkey = signing_key.verifying_key();
    let pubkey_hex = verifying_key_to_hex(&pubkey);
    assert_eq!(pubkey_hex, expected_pubkey_hex);

    let sig_hex = sign_hex(&signing_key, message);
    assert_eq!(sig_hex, expected_sig_hex);

    let verify_result = verify_hex(expected_pubkey_hex, message, expected_sig_hex);
    assert!(verify_result.is_ok(), "RFC 8032 Test 3 verification failed: {:?}", verify_result);
}

// =========================================================================
// 2. BLAKE3 vs Official BLAKE3 Team Test Vectors
// =========================================================================

#[test]
fn test_blake3_official_test_vectors() {
    let key = b"whats the Elvish word for friend";
    let context_string = "BLAKE3 2019-12-27 16:29:52 test vectors context";

    let get_input = |len: usize| -> Vec<u8> {
        let mut buf = Vec::with_capacity(len);
        for i in 0..len {
            buf.push((i % 251) as u8);
        }
        buf
    };

    let cases = [
        (0, "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
            "92b2b75604ed3c761f9d6f62392c8a9227ad0ea3f09573e783f1498a4ed60d26",
            "2cc39783c223154fea8dfb7c1b1660f2ac2dcbd1c1de8277b0b0dd39b7e50d7d"),
        (1, "2d3adedff11b61f14c886e35afa036736dcd87a74d27b5c1510225d0f592e213",
            "6d7878dfff2f485635d39013278ae14f1454b8c0a3a2d34bc1ab38228a80c95b",
            "b3e2e340a117a499c6cf2398a19ee0d29cca2bb7404c73063382693bf66cb06c"),
        (2, "7b7015bb92cf0b318037702a6cdd81dee41224f734684c2c122cd6359cb1ee63",
            "5392ddae0e0a69d5f40160462cbd9bd889375082ff224ac9c758802b7a6fd20a",
            "1f166565a7df0098ee65922d7fea425fb18b9943f19d6161e2d17939356168e6"),
        (3, "e1be4d7a8ab5560aa4199eea339849ba8e293d55ca0a81006726d184519e647f",
            "39e67b76b5a007d4921969779fe666da67b5213b096084ab674742f0d5ec62b9",
            "440aba35cb006b61fc17c0529255de438efc06a8c9ebf3f2ddac3b5a86705797"),
    ];

    for (len, exp_hash, exp_keyed, exp_derived) in cases {
        let input = get_input(len);

        let h = blake3_hash_hex(&input);
        assert_eq!(h, exp_hash, "BLAKE3 hash failed for len {len}");

        let mut key32 = [0u8; 32];
        key32.copy_from_slice(key);
        let kh = blake3_keyed_hash_hex(&key32, &input);
        assert_eq!(kh, exp_keyed, "BLAKE3 keyed_hash failed for len {len}");

        let dk = derive_key(context_string, &input);
        let mut dk_hex = String::with_capacity(64);
        for b in dk {
            use std::fmt::Write;
            write!(&mut dk_hex, "{:02x}", b).unwrap();
        }
        assert_eq!(dk_hex, exp_derived, "BLAKE3 derive_key failed for len {len}");
    }
}

// =========================================================================
// 3. RFC 8785 JSON Canonicalization Scheme (JCS) Verification
// =========================================================================

#[test]
fn test_jcs_case1_utf16_surrogate_key_sorting() {
    let raw = r#"{
        "\uE000": 1,
        "\uD83D\uDCA9": 2,
        "a": 3,
        "z": 4
    }"#;
    let canonical = canonicalize_json_str(raw).expect("canonicalize should succeed");
    let poo = "\u{1F4A9}";
    let expected = format!(r#"{{"a":3,"z":4,"{}":2,"{}":1}}"#, poo, "\u{e000}");
    assert_eq!(canonical, expected);
}

#[test]
fn test_jcs_case2_number_normalization() {
    let raw = r#"{
        "int": 42,
        "float_zero": 0.0,
        "neg_zero": -0.0,
        "int_float": 100.0,
        "scientific": 1e6,
        "neg_scientific": -2.5e-3
    }"#;
    let canonical = canonicalize_json_str(raw).expect("canonicalize should succeed");
    assert!(canonical.contains(r#""float_zero":0"#));
    assert!(canonical.contains(r#""neg_zero":0"#));
    assert!(canonical.contains(r#""int_float":100"#));
    assert!(canonical.contains(r#""scientific":1000000"#));
    assert!(canonical.contains(r#""neg_scientific":-0.0025"#));
}

#[test]
fn test_jcs_case3_string_minimal_escaping() {
    let raw = r#"{
        "url": "https:\/\/example.com\/path\/to\/resource",
        "escapes": "quote: \" backslash: \\ newline: \n tab: \t",
        "control": "\u0000\u001f"
    }"#;
    let canonical = canonicalize_json_str(raw).expect("canonicalize should succeed");
    assert!(canonical.contains(r#""url":"https://example.com/path/to/resource""#));
    assert!(canonical.contains(r#""escapes":"quote: \" backslash: \\ newline: \n tab: \t""#));
    assert!(canonical.contains(r#""control":"\u0000\u001f""#));
}

#[test]
fn test_jcs_case4_deeply_nested_empty_structures() {
    let raw = r#"{
        "empty_arr": [   ],
        "empty_obj": {   },
        "nested": {
            "arr": [ {}, [], { "inner": [] } ]
        }
    }"#;
    let canonical = canonicalize_json_str(raw).expect("canonicalize should succeed");
    let expected = r#"{"empty_arr":[],"empty_obj":{},"nested":{"arr":[{},[],{"inner":[]}]}}"#;
    assert_eq!(canonical, expected);
}

#[test]
fn test_jcs_case5_unicode_keys_and_negative_numbers() {
    let raw = r#"{
        "яблоко": -99.9,
        "апельсин": -0.0001,
        "банан": [ -100, -0.0, 0 ]
    }"#;
    let canonical = canonicalize_json_str(raw).expect("canonicalize should succeed");
    let expected = r#"{"апельсин":-0.0001,"банан":[-100,0,0],"яблоко":-99.9}"#;
    assert_eq!(canonical, expected);
}

// =========================================================================
// 4. Ratchet Anti-Shrink Adversarial Bypass Verification
// =========================================================================

fn make_dummy_manifest(tests: Vec<(&str, &str, usize)>) -> BaselineManifest {
    let test_items = tests
        .into_iter()
        .map(|(id, fp, ac)| TestItemBaseline {
            id: id.to_string(),
            name: id.to_string(),
            fingerprint: fp.to_string(),
            assertion_count: ac,
        })
        .collect::<Vec<_>>();

    let total = test_items.len();
    let file = TestFileBaseline {
        path: "tests/suite.rs".to_string(),
        language: TestLanguage::Rust,
        tier: TestTier::Tier1Unit,
        blake3_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        tests: test_items,
    };

    BaselineManifest {
        schema_version: 1,
        config: BaselineConfig {
            min_assertions_per_test: 1,
        },
        ratchet: RatchetState {
            total_tests: total,
            files_count: 1,
        },
        files: vec![file],
        signature: None,
    }
}

const FP_1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const FP_2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const FP_3: &str = "3333333333333333333333333333333333333333333333333333333333333333";

#[test]
fn test_ratchet_bypass_attempt_1_direct_test_deletion_without_allow_shrink() {
    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2), ("t3", FP_3, 2)]);
    let current = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let opts = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must reject test deletion without allow_shrink");
    match res.unwrap_err() {
        RatchetError::MissingTests(missing) => assert_eq!(missing, vec!["t3"]),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ratchet_bypass_attempt_2_test_renaming_substitution_without_allow_shrink() {
    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let current = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2_new", FP_2, 2)]);
    let opts = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must detect renamed test as deletion of old test");
    match res.unwrap_err() {
        RatchetError::MissingTests(missing) => assert_eq!(missing, vec!["t2"]),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ratchet_bypass_attempt_3_counter_inflation_fraud_with_missing_tests() {
    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let mut current = make_dummy_manifest(vec![("t1", FP_1, 2)]);
    current.ratchet.total_tests = 5;

    let opts = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must reject even if total_tests counter is artificially inflated");
    match res.unwrap_err() {
        RatchetError::MissingTests(missing) => assert_eq!(missing, vec!["t2"]),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ratchet_bypass_attempt_4_shrink_with_allow_shrink_flag_but_no_signature() {
    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let current = make_dummy_manifest(vec![("t1", FP_1, 2)]);
    let opts = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must reject shrink without valid cryptographic signature");
    match res.unwrap_err() {
        RatchetError::ShrinkRequiresSignature => {}
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ratchet_bypass_attempt_5_shrink_with_forged_untrusted_signature() {
    let (attacker_sign, attacker_verify) = generate_keypair();
    let (_trusted_sign, trusted_verify) = generate_keypair();

    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let mut current = make_dummy_manifest(vec![("t1", FP_1, 2)]);
    current.sign(&attacker_sign).expect("signing succeeds");

    let opts = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&trusted_verify)],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must reject shrink signed by untrusted attacker key");
    match res.unwrap_err() {
        RatchetError::UntrustedKey(key) => assert_eq!(key, verifying_key_to_hex(&attacker_verify)),
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn test_ratchet_bypass_attempt_6_corrupted_signature_with_allow_shrink() {
    let (signing_key, verifying_key) = generate_keypair();

    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let mut current = make_dummy_manifest(vec![("t1", FP_1, 2)]);
    current.sign(&signing_key).expect("signing succeeds");

    if let Some(ref mut sig) = current.signature {
        sig.signature = "00".repeat(64);
    }

    let opts = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&verifying_key)],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(
        matches!(res, Err(RatchetError::InvalidSignature(_))),
        "Ratchet must reject corrupted signature"
    );
}

#[test]
fn test_ratchet_bypass_attempt_7_signature_replay_attack() {
    let (signing_key, verifying_key) = generate_keypair();

    let mut baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    baseline.sign(&signing_key).expect("signing baseline succeeds");
    let stolen_sig = baseline.signature.clone().unwrap();

    let mut current = make_dummy_manifest(vec![("t1", FP_1, 2)]);
    current.signature = Some(stolen_sig);

    let opts = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&verifying_key)],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(
        matches!(res, Err(RatchetError::InvalidSignature(_))),
        "Ratchet must reject replayed signature over modified canonical content"
    );
}

#[test]
fn test_ratchet_bypass_attempt_8_sybil_substitution_test_count_invariant() {
    let baseline = make_dummy_manifest(vec![("t1", FP_1, 2), ("t2", FP_2, 2)]);
    let current = make_dummy_manifest(vec![("t1", FP_1, 2), ("t_dummy", FP_3, 2)]);
    assert_eq!(baseline.ratchet.total_tests, current.ratchet.total_tests);

    let opts = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let res = verify_ratchet(&baseline, &current, &opts);
    assert!(res.is_err(), "Ratchet must catch deleted test even if total_tests count is conserved");
    match res.unwrap_err() {
        RatchetError::MissingTests(missing) => assert_eq!(missing, vec!["t2"]),
        other => panic!("Unexpected error: {:?}", other),
    }
}

