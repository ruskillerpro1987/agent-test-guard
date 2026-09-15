// crates/ast/tests/audit_r2_test.rs
//! Independent Forensic Audit Suite - R2: AST Fingerprinting & Fault Tolerance
//!
//! Verifies:
//! 1. Five adversarial examples (disguised tautology, identifier renames, test duplication,
//!    comments/empty blocks, JSX tags/props renames).
//! 2. Parser fault-tolerance, error detection, UTF-8 BOM handling, and corrupt input behavior.

use agent_test_guard_ast::fingerprint::compute_structural_fingerprint;
use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use agent_test_guard_ast::queries::discover_tests;

// =========================================================================
// 1. ADVERSARIAL TEST EXAMPLES
// =========================================================================

/// Adversarial 1: Rust tautological assert disguised via parens and intermediate variable.
///
/// Goal: Check whether fingerprint.rs catches structural differences vs semantic equivalence.
/// Expected:
/// - assert!(true) vs assert!(((true))) vs let flag = true; assert!(flag)
/// - Each has a distinct AST structure -> DIFFERENT BLAKE3 fingerprints.
/// - Structural fingerprint catches the syntax change, but FAILS to catch the semantic tautology.
#[test]
fn test_audit_adversarial_1_rust_tautology_disguises() {
    let code_assert_true = r#"
        #[test]
        fn test_simple() {
            assert!(true);
        }
    "#;

    let code_nested_parens = r#"
        #[test]
        fn test_simple() {
            assert!(((true)));
        }
    "#;

    let code_temp_var = r#"
        #[test]
        fn test_simple() {
            let flag = true;
            assert!(flag);
        }
    "#;

    let tree1 = parse(GrammarLanguage::Rust, code_assert_true).expect("parse tree1");
    let tree2 = parse(GrammarLanguage::Rust, code_nested_parens).expect("parse tree2");
    let tree3 = parse(GrammarLanguage::Rust, code_temp_var).expect("parse tree3");

    let fp1 = compute_structural_fingerprint(&tree1.root_node());
    let fp2 = compute_structural_fingerprint(&tree2.root_node());
    let fp3 = compute_structural_fingerprint(&tree3.root_node());

    // Structural fingerprints differ because Tree-sitter AST nodes differ:
    // fp1: token_tree -> boolean_literal
    // fp2: token_tree -> token_tree -> token_tree -> boolean_literal
    // fp3: let_declaration, token_tree -> identifier
    assert_ne!(
        fp1, fp2,
        "Parenthesized assert has different AST node nesting, producing a different fingerprint"
    );
    assert_ne!(
        fp1, fp3,
        "Intermediate variable introduces let_declaration, producing a different fingerprint"
    );
    assert_ne!(
        fp2, fp3,
        "Parenthesized vs intermediate variable have different fingerprints"
    );

    // Audit finding: fingerprint.rs does NOT detect semantic tautology.
    // It only hashes syntactic node kinds. All three return valid 64-char hex strings.
    assert_eq!(fp1.len(), 64);
    assert_eq!(fp2.len(), 64);
    assert_eq!(fp3.len(), 64);
}

/// Adversarial 2: TypeScript test with renamed local variables.
///
/// Goal: Verify that renaming local variables yields an IDENTICAL BLAKE3 structural fingerprint.
#[test]
fn test_audit_adversarial_2_ts_variable_renaming() {
    let code_a = r#"
        test("validation", () => {
            const userId = "usr_123";
            const isValid = userId.length > 0;
            expect(isValid).toBe(true);
        });
    "#;

    let code_b = r#"
        test("validation", () => {
            const x = "usr_123";
            const y = x.length > 0;
            expect(y).toBe(true);
        });
    "#;

    let tree_a = parse(GrammarLanguage::TypeScript, code_a).expect("parse ts code_a");
    let tree_b = parse(GrammarLanguage::TypeScript, code_b).expect("parse ts code_b");

    let fp_a = compute_structural_fingerprint(&tree_a.root_node());
    let fp_b = compute_structural_fingerprint(&tree_b.root_node());

    // Since variable names are stored in identifier nodes and kind() is "identifier",
    // the structural fingerprint is strictly invariant to variable renaming.
    assert_eq!(
        fp_a, fp_b,
        "Renaming local variables MUST produce identical structural fingerprints"
    );
}

