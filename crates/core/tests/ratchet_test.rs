// allow: SIZE_OK — comprehensive 15-case ratchet engine integration test suite
use agent_test_guard_core::crypto::*;
use agent_test_guard_core::ratchet::*;
use agent_test_guard_core::schema::*;

const FP_ONE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const FP_TWO: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const FP_THREE: &str = "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff";
const FP_FOUR: &str = "4444555566667777888899990000111122223333aaaabbbbccccddddeeeeffff";
const FP_MODIFIED: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const FILE_HASH_1: &str = "deadbeefcafebabe0123456789abcdefdeadbeefcafebabe0123456789abcdef";
const FILE_HASH_2: &str = "cafebabedeadbeef0123456789abcdefcafebabedeadbeef0123456789abcdef";

fn make_test_item(id: &str, name: &str, fingerprint: &str) -> TestItemBaseline {
    TestItemBaseline {
        id: id.to_string(),
        name: name.to_string(),
        fingerprint: fingerprint.to_string(),
        assertion_count: 1,
    }
}

fn make_test_file(path: &str, tests: Vec<TestItemBaseline>, blake3_hash: &str) -> TestFileBaseline {
    TestFileBaseline {
        path: path.to_string(),
        language: TestLanguage::Rust,
        tier: TestTier::Tier1Unit,
        blake3_hash: blake3_hash.to_string(),
        tests,
    }
}

fn make_manifest(files: Vec<TestFileBaseline>) -> BaselineManifest {
    let total_tests = files.iter().map(|f| f.tests.len()).sum();
    let files_count = files.len();
    BaselineManifest {
        schema_version: 1,
        config: BaselineConfig {
            min_assertions_per_test: 1,
        },
        ratchet: RatchetState {
            total_tests,
            files_count,
        },
        files,
        signature: None,
    }
}

fn sample_baseline_manifest() -> BaselineManifest {
    let test1 = make_test_item("tests/core_test.rs::test_one", "test_one", FP_ONE);
    let test2 = make_test_item("tests/core_test.rs::test_two", "test_two", FP_TWO);
    let test3 = make_test_item("tests/core_test.rs::test_three", "test_three", FP_THREE);
    let file = make_test_file("tests/core_test.rs", vec![test1, test2, test3], FILE_HASH_1);
    make_manifest(vec![file])
}

#[test]
fn test_verify_ratchet_when_identical_current_and_baseline() {
    // Given: identical baseline and current manifests
    let baseline = sample_baseline_manifest();
    let current = sample_baseline_manifest();
    let options = RatchetOptions::default();

    // When: verifying ratchet between identical manifests
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification succeeds with zero diff and test count unchanged
    let diff = result.expect("ratchet verification should succeed for identical manifests");
    assert_eq!(diff.added.len(), 0, "expected 0 added tests");
    assert_eq!(diff.removed.len(), 0, "expected 0 removed tests");
    assert_eq!(diff.modified.len(), 0, "expected 0 modified tests");
    assert_eq!(
        diff.baseline_total, diff.current_total,
        "test count must remain unchanged"
    );
    assert_eq!(diff.baseline_total, 3);
}

#[test]
fn test_verify_ratchet_when_monotonic_addition_of_tests() {
    // Given: baseline with 3 tests and current with 4 tests (test_four added)
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    let test4 = make_test_item("tests/core_test.rs::test_four", "test_four", FP_FOUR);
    current.files[0].tests.push(test4);
    current.ratchet.total_tests = 4;
    let options = RatchetOptions::default();

    // When: verifying ratchet with newly added test
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification succeeds, detecting added test and increased count
    let diff = result.expect("ratchet verification should succeed when adding tests");
    assert_eq!(diff.added.len(), 1, "expected 1 added test");
    assert_eq!(diff.added[0].id, "tests/core_test.rs::test_four");
    assert_eq!(diff.removed.len(), 0, "expected 0 removed tests");
    assert_eq!(diff.modified.len(), 0, "expected 0 modified tests");
    assert_eq!(diff.baseline_total, 3);
    assert_eq!(diff.current_total, 4);
    assert!(
        diff.current_total > diff.baseline_total,
        "current count must be strictly greater than baseline count"
    );
}

