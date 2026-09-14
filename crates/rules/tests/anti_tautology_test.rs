// allow: SIZE_OK — comprehensive integration test suite for E002 Anti-Tautology rule

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::diagnostic::RuleCode;
use agent_test_guard_rules::AntiTautologyRule;

#[test]
fn test_rust_clean_assertions_pass_without_diagnostics() {
    // Given: valid, meaningful dynamic assertions in Rust
    let source = r#"
        #[test]
        fn test_valid_calculations() {
            let result = compute(10, 20);
            assert_eq!(result, 30);
            assert!(result > 0);
            assert_ne!(result, 0);
            debug_assert!(is_valid(&result));
        }
    "#;

    // When: analyzing source with AntiTautologyRule
    let diagnostics =
        AntiTautologyRule::check_source("tests/test_math.rs", source, GrammarLanguage::Rust)
            .expect("checking valid Rust assertions should succeed");

    // Then: no tautological assertion diagnostics are produced
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_rust_assert_true_literal_tautology_detected() {
    // Given: Rust test asserting constant true
    let source = r#"
        #[test]
        fn test_fictitious_progress() {
            assert!(true);
            assert!(true, "always passes");
            debug_assert!(true);
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/test_fake.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: all 3 assert!(true) tautologies are detected
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
        assert!(diag.message.contains("Tautological assertion"));
        assert!(diag.fix_hint.contains("dynamic values"));
    }
}

#[test]
fn test_rust_assert_eq_self_comparison_tautology_detected() {
    // Given: Rust test with assert_eq! comparing identical operands
    let source = r#"
        #[test]
        fn test_self_comparisons() {
            let x = 42;
            assert_eq!(x, x);
            assert_eq!(100, 100);
            assert_eq!("alpha", "alpha");
            debug_assert_eq!(status.code, status.code);
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/test_self.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: all 4 identical operand comparisons are flagged
    assert_eq!(diagnostics.len(), 4);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
        assert!(diag.message.contains("Tautological assertion"));
    }
}

#[test]
fn test_rust_assert_binary_tautology_detected() {
    // Given: Rust assert with binary tautology
    let source = r#"
        #[test]
        fn test_binary_tautologies() {
            let val = 5;
            assert!(val == val);
            assert!(1 == 1);
            assert!("hello" == "hello");
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/test_bin.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: all 3 binary tautologies are flagged
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}

#[test]
fn test_rust_assert_ne_distinct_literals_detected() {
    // Given: assert_ne! comparing two distinct constant literals
    let source = r#"
        #[test]
        fn test_inequality_tautology() {
            assert_ne!(1, 2);
            assert_ne!("foo", "bar");
            assert_ne!(true, false);
        }
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/test_ineq.rs", source, GrammarLanguage::Rust)
            .expect("checking Rust source should succeed");

    // Then: constant literal inequality tautologies are detected
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}

#[test]
fn test_ts_clean_assertions_pass_without_diagnostics() {
    // Given: valid, meaningful dynamic assertions in TypeScript
    let source = r#"
        describe("UserService", () => {
            it("authenticates and verifies profile", async () => {
                const user = await login("admin", "secret");
                expect(user.id).toBe(101);
                expect(user.roles).toContain("admin");
                expect(user.isActive).toBeTruthy();
                expect(user.deletedAt).toBeNull();
                assert.strictEqual(user.token.length, 64);
            });
        });
    "#;

    // When: analyzing source with AntiTautologyRule
    let diagnostics =
        AntiTautologyRule::check_source("tests/user.test.ts", source, GrammarLanguage::TypeScript)
            .expect("checking valid TypeScript assertions should succeed");

    // Then: 0 diagnostics produced
    assert!(
        diagnostics.is_empty(),
        "expected 0 diagnostics, got: {:?}",
        diagnostics
    );
}

#[test]
fn test_ts_expect_self_comparison_tautology_detected() {
    // Given: TypeScript tests comparing identical target and matcher arguments
    let source = r#"
        test("hollowed verification", () => {
            const result = calculate();
            expect(result).toBe(result);
            expect(result.status).toEqual(result.status);
            expect(42).toBe(42);
            expect("sample").toStrictEqual("sample");
            expect(true).toBe(true);
        });
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/hollow.test.ts", source, GrammarLanguage::TypeScript)
            .expect("checking TypeScript source should succeed");

    // Then: all 5 self-comparison tautologies are flagged
    assert_eq!(diagnostics.len(), 5);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}

#[test]
fn test_ts_expect_constant_matcher_tautologies_detected() {
    // Given: TypeScript assertions testing constant literals against built-in matchers
    let source = r#"
        test("tautological literal matchers", () => {
            expect(true).toBeTruthy();
            expect(false).toBeFalsy();
            expect(null).toBeNull();
            expect(undefined).toBeUndefined();
            expect(NaN).toBeNaN();
            expect(123).toBeDefined();
            expect("constant").toBeDefined();
        });
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/literal.test.ts", source, GrammarLanguage::TypeScript)
            .expect("checking TypeScript source should succeed");

    // Then: all 7 literal tautologies are flagged
    assert_eq!(diagnostics.len(), 7);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}

#[test]
fn test_ts_assert_tautologies_detected() {
    // Given: Node.js / Chai assert tautologies
    let source = r#"
        test("assert library tautologies", () => {
            const x = 10;
            assert(true);
            assert.ok(true);
            assert.equal(1, 1);
            assert.strictEqual(x, x);
            assert.deepEqual(x, x);
            assert(1 === 1);
            assert(x === x);
        });
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/assert.test.ts", source, GrammarLanguage::TypeScript)
            .expect("checking TypeScript source should succeed");

    // Then: all 7 assert tautologies are detected
    assert_eq!(diagnostics.len(), 7);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}

#[test]
fn test_ts_not_inversion_literal_tautologies_detected() {
    // Given: inverted literal assertions that can never fail
    let source = r#"
        test("inverted literal tautologies", () => {
            expect(true).not.toBe(false);
            expect(false).not.toBe(true);
            expect(1).not.toBe(2);
        });
    "#;

    // When: checking source
    let diagnostics =
        AntiTautologyRule::check_source("tests/not.test.ts", source, GrammarLanguage::TypeScript)
            .expect("checking TypeScript source should succeed");

    // Then: all 3 inverted tautologies are flagged
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E002);
    }
}
