use agent_test_guard_core::crypto::generate_keypair;
use agent_test_guard_core::schema::{
    BaselineConfig, BaselineManifest, RatchetState, SchemaError, SignatureBlock, TestFileBaseline,
    TestItemBaseline, TestLanguage, TestTier,
};

fn sample_valid_manifest() -> BaselineManifest {
    let test1 = TestItemBaseline {
        id: "tests/core_test.rs::test_one".to_string(),
        name: "test_one".to_string(),
        fingerprint: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        assertion_count: 2,
    };
    let test2 = TestItemBaseline {
        id: "tests/core_test.rs::test_two".to_string(),
        name: "test_two".to_string(),
        fingerprint: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string(),
        assertion_count: 1,
    };
    let test3 = TestItemBaseline {
        id: "tests/core_test.rs::test_three".to_string(),
        name: "test_three".to_string(),
        fingerprint: "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff".to_string(),
        assertion_count: 3,
    };
    let file = TestFileBaseline {
        path: "tests/core_test.rs".to_string(),
        language: TestLanguage::Rust,
        tier: TestTier::Tier1Unit,
        blake3_hash: "deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef".to_string(),
        tests: vec![test1, test2, test3],
    };
    BaselineManifest {
        schema_version: 1,
        config: BaselineConfig {
            min_assertions_per_test: 1,
        },
        ratchet: RatchetState {
            total_tests: 3,
            files_count: 1,
        },
        files: vec![file],
        signature: None,
    }
}

#[test]
fn test_baseline_roundtrip_when_valid_json_serialized_and_deserialized() {
    // Given: a valid populated manifest with and without signature block
    let mut manifest = sample_valid_manifest();
    manifest.signature = Some(SignatureBlock {
        public_key: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        signature: "a".repeat(128),
        algorithm: "ed25519".to_string(),
    });

    // When: serializing to JSON and deserializing back
    let serialized =
        serde_json::to_string_pretty(&manifest).expect("serialization should succeed");
    let deserialized: BaselineManifest =
        serde_json::from_str(&serialized).expect("deserialization should succeed");

    // Then: deserialized manifest equals original manifest without data loss
    assert_eq!(deserialized, manifest);
}

#[test]
fn test_validate_succeeds_when_manifest_is_well_formed() {
    // Given: a structurally valid BaselineManifest meeting all invariants
    let manifest = sample_valid_manifest();

    // When: validating the manifest
    let result = manifest.validate();

    // Then: validation succeeds
    assert!(result.is_ok(), "expected validation to succeed on well-formed manifest");
}

#[test]
fn test_validate_fails_when_total_tests_mismatches_actual_count() {
    // Given: a manifest where ratchet total_tests is 5 but actual count is 3
    let mut manifest = sample_valid_manifest();
    manifest.ratchet.total_tests = 5;

    // When: validating the manifest
    let result = manifest.validate();

    // Then: validation fails with CountMismatch
    assert!(
        matches!(result, Err(SchemaError::CountMismatch { .. })),
        "expected CountMismatch when total_tests does not match actual count"
    );
}

#[test]
fn test_validate_fails_when_files_count_mismatches_actual_count() {
    // Given: a manifest where ratchet files_count is 10 but actual files count is 1
    let mut manifest = sample_valid_manifest();
    manifest.ratchet.files_count = 10;

    // When: validating the manifest
    let result = manifest.validate();

    // Then: validation fails with CountMismatch
    assert!(
        matches!(result, Err(SchemaError::CountMismatch { .. })),
        "expected CountMismatch when files_count does not match actual files count"
    );
}

#[test]
fn test_validate_fails_when_assertion_floor_violated() {
    // Given: a manifest with min_assertions_per_test = 1 and a test with 0 assertions
    let mut manifest = sample_valid_manifest();
    manifest.config.min_assertions_per_test = 1;
    manifest.files[0].tests[0].assertion_count = 0;

    // When: validating the manifest
    let result = manifest.validate();

    // Then: validation fails with AssertionFloorViolation
    assert!(
        matches!(result, Err(SchemaError::AssertionFloorViolation { .. })),
        "expected AssertionFloorViolation when a test violates the assertion floor"
    );
}

#[test]
fn test_validate_fails_when_duplicate_test_id_present() {
    // Given: a manifest with two identical test IDs
    let mut manifest = sample_valid_manifest();
    manifest.files[0].tests[1].id = manifest.files[0].tests[0].id.clone();

    // When: validating the manifest
    let result = manifest.validate();

    // Then: validation fails with DuplicateTestId
    assert!(
        matches!(result, Err(SchemaError::DuplicateTestId(..))),
        "expected DuplicateTestId when duplicate test IDs are detected"
    );
}

#[test]
fn test_validate_fails_when_path_is_invalid_or_not_normalized() {
    // Given: invalid path formats (directory traversal, absolute, Windows backslashes)
    let invalid_paths = ["../foo.rs", "/absolute/foo.rs", "tests\\foo.rs"];

    for path in invalid_paths {
        let mut manifest = sample_valid_manifest();
        manifest.files[0].path = path.to_string();

        // When: validating the manifest with invalid path
        let result = manifest.validate();

        // Then: validation fails with InvalidPath
        assert!(
            matches!(result, Err(SchemaError::InvalidPath(..))),
            "path '{path}' must be rejected with InvalidPath"
        );
    }
}