#[test]
fn test_verify_ratchet_fails_when_test_deleted_without_allow_shrink() {
    // Given: baseline with 3 tests, current with test_three deleted and allow_shrink = false
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;
    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: Vec::new(),
    };

    // When: verifying ratchet without shrink authorization
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with MissingTests containing test_three
    match result {
        Err(RatchetError::MissingTests(missing)) => {
            assert!(
                missing.contains(&"tests/core_test.rs::test_three".to_string()),
                "missing tests must contain deleted test_three"
            );
        }
        other => panic!(
            "expected RatchetError::MissingTests when test deleted without allow_shrink, got {other:?}"
        ),
    }
}

#[test]
fn test_verify_ratchet_fails_when_test_count_decreased_without_allow_shrink() {
    // Given: baseline with 3 tests, current with decreased count to 2 without allow_shrink
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.truncate(2);
    current.ratchet.total_tests = 2;
    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: Vec::new(),
    };

    // When: verifying ratchet with decreased test count
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with TestCountDecreased or MissingTests
    assert!(
        matches!(
            result,
            Err(RatchetError::TestCountDecreased { .. } | RatchetError::MissingTests(..))
        ),
        "expected TestCountDecreased or MissingTests when count decreases without allow_shrink"
    );
}

#[test]
fn test_verify_ratchet_fails_when_sybil_substitution_attack_detected() {
    // Given: baseline with {test_one, test_two, test_three} (count = 3).
    // Current has {test_one, test_two, test_sybil} (count = 3).
    // Total count is preserved, but test_three was substituted by test_sybil.
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    let test_sybil = make_test_item("tests/core_test.rs::test_sybil", "test_sybil", FP_FOUR);
    current.files[0].tests.push(test_sybil);
    assert_eq!(baseline.ratchet.total_tests, current.ratchet.total_tests);

    let options = RatchetOptions::default();

    // When: verifying ratchet under Sybil substitution attack
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification MUST fail with MissingTests containing test_three
    match result {
        Err(RatchetError::MissingTests(missing)) => {
            assert!(
                missing.contains(&"tests/core_test.rs::test_three".to_string()),
                "missing tests must explicitly flag missing test_three in Sybil attack"
            );
        }
        other => panic!("expected RatchetError::MissingTests containing test_three, got {other:?}"),
    }
}

#[test]
fn test_verify_ratchet_fails_when_entire_file_deleted_without_allow_shrink() {
    // Given: baseline with two test files (file1 with 2 tests, file2 with 1 test).
    // Current deletes file2 entirely.
    let test1 = make_test_item("tests/file1.rs::test_one", "test_one", FP_ONE);
    let test2 = make_test_item("tests/file1.rs::test_two", "test_two", FP_TWO);
    let file1 = make_test_file("tests/file1.rs", vec![test1, test2], FILE_HASH_1);

    let test3 = make_test_item("tests/file2.rs::test_three", "test_three", FP_THREE);
    let file2 = make_test_file("tests/file2.rs", vec![test3], FILE_HASH_2);

    let baseline = make_manifest(vec![file1.clone(), file2]);
    let current = make_manifest(vec![file1]);
    let options = RatchetOptions::default();

    // When: verifying ratchet when an entire file was deleted without allow_shrink
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with MissingTests containing test_three from deleted file
    match result {
        Err(RatchetError::MissingTests(missing)) => {
            assert!(
                missing.contains(&"tests/file2.rs::test_three".to_string()),
                "missing tests must contain tests from deleted file"
            );
        }
        other => panic!("expected RatchetError::MissingTests for deleted file, got {other:?}"),
    }
}

#[test]
fn test_verify_ratchet_fails_when_allow_shrink_true_without_signature() {
    // Given: baseline with 3 tests, current with 2 tests (shrink), allow_shrink = true, but current is unsigned
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;
    current.signature = None;

    let options = RatchetOptions {
        allow_shrink: true,
        trusted_keys: Vec::new(),
    };

    // When: verifying authorized shrink without cryptographic signature
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with ShrinkRequiresSignature
    assert!(
        matches!(result, Err(RatchetError::ShrinkRequiresSignature)),
        "expected ShrinkRequiresSignature when allow_shrink is true but signature is missing"
    );
}

