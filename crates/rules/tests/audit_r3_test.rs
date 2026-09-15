//! Independent Forensic Audit R3: Adversarial bypass verification for E001, E002, E003
//!
//! Evaluates 9 adversarial bypass attempts against:
//! - E001 (anti-skip)
//! - E002 (anti-tautology)
//! - E003 (anti-hollowing / assertion floor)

use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::anti_hollowing::AntiHollowingRule;
use agent_test_guard_rules::anti_skip::AntiSkipRule;
use agent_test_guard_rules::anti_tautology::AntiTautologyRule;
use agent_test_guard_rules::diagnostic::RuleCode;

// =========================================================================
// E001 (anti-skip) Adversarial Bypass Attempts
// =========================================================================

/// E001.1: Nested describe.skip inside an active (unskipped) suite.
/// EXPECTATION: ПОЙМАНО (CAUGHT)
/// Mechanism: walk_descendants recursively visits all AST nodes,
/// detecting member expression `describe.skip` regardless of suite nesting depth.
#[test]
fn test_audit_e001_1_nested_describe_skip() {
    let source = r#"
        describe("active_outer_suite", () => {
            describe.skip("nested_skipped_suite", () => {
                it("inner_test_should_run", () => {
                    expect(1).toBe(1);
                });
            });
        });
    "#;

    let diags = AntiSkipRule::check_source("tests/suite.test.ts", source, GrammarLanguage::TypeScript)
        .expect("AST parsing should succeed");

    assert!(
        !diags.is_empty(),
        "E001.1 should catch nested describe.skip"
    );
    assert_eq!(diags[0].code, RuleCode::E001);
    assert!(
        diags[0].message.contains("describe.skip"),
        "Diagnostic message should reference describe.skip: {}",
        diags[0].message
    );
}

/// E001.2: Conditional ignore via `#[cfg_attr(test, ignore)]` or `#[cfg_attr(all(), ignore)]`.
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT)
/// Mechanism: `inspect_rust_function` only matches attribute `named_child(0) == "ignore"`.
/// It treats `cfg_attr` as the attribute name and fails to inspect the meta arguments inside `cfg_attr(...)`.
#[test]
fn test_audit_e001_2_cfg_attr_ignore() {
    let source = r#"
        #[test]
        #[cfg_attr(test, ignore)]
        fn conditionally_ignored_test() {
            assert_eq!(2 + 2, 4);
        }
    "#;

    let diags = AntiSkipRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    // Defect reproduction: The bypass passes undetected (diags is empty)
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E001 missed #[cfg_attr(test, ignore)]"
    );
}

/// E001.3: Dynamic `.skip` via variable index access (e.g. `const s = 'skip'; describe[s](...)`).
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT / CST LIMITATION)
/// Mechanism: Tree-sitter parses `describe[s]` as `subscript_expression`, which `check_js_call`
/// completely ignores (only matching `identifier` and `member_expression`).
/// Additionally, static CST lacks variable dataflow/alias resolution.
#[test]
fn test_audit_e001_3_dynamic_skip_via_variable() {
    let source = r#"
        const s = 'skip';
        describe[s]("dynamically_skipped_suite", () => {
            it("test_inside", () => {
                expect(1).toBe(1);
            });
        });
    "#;

    let diags = AntiSkipRule::check_source("tests/suite.test.ts", source, GrammarLanguage::TypeScript)
        .expect("AST parsing should succeed");

    // Defect reproduction: The dynamic skip via subscript access is ignored
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E001 missed dynamic describe[s] skip"
    );
}

// =========================================================================
// E002 (anti-tautology) Adversarial Bypass Attempts
// =========================================================================

/// E002.1: Intermediate variable self-comparison (`let x = compute(); assert_eq!(x, x)`).
/// EXPECTATION: ПОЙМАНО (CAUGHT)
/// Mechanism: `inspect_rust_macro` verifies `l == r` on argument string slices.
/// Since `"x"` == `"x"`, identical identifier comparison is caught without requiring type resolution.
#[test]
fn test_audit_e002_1_intermediate_variable_self_comparison() {
    let source = r#"
        #[test]
        fn test_self_comparison_tautology() {
            let x = compute_expensive_state();
            assert_eq!(x, x);
        }
    "#;

    let diags = AntiTautologyRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    assert_eq!(
        diags.len(),
        1,
        "E002.1 must catch assert_eq!(x, x) as tautology"
    );
    assert_eq!(diags[0].code, RuleCode::E002);
    assert!(
        diags[0].message.contains("assert_eq!(x, x)"),
        "Diagnostic should cite the tautological expression"
    );
}

/// E002.2: Constant computed from same value or arithmetic tautology `assert_eq!(1 + 1, 2)`.
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT / CONSTANT FOLDING LIMITATION)
/// Mechanism: `inspect_rust_macro` checks `l == r` ("1 + 1" != "2") and `is_constant_literal("1 + 1")`
/// which rejects expressions containing operators. Without an expression evaluator or constant folder,
/// computed constant tautologies are not recognized.
#[test]
fn test_audit_e002_2_constant_computed_assert_eq() {
    let source = r#"
        #[test]
        fn test_arithmetic_tautology() {
            assert_eq!(1 + 1, 2);
        }
    "#;

    let diags = AntiTautologyRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    // Defect reproduction: assert_eq!(1 + 1, 2) is not caught by E002
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E002 missed assert_eq!(1 + 1, 2)"
    );
}