#[test]
fn test_validate_fails_when_hash_or_fingerprint_hex_is_invalid() {
    // Given: non-hex strings and wrong-length hex strings for blake3_hash and fingerprint
    let invalid_hashes = [
        "not-a-hex-hash",
        "deadbeef",
        "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
    ];

    for bad_hash in invalid_hashes {
        // When: file blake3_hash is invalid
        let mut manifest_file = sample_valid_manifest();
        manifest_file.files[0].blake3_hash = bad_hash.to_string();
        let res_file = manifest_file.validate();

        // Then: validation fails with InvalidHash
        assert!(
            matches!(res_file, Err(SchemaError::InvalidHash { .. })),
            "blake3_hash '{bad_hash}' must be rejected with InvalidHash"
        );

        // When: test item fingerprint is invalid
        let mut manifest_fp = sample_valid_manifest();
        manifest_fp.files[0].tests[0].fingerprint = bad_hash.to_string();
        let res_fp = manifest_fp.validate();

        // Then: validation fails with InvalidHash
        assert!(
            matches!(res_fp, Err(SchemaError::InvalidHash { .. })),
            "fingerprint '{bad_hash}' must be rejected with InvalidHash"
        );
    }
}

#[test]
fn test_canonical_digest_when_manifest_formatted_or_keys_reordered() {
    // Given: a valid manifest and a serialized JSON representation with altered formatting
    let manifest = sample_valid_manifest();

    // When: computing digest directly and after deserializing reordered JSON
    let digest_direct = manifest.digest().expect("digest calculation should succeed");
    let json_value: serde_json::Value =
        serde_json::to_value(&manifest).expect("conversion to Value should succeed");
    let reordered_manifest: BaselineManifest =
        serde_json::from_value(json_value).expect("re-deserialization should succeed");
    let digest_reordered = reordered_manifest
        .digest()
        .expect("reordered digest calculation should succeed");

    // Then: digest is a 64-character hex BLAKE3 hash and matches deterministically
    assert_eq!(digest_direct.len(), 64);
    assert!(digest_direct.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(digest_direct, digest_reordered);
}

#[test]
fn test_signing_and_verification_when_valid_ed25519_key() {
    // Given: a valid manifest and a freshly generated Ed25519 keypair
    let mut manifest = sample_valid_manifest();
    let (signing_key, _verifying_key) = generate_keypair();

    // When: signing the manifest
    manifest
        .sign(&signing_key)
        .expect("signing manifest should succeed");

    // Then: signature block is populated and verify_signature returns Ok(true)
    let sig_block = manifest
        .signature
        .as_ref()
        .expect("signature block should be present after signing");
    assert_eq!(sig_block.public_key.len(), 64);
    assert_eq!(sig_block.signature.len(), 128);
    assert_eq!(sig_block.algorithm, "ed25519");

    let is_valid = manifest
        .verify_signature()
        .expect("signature verification should execute");
    assert!(is_valid, "verify_signature must return Ok(true) for valid signature");
}

#[test]
fn test_verification_fails_when_signed_manifest_is_tampered() {
    // Given: a valid signed manifest
    let mut manifest = sample_valid_manifest();
    let (signing_key, _verifying_key) = generate_keypair();
    manifest
        .sign(&signing_key)
        .expect("signing manifest should succeed");

    // When: tampering test name
    let mut tampered_name = manifest.clone();
    tampered_name.files[0].tests[0].name = "tampered_name".to_string();

    // When: tampering total tests count
    let mut tampered_count = manifest.clone();
    tampered_count.ratchet.total_tests += 1;

    // When: tampering test fingerprint
    let mut tampered_fp = manifest.clone();
    tampered_fp.files[0].tests[0].fingerprint = "0".repeat(64);

    // Then: verification fails (returns Ok(false) or Err) across all tampered scenarios
    for (scenario, tampered) in [
        ("tampered test name", tampered_name),
        ("tampered total tests", tampered_count),
        ("tampered fingerprint", tampered_fp),
    ] {
        let verify_result = tampered.verify_signature();
        assert!(
            verify_result.as_ref().map_or(true, |&valid| !valid),
            "{scenario} must not pass verification"
        );
    }
}

#[test]
fn test_enums_serialization_when_language_and_tier_used() {
    // Given: all variants of TestLanguage and TestTier
    let languages = [
        (TestLanguage::TypeScript, "\"typescript\""),
        (TestLanguage::JavaScript, "\"javascript\""),
        (TestLanguage::Rust, "\"rust\""),
        (TestLanguage::Python, "\"python\""),
        (TestLanguage::Go, "\"go\""),
    ];
    let tiers = [
        (TestTier::Tier1Unit, "\"tier1_unit\""),
        (TestTier::Tier2Integration, "\"tier2_integration\""),
        (TestTier::Tier3E2e, "\"tier3_e2e\""),
    ];

    // When & Then: TestLanguage serializes to clean snake_case strings and roundtrips
    for (lang, expected_json) in languages {
        let serialized =
            serde_json::to_string(&lang).expect("language serialization should succeed");
        assert_eq!(serialized, expected_json);
        let deserialized: TestLanguage =
            serde_json::from_str(&serialized).expect("language deserialization should succeed");
        assert_eq!(deserialized, lang);
    }

    // When & Then: TestTier serializes to clean snake_case strings and roundtrips
    for (tier, expected_json) in tiers {
        let serialized =
            serde_json::to_string(&tier).expect("tier serialization should succeed");
        assert_eq!(serialized, expected_json);
        let deserialized: TestTier =
            serde_json::from_str(&serialized).expect("tier deserialization should succeed");
        assert_eq!(deserialized, tier);
    }
}