#[test]
fn test_verify_ratchet_fails_when_allow_shrink_true_with_invalid_signature() {
    // Given: baseline with 3 tests, current with 2 tests, allow_shrink = true,
    // signed by a valid key, but signature is corrupted
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;

    let (signing_key, verifying_key) = generate_keypair();
    current
        .sign(&signing_key)
        .expect("signing current manifest should succeed");

    // Corrupt the signature hex
    if let Some(sig) = &mut current.signature {
        let mut corrupted = sig.signature.clone();
        corrupted.replace_range(
            0..2,
            if corrupted.starts_with("aa") {
                "bb"
            } else {
                "aa"
            },
        );
        sig.signature = corrupted;
    }

    let options = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&verifying_key)],
    };

    // When: verifying authorized shrink with corrupted signature
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with InvalidSignature
    assert!(
        matches!(result, Err(RatchetError::InvalidSignature(..))),
        "expected InvalidSignature when signature is corrupted"
    );
}

#[test]
fn test_verify_ratchet_fails_when_allow_shrink_true_with_untrusted_public_key() {
    // Given: baseline with 3 tests, current with 2 tests, allow_shrink = true,
    // signed by keypair B, but trusted_keys only trusts keypair A
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;

    let (_trusted_signing_key, trusted_verifying_key) = generate_keypair();
    let (untrusted_signing_key, _untrusted_verifying_key) = generate_keypair();

    current
        .sign(&untrusted_signing_key)
        .expect("signing with untrusted key should succeed");

    let options = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&trusted_verifying_key)],
    };

    // When: verifying authorized shrink signed with an untrusted public key
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification fails with UntrustedKey
    assert!(
        matches!(result, Err(RatchetError::UntrustedKey(..))),
        "expected UntrustedKey when public key is not in trusted_keys list"
    );
}

#[test]
fn test_verify_ratchet_succeeds_when_allow_shrink_true_with_valid_trusted_signature() {
    // Given: baseline with 3 tests, current with 2 tests (test_three removed),
    // allow_shrink = true, signed by a trusted developer key
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;

    let (signing_key, verifying_key) = generate_keypair();
    current
        .sign(&signing_key)
        .expect("signing current manifest should succeed");

    let options = RatchetOptions {
        allow_shrink: true,
        trusted_keys: vec![verifying_key_to_hex(&verifying_key)],
    };

    // When: verifying authorized shrink with valid signature and trusted key
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification succeeds with diff reflecting removed test_three
    let diff = result.expect("ratchet verification should succeed for signed authorized shrink");
    assert_eq!(diff.removed.len(), 1, "expected 1 removed test in diff");
    assert_eq!(diff.removed[0].id, "tests/core_test.rs::test_three");
    assert_eq!(diff.added.len(), 0, "expected 0 added tests");
    assert_eq!(diff.baseline_total, 3);
    assert_eq!(diff.current_total, 2);
    assert!(
        diff.current_total < diff.baseline_total,
        "current count must be strictly less than baseline count on authorized shrink"
    );
}

#[test]
fn test_verify_ratchet_tracks_modified_tests_when_ast_fingerprint_changes() {
    // Given: baseline with test_one having fingerprint FP_ONE.
    // Current has test_one with same ID but modified fingerprint FP_MODIFIED.
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests[0].fingerprint = FP_MODIFIED.to_string();
    let options = RatchetOptions::default();

    // When: verifying ratchet when an AST fingerprint has changed
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification succeeds and diff records modified test
    let diff = result.expect("ratchet verification should succeed when fingerprint changes");
    assert_eq!(diff.modified.len(), 1, "expected 1 modified test");
    assert_eq!(diff.modified[0].id, "tests/core_test.rs::test_one");
    assert_eq!(diff.modified[0].old_fingerprint, FP_ONE);
    assert_eq!(diff.modified[0].new_fingerprint, FP_MODIFIED);
    assert_eq!(diff.added.len(), 0, "no tests added");
    assert_eq!(diff.removed.len(), 0, "no tests removed");
    assert_eq!(diff.baseline_total, diff.current_total);
}