/// Adversarial 3: Test duplication under a different test name (TS and Rust).
///
/// Goal: Verify whether duplicating a test under a new name produces an identical fingerprint.
/// Finding: Since string literal contents and function identifiers both share node kinds
/// ("string"/"string_fragment" and "identifier"), duplicated tests have IDENTICAL fingerprints!
#[test]
fn test_audit_adversarial_3_test_duplication_under_new_name() {
    // TypeScript test duplication
    let code_ts_original = r#"
        test("auth_token_validation", () => {
            const token = "abc";
            expect(token).toBeDefined();
        });
    "#;

    let code_ts_duplicated = r#"
        test("completely_different_test_name_to_game_ratchet", () => {
            const token = "abc";
            expect(token).toBeDefined();
        });
    "#;

    let tree_ts_orig = parse(GrammarLanguage::TypeScript, code_ts_original).unwrap();
    let tree_ts_dup = parse(GrammarLanguage::TypeScript, code_ts_duplicated).unwrap();

    let fp_ts_orig = compute_structural_fingerprint(&tree_ts_orig.root_node());
    let fp_ts_dup = compute_structural_fingerprint(&tree_ts_dup.root_node());

    assert_eq!(
        fp_ts_orig, fp_ts_dup,
        "Duplicated TS test under a different name produces an IDENTICAL fingerprint!"
    );

    // Rust test duplication
    let code_rust_original = r#"
        #[test]
        fn test_original_logic() {
            let val = 42;
            assert_eq!(val, 42);
        }
    "#;

    let code_rust_duplicated = r#"
        #[test]
        fn test_cloned_logic_new_name() {
            let val = 42;
            assert_eq!(val, 42);
        }
    "#;

    let tree_rust_orig = parse(GrammarLanguage::Rust, code_rust_original).unwrap();
    let tree_rust_dup = parse(GrammarLanguage::Rust, code_rust_duplicated).unwrap();

    let fp_rust_orig = compute_structural_fingerprint(&tree_rust_orig.root_node());
    let fp_rust_dup = compute_structural_fingerprint(&tree_rust_dup.root_node());

    assert_eq!(
        fp_rust_orig, fp_rust_dup,
        "Duplicated Rust test under a different function name produces an IDENTICAL fingerprint!"
    );
}

/// Adversarial 4: Rust test with added comments and dummy empty blocks.
///
/// Goal: Check whether non-semantic comments or empty blocks { { } } affect the fingerprint.
/// Finding:
/// - Comments (line_comment, block_comment) ARE named CST nodes in Tree-sitter!
///   Therefore, adding/removing comments changes the fingerprint! (Defect/Limitation)
/// - Empty blocks { { } } introduce named `block` nodes, changing the fingerprint!
#[test]
fn test_audit_adversarial_4_rust_empty_blocks_and_comments() {
    let code_base = r#"
        #[test]
        fn test_case() {
            let a = 1;
            assert_eq!(a, 1);
        }
    "#;

    let code_comments = r#"
        #[test]
        fn test_case() {
            // Explanatory line comment
            /* Multi-line
               comment block */
            let a = 1;
            assert_eq!(a, 1);
        }
    "#;

    let code_nested_blocks = r#"
        #[test]
        fn test_case() {
            {
                {
                    let a = 1;
                    assert_eq!(a, 1);
                }
            }
        }
    "#;

    let tree_base = parse(GrammarLanguage::Rust, code_base).unwrap();
    let tree_comments = parse(GrammarLanguage::Rust, code_comments).unwrap();
    let tree_blocks = parse(GrammarLanguage::Rust, code_nested_blocks).unwrap();

    let fp_base = compute_structural_fingerprint(&tree_base.root_node());
    let fp_comments = compute_structural_fingerprint(&tree_comments.root_node());
    let fp_blocks = compute_structural_fingerprint(&tree_blocks.root_node());

    // DEFECT DISCOVERY: comments change the structural fingerprint!
    assert_ne!(
        fp_base, fp_comments,
        "Comments are treated as named CST nodes in Tree-sitter, altering the structural hash!"
    );

    // Dummy empty blocks change the structural fingerprint!
    assert_ne!(
        fp_base, fp_blocks,
        "Nested empty blocks alter the structural hash!"
    );
}

/// Adversarial 5: TSX component test with JSX tag and property renames.
///
/// Goal: Verify whether renaming JSX elements and props produces an identical fingerprint.
/// Finding: JSX tags and props are `identifier` and `property_identifier` nodes.
/// Renaming `<Button disabled={true}>` to `<Card active={true}>` produces an IDENTICAL fingerprint!
#[test]
fn test_audit_adversarial_5_tsx_tag_prop_renaming() {
    let code_btn = r#"
        test("renders button", () => {
            const el = <Button disabled={true} variant="primary">Submit</Button>;
            expect(el).toBeDefined();
        });
    "#;

    let code_card = r#"
        test("renders card", () => {
            const el = <Card active={true} layout="vertical">Submit</Card>;
            expect(el).toBeDefined();
        });
    "#;

    let tree_btn = parse(GrammarLanguage::Tsx, code_btn).unwrap();
    let tree_card = parse(GrammarLanguage::Tsx, code_card).unwrap();

    let fp_btn = compute_structural_fingerprint(&tree_btn.root_node());
    let fp_card = compute_structural_fingerprint(&tree_card.root_node());

    assert_eq!(
        fp_btn, fp_card,
        "TSX component tag and prop renames preserve identical structural fingerprints!"
    );
}

// =========================================================================
// 2. FAULT-TOLERANCE & CORRUPTION AUDIT
// =========================================================================

