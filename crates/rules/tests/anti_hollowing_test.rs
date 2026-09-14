// allow: SIZE_OK — comprehensive integration test suite for E003 Anti-Hollowing rule

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::diagnostic::RuleCode;
use agent_test_guard_rules::AntiHollowingRule;

#[test]
fn test_rust_valid_test_with_assertions_passes() {
    // Given: valid Rust test with assert_eq! and helper functions without #[test]
    let source = r#"
        fn helper_calculation(a: i32, b: i32) -> i32 {
            a + b
        }

        #[test]
        fn test_real_calculation() {
            let res = helper_calculation(10, 20);
            assert_eq!(res, 30);
            assert!(res > 0);
        }

        #[tokio::test]
        async fn test_async_fetch() {
            let val = async_op().await;
            debug_assert!(val.is_ok());
        }
    "#;

    // When: checking source with default assertion floor (1)
    let diagnostics =
        AntiHollowingRule::check_source("tests/test_math.rs", source, GrammarLanguage::Rust)
            .expect("checking valid Rust source should succeed");

    // Then: 0 diagnostics produced
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_rust_empty_test_body_detected() {
    // Given: Rust test with completely empty body
    let source = r#"
        #[test]
        fn test_hollow_empty() {
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiHollowingRule::check_source("tests/test_empty.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: E003 diagnostic produced for empty test
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("test_hollow_empty"));
    assert!(diag.message.contains("0 assertions"));
    assert!(diag.fix_hint.contains("assertion"));
}

#[test]
fn test_rust_test_without_assertions_detected() {
    // Given: Rust test with statements but 0 assertions
    let source = r#"
        #[test]
        fn test_pretend_work() {
            let x = 1 + 2;
            let _ = format!("Result: {}", x);
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiHollowingRule::check_source("tests/test_no_assert.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: E003 diagnostic produced
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("test_pretend_work"));
    assert!(diag.message.contains("0 assertions"));
}

#[test]
fn test_rust_custom_assertion_floor_enforcement() {
    // Given: Rust test with 1 assertion
    let source = r#"
        #[test]
        fn test_single_check() {
            assert_eq!(2 + 2, 4);
        }
    "#;

    // When: checking with assertion floor of 2
    let rule_floor_2 = AntiHollowingRule::new(2);
    let diagnostics = rule_floor_2
        .evaluate_source("tests/test_floor.rs", source, GrammarLanguage::Rust)
        .expect("checking Rust source should succeed");

    // Then: E003 diagnostic produced because 1 < 2
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("1 assertion"));
    assert!(diag.message.contains("2"));

    // When: checking with assertion floor of 1
    let rule_floor_1 = AntiHollowingRule::new(1);
    let diagnostics_ok = rule_floor_1
        .evaluate_source("tests/test_floor.rs", source, GrammarLanguage::Rust)
        .expect("checking Rust source should succeed");

    // Then: passes with 0 diagnostics
    assert!(diagnostics_ok.is_empty());
}

#[test]
fn test_ts_valid_tests_with_assertions_pass() {
    // Given: valid TypeScript tests using Jest/Vitest expect, assert, and ava t.is
    let source = r#"
        function setup() {
            return { user: "admin", active: true };
        }

        describe("User Account", () => {
            it("validates status", () => {
                const data = setup();
                expect(data.active).toBe(true);
            });

            test("validates node assert", () => {
                const val = 42;
                assert.strictEqual(val, 42);
            });

            test("validates ava style", (t) => {
                t.is(1 + 1, 2);
            });
        });
    "#;

    // When: checking source
    let diagnostics = AntiHollowingRule::check_source(
        "src/user.test.ts",
        source,
        GrammarLanguage::TypeScript,
    )
    .expect("checking valid TypeScript source should succeed");

    // Then: 0 diagnostics produced (describe and setup helper do not trigger)
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_ts_empty_test_body_detected() {
    // Given: TypeScript tests with empty arrow function, function expr, and async arrow
    let source = r#"
        test("empty arrow test", () => {});
        it("empty async test", async () => {});
        test("empty function expr", function() {});
    "#;

    // When: checking source
    let diagnostics = AntiHollowingRule::check_source(
        "src/empty.test.ts",
        source,
        GrammarLanguage::TypeScript,
    )
    .expect("checking TypeScript source should succeed");

    // Then: all 3 empty tests detected
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E003);
        assert!(diag.message.contains("0 assertions"));
        assert!(diag.fix_hint.contains("assertion"));
    }
}

#[test]
fn test_ts_test_without_assertions_detected() {
    // Given: TypeScript test with dummy statements but no assertions
    let source = r#"
        it("simulates activity without verification", () => {
            const temp = 100 * 2;
            console.log("Calculated:", temp);
        });
    "#;

    // When: checking source
    let diagnostics = AntiHollowingRule::check_source(
        "src/hollow.test.ts",
        source,
        GrammarLanguage::TypeScript,
    )
    .expect("checking TypeScript source should succeed");

    // Then: 1 E003 diagnostic produced
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("simulates activity without verification"));
    assert!(diag.message.contains("0 assertions"));
}

#[test]
fn test_ts_custom_assertion_floor_enforcement() {
    // Given: TypeScript test with 1 assertion
    let source = r#"
        test("has one assertion", () => {
            expect(true).toBeDefined();
        });
    "#;

    // When: checking with floor of 3
    let rule = AntiHollowingRule::new(3);
    let diagnostics = rule
        .evaluate_source("src/floor.test.ts", source, GrammarLanguage::TypeScript)
        .expect("checking TypeScript source should succeed");

    // Then: E003 diagnostic produced
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("has one assertion"));
    assert!(diag.message.contains("1 assertion"));
    assert!(diag.message.contains("3"));
}

#[test]
fn test_ts_describe_suite_with_mixed_tests() {
    // Given: describe suite containing 1 valid test and 1 hollow test
    let source = r#"
        describe("Order Processing", () => {
            it("validates payment flow", () => {
                expect(processPayment(50)).toBe(true);
            });

            it("handles refund flow", () => {
                // hollowed out test
                let refundAmount = 50;
            });
        });
    "#;

    // When: checking source
    let diagnostics = AntiHollowingRule::check_source(
        "src/orders.test.ts",
        source,
        GrammarLanguage::TypeScript,
    )
    .expect("checking TypeScript source should succeed");

    // Then: exactly 1 diagnostic for the hollowed test
    assert_eq!(diagnostics.len(), 1);
    let diag = &diagnostics[0];
    assert_eq!(diag.code, RuleCode::E003);
    assert!(diag.message.contains("handles refund flow"));
    assert!(!diag.message.contains("validates payment flow"));
}

#[test]
fn test_multiple_hollow_tests_in_file_detected() {
    // Given: multiple hollow tests across a file
    let source = r#"
        #[test]
        fn test_one_hollow() {}

        #[test]
        fn test_two_valid() {
            assert!(1 > 0);
        }

        #[test]
        fn test_three_hollow() {
            let _ = 42;
        }
    "#;

    // When: checking source
    let diagnostics = AntiHollowingRule::check_source(
        "tests/multi_hollow.rs",
        source,
        GrammarLanguage::Rust,
    )
    .expect("checking Rust source should succeed");

    // Then: tests 1 and 3 are flagged, test 2 is not
    assert_eq!(diagnostics.len(), 2);
    let names: Vec<_> = diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert!(names.iter().any(|m| m.contains("test_one_hollow")));
    assert!(names.iter().any(|m| m.contains("test_three_hollow")));
    assert!(!names.iter().any(|m| m.contains("test_two_valid")));
}