#[test]
fn test_advance_baseline_succeeds_when_adding_tests_and_signs_if_key_provided() {
    // Given: baseline with 3 tests, current with 4 tests (test_four added)
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    let test4 = make_test_item("tests/core_test.rs::test_four", "test_four", FP_FOUR);
    current.files[0].tests.push(test4);
    current.ratchet.total_tests = 4;
    let options = RatchetOptions::default();

    let (signing_key, _verifying_key) = generate_keypair();

    // When: advancing baseline with a SigningKey provided
    let advanced_signed = advance_baseline(&baseline, &current, &options, Some(&signing_key))
        .expect("advance_baseline should succeed with SigningKey");

    // Then: updated counts match current, tests are preserved, and signature is verified
    assert_eq!(advanced_signed.ratchet.total_tests, 4);
    assert_eq!(advanced_signed.files[0].tests.len(), 4);
    assert!(
        advanced_signed.signature.is_some(),
        "signature must be present when SigningKey is provided"
    );
    let sig_valid = advanced_signed
        .verify_signature()
        .expect("signature verification should execute");
    assert!(
        sig_valid,
        "signature on advanced baseline must be cryptographically valid"
    );

    // When: advancing baseline without a SigningKey (None)
    let advanced_unsigned = advance_baseline(&baseline, &current, &options, None)
        .expect("advance_baseline should succeed without SigningKey");

    // Then: updated counts match current and signature is None
    assert_eq!(advanced_unsigned.ratchet.total_tests, 4);
    assert!(
        advanced_unsigned.signature.is_none(),
        "signature must be None when no SigningKey is provided"
    );
}

#[test]
fn test_advance_baseline_fails_when_unauthorized_shrink_attempted() {
    // Given: baseline with 3 tests, current with 2 tests (shrink attempted) and allow_shrink = false
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests.pop();
    current.ratchet.total_tests = 2;
    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: Vec::new(),
    };

    // When: attempting to advance baseline with unauthorized shrink
    let result = advance_baseline(&baseline, &current, &options, None);

    // Then: advance_baseline fails with MissingTests or TestCountDecreased
    assert!(
        matches!(
            result,
            Err(RatchetError::MissingTests(..) | RatchetError::TestCountDecreased { .. })
        ),
        "advance_baseline must reject unauthorized shrink"
    );

    // When: allow_shrink = true but current has no signature
    let options_shrink_no_sig = RatchetOptions {
        allow_shrink: true,
        trusted_keys: Vec::new(),
    };
    let result_no_sig = advance_baseline(&baseline, &current, &options_shrink_no_sig, None);

    // Then: advance_baseline fails with ShrinkRequiresSignature
    assert!(
        matches!(result_no_sig, Err(RatchetError::ShrinkRequiresSignature)),
        "advance_baseline must reject shrink when signature is missing"
    );
}

#[test]
fn test_verify_ratchet_when_tests_reordered_in_file() {
    // Given: baseline with tests [test_one, test_two, test_three] and current with [test_three, test_one, test_two]
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    let t1 = current.files[0].tests[0].clone();
    let t2 = current.files[0].tests[1].clone();
    let t3 = current.files[0].tests[2].clone();
    current.files[0].tests = vec![t3, t1, t2];
    let options = RatchetOptions::default();

    // When: verifying ratchet when tests are merely reordered
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: verification succeeds with 0 added, 0 removed, 0 modified
    let diff = result.expect("reordered tests must be accepted by ratchet");
    assert_eq!(diff.added.len(), 0);
    assert_eq!(diff.removed.len(), 0);
    assert_eq!(diff.modified.len(), 0);
    assert_eq!(diff.baseline_total, diff.current_total);
}

#[test]
fn test_verify_ratchet_when_simultaneous_addition_and_fingerprint_modification() {
    // Given: baseline with test_one (FP_ONE), test_two (FP_TWO), test_three (FP_THREE).
    // Current modifies test_one to FP_MODIFIED and adds test_four (FP_FOUR).
    let baseline = sample_baseline_manifest();
    let mut current = sample_baseline_manifest();
    current.files[0].tests[0].fingerprint = FP_MODIFIED.to_string();
    let test4 = make_test_item("tests/core_test.rs::test_four", "test_four", FP_FOUR);
    current.files[0].tests.push(test4);
    current.ratchet.total_tests = 4;
    let options = RatchetOptions::default();

    // When: verifying ratchet with both modified and added tests
    let result = verify_ratchet(&baseline, &current, &options);

    // Then: diff records both addition and modification accurately
    let diff = result.expect("verification should succeed when tests are added and modified");
    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.added[0].id, "tests/core_test.rs::test_four");
    assert_eq!(diff.modified.len(), 1);
    assert_eq!(diff.modified[0].id, "tests/core_test.rs::test_one");
    assert_eq!(diff.modified[0].old_fingerprint, FP_ONE);
    assert_eq!(diff.modified[0].new_fingerprint, FP_MODIFIED);
    assert_eq!(diff.removed.len(), 0);
    assert_eq!(diff.baseline_total, 3);
    assert_eq!(diff.current_total, 4);
}