/// E002.3: Tautology hidden inside boolean disjunction `assert!(1 == 1 || complex_condition)`.
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT)
/// Mechanism: `find_binary_op` stops at the first `==` and returns `l = "1"` and `r = "1 || complex_condition()"`.
/// `is_binary_tautology` checks `l == r` which fails, and does not parse logical operators `||` or `&&`
/// to evaluate sub-expressions independently.
#[test]
fn test_audit_e002_3_assert_tautology_or_complex_condition() {
    let source = r#"
        #[test]
        fn test_disjunction_tautology() {
            assert!(1 == 1 || complex_condition());
        }
    "#;

    let diags = AntiTautologyRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    // Defect reproduction: assert!(1 == 1 || complex_condition()) bypasses E002
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E002 missed assert!(1 == 1 || complex_condition())"
    );
}

// =========================================================================
// E003 (anti-hollowing / assertion floor) Adversarial Bypass Attempts
// =========================================================================

/// E003.1: Type assertion instead of value verification (`expect(typeof x).toBe('string')`).
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT / SEMANTIC DEPTH LIMITATION)
/// Mechanism: `is_js_assertion` only verifies the presence of an `expect()` or `assert()` call.
/// It does not inspect assertion depth or target expression (such as `typeof x`), and
/// `AntiTautologyRule` does not treat `typeof x == "string"` as a tautology.
#[test]
fn test_audit_e003_1_type_assertion_typeof() {
    let source = r#"
        test("specification gaming with type-only assertion", () => {
            const x = fetchUserData();
            expect(typeof x).toBe('string');
        });
    "#;

    let diags = AntiHollowingRule::check_source("tests/user.test.ts", source, GrammarLanguage::TypeScript)
        .expect("AST parsing should succeed");

    // Defect reproduction: AntiHollowingRule accepts expect(typeof x) as a valid assertion
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E003 satisfied by shallow typeof assertion"
    );
}

/// E003.2: Unreachable assertion placed after early `return` or `panic!()`.
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT / CFG LIMITATION)
/// Mechanism: `count_rust_assertions` walks AST descendants unconditionally without a
/// Control Flow Graph (CFG) or reachability analysis. Any assertion located after a terminal statement
/// (`return`, `panic!`) is still counted towards the assertion floor.
#[test]
fn test_audit_e003_2_unreachable_assertion_after_return() {
    let source = r#"
        #[test]
        fn test_early_return_unreachable_assertion() {
            return;
            assert_eq!(1, 2);
        }
    "#;

    let diags = AntiHollowingRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    // Defect reproduction: Unreachable assertion satisfies the floor (0 diagnostics)
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E003 satisfied by dead assertion after early return"
    );
}

/// E003.3: Assertion placed inside a dead conditional branch (`if false { ... }`).
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT / DEAD CODE LIMITATION)
/// Mechanism: `walk_descendants` recursively traverses the block inside `if false { ... }`
/// without evaluating boolean constant branch conditions, counting dead assertions.
#[test]
fn test_audit_e003_3_dead_conditional_branch() {
    let source = r#"
        #[test]
        fn test_dead_branch_assertion() {
            if false {
                assert_eq!(2 + 2, 5);
            }
        }
    "#;

    let diags = AntiHollowingRule::check_source("tests/test_mod.rs", source, GrammarLanguage::Rust)
        .expect("AST parsing should succeed");

    // Defect reproduction: Dead branch assertion satisfies the floor
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E003 satisfied by assertion in if false block"
    );
}

/// E003.3b: Empty or standalone dummy `expect()` in JS/TS.
/// EXPECTATION: ПРОПУЩЕНО (BYPASSED - DEFECT)
/// Mechanism: `is_js_assertion` checks only `func == "expect"`, counting a bare `expect();`
/// without matcher or arguments as a valid assertion satisfying the floor.
#[test]
fn test_audit_e003_3b_bare_dummy_expect() {
    let source = r#"
        test("dummy test with bare expect", () => {
            doSomething();
            expect();
        });
    "#;

    let diags = AntiHollowingRule::check_source("tests/test.js", source, GrammarLanguage::JavaScript)
        .expect("AST parsing should succeed");

    // Defect reproduction: bare expect() satisfies the assertion floor
    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E003 satisfied by bare dummy expect() call"
    );
}

/// E003.2b: TS unreachable assertion after return.
#[test]
fn test_audit_e003_2_ts_unreachable_assertion() {
    let source = r#"
        test("early return in ts", () => {
            return;
            expect(1).toBe(2);
        });
    "#;

    let diags = AntiHollowingRule::check_source("tests/test.ts", source, GrammarLanguage::TypeScript)
        .expect("AST parsing should succeed");

    assert!(
        diags.is_empty(),
        "DEFECT CONFIRMED: E003 satisfied by TS dead assertion after early return"
    );
}