/// Audit: Syntax error handling (unclosed brace, truncated function).
///
/// Invariant 5: "No Silent Fallbacks (Fail Loudly): Нераспознанные синтаксические конструкции...
/// вызывают отказ с явной диагностической карточкой".
/// Finding:
/// - `parse()` returns Ok(Tree), NOT ParseError!
/// - `tree.root_node().has_error()` is true.
/// - `discover_tests()` does NOT check `has_error()` and returns Ok(Vec) without any error!
#[test]
fn test_audit_fault_tolerance_syntax_error_silent_pass() {
    // Broken code with unclosed function brace
    let broken_code = "#[test]\nfn test_broken() { let a = 1;";

    let parse_result = parse(GrammarLanguage::Rust, broken_code);
    assert!(
        parse_result.is_ok(),
        "Tree-sitter parser returns Ok(Tree) even for syntax errors due to error recovery"
    );

    let tree = parse_result.unwrap();
    assert!(
        tree.root_node().has_error(),
        "AST root node must report has_error() == true for broken syntax"
    );

    // CRITICAL DEFECT: discover_tests does not check for syntax errors!
    let discovery_result = discover_tests(&tree, broken_code, GrammarLanguage::Rust);
    assert!(
        discovery_result.is_ok(),
        "discover_tests does NOT return QueryError on broken syntax trees"
    );
    let tests = discovery_result.unwrap();
    // In this broken function, test is not discovered because function_item body is malformed
    assert_eq!(tests.len(), 0);

    // Now test a file with one valid test followed by broken syntax:
    let mixed_code = r#"
        #[test]
        fn test_good() {
            assert!(true);
        }

        fn broken_syntax( {
    "#;
    let tree_mixed = parse(GrammarLanguage::Rust, mixed_code).unwrap();
    assert!(tree_mixed.root_node().has_error());

    let tests_mixed = discover_tests(&tree_mixed, mixed_code, GrammarLanguage::Rust).unwrap();
    // DEFECT: test_good is silently returned despite fatal syntax error in file!
    assert_eq!(
        tests_mixed.len(), 1,
        "discover_tests silently discovers valid tests in a corrupted/broken file without failing!"
    );
}

/// Audit: UTF-8 Byte Order Mark (BOM: \u{feff}) handling in Rust and TS.
///
/// Finding:
/// - In Rust: Tree-sitter parses BOM as part of the source, root_node().has_error() is FALSE.
///   discover_tests finds #[test] correctly.
/// - In TypeScript: Tree-sitter parses BOM, root_node().has_error() is FALSE.
///   discover_tests finds test() correctly.
#[test]
fn test_audit_fault_tolerance_utf8_bom_support() {
    // Rust with UTF-8 BOM
    let rust_bom = "\u{feff}#[test]\nfn test_with_bom() {\n    assert!(true);\n}\n";
    let tree_rust = parse(GrammarLanguage::Rust, rust_bom).expect("parse Rust with BOM");
    assert!(
        !tree_rust.root_node().has_error(),
        "Rust parser must not report syntax error for UTF-8 BOM"
    );
    let rust_tests = discover_tests(&tree_rust, rust_bom, GrammarLanguage::Rust).unwrap();
    assert_eq!(
        rust_tests.len(), 1,
        "Rust test discovery must find test annotated after BOM"
    );
    assert_eq!(rust_tests[0].name, "test_with_bom");

    // TypeScript with UTF-8 BOM
    let ts_bom = "\u{feff}test('bom_test', () => { expect(1).toBe(1); });\n";
    let tree_ts = parse(GrammarLanguage::TypeScript, ts_bom).expect("parse TS with BOM");
    assert!(
        !tree_ts.root_node().has_error(),
        "TypeScript parser must not report syntax error for UTF-8 BOM"
    );
    let ts_tests = discover_tests(&tree_ts, ts_bom, GrammarLanguage::TypeScript).unwrap();
    assert_eq!(
        ts_tests.len(), 1,
        "TypeScript test discovery must find test annotated after BOM"
    );
    assert_eq!(ts_tests[0].name, "bom_test");
}

/// Audit: Handling of Unicode replacement character \u{fffd} (corrupted UTF-8 lossy decoding).
///
/// Finding:
/// - When corrupted bytes are decoded via lossy UTF-8 (e.g. invalid bytes -> \u{fffd}),
///   if \u{fffd} occurs inside an identifier or token position, Tree-sitter flags an ERROR node.
/// - However, `parse()` does not fail (returns Ok(Tree)).
#[test]
fn test_audit_fault_tolerance_replacement_character_corrupted_input() {
    // Corrupted identifier containing Unicode replacement character
    let corrupted_rust = "fn test_\u{fffd}() { assert!(true); }";
    let tree = parse(GrammarLanguage::Rust, corrupted_rust).unwrap();
    // \u{fffd} in Rust identifier is invalid syntax for Tree-sitter Rust grammar
    assert!(
        tree.root_node().has_error(),
        "Corrupted identifier with replacement char must trigger AST ERROR node"
    );

    // Corrupted inside string literal: valid syntax (string fragment)
    let string_rust = "fn test_valid() { let s = \"corrupted_\u{fffd}\"; assert_eq!(s, s); }";
    let tree_str = parse(GrammarLanguage::Rust, string_rust).unwrap();
    assert!(
        !tree_str.root_node().has_error(),
        "Replacement character inside string literal is parsed as valid text"
    );
}
