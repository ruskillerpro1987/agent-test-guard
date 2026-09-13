// allow: SIZE_OK — comprehensive AST integration test suite covering grammar, parser, queries, and fingerprinting

use agent_test_guard_ast::fingerprint::*;
use agent_test_guard_ast::grammar::*;
use agent_test_guard_ast::parser::*;
use agent_test_guard_ast::queries::*;

#[test]
fn test_language_detection_when_file_paths_have_standard_extensions() {
    // Given: file paths representing supported programming languages and unsupported formats
    let test_cases = [
        ("src/lib.rs", Some(GrammarLanguage::Rust)),
        ("tests/integration_test.rs", Some(GrammarLanguage::Rust)),
        ("src/index.ts", Some(GrammarLanguage::TypeScript)),
        ("tests/unit.test.ts", Some(GrammarLanguage::TypeScript)),
        ("src/App.tsx", Some(GrammarLanguage::Tsx)),
        ("components/Button.test.tsx", Some(GrammarLanguage::Tsx)),
        ("index.js", Some(GrammarLanguage::JavaScript)),
        ("tests/legacy.spec.js", Some(GrammarLanguage::JavaScript)),
        ("docs/README.md", None),
        ("Cargo.toml", None),
        ("Makefile", None),
        (".gitignore", None),
        ("no_extension", None),
        ("", None),
    ];

    // When: detecting grammar language from each file path
    for (path, expected) in test_cases {
        let detected = detect_language(path);

        // Then: detected language matches the expected GrammarLanguage variant or None
        assert_eq!(
            detected, expected,
            "failed language detection for path: '{path}'"
        );
    }
}

#[test]
fn test_tree_sitter_language_when_initialized_for_all_supported_grammars() {
    // Given: all variants of GrammarLanguage
    let supported_languages = [
        GrammarLanguage::Rust,
        GrammarLanguage::TypeScript,
        GrammarLanguage::Tsx,
        GrammarLanguage::JavaScript,
    ];

    // When: obtaining the tree-sitter language descriptor for each grammar
    for lang in supported_languages {
        let ts_lang = lang.tree_sitter_language();

        // Then: tree-sitter language contains valid grammar node definitions and non-zero node kinds
        assert!(
            ts_lang.node_kind_count() > 0,
            "expected non-zero node kinds for language: {lang:?}"
        );
    }
}

#[test]
fn test_ast_parse_succeeds_when_source_is_valid_rust_and_typescript() {
    // Given: syntactically valid Rust, TypeScript, and TSX source code snippets
    let valid_rust = r#"
        pub fn calculate_sum(values: &[i64]) -> i64 {
            values.iter().sum()
        }
    "#;
    let valid_typescript = r#"
        export interface UserConfig {
            timeoutMs: number;
            retryCount: number;
        }
        export function configure(config: UserConfig): boolean {
            return config.timeoutMs > 0;
        }
    "#;
    let valid_tsx = r#"
        import React from 'react';
        export const Badge = ({ label }: { label: string }) => {
            return <span className="badge">{label}</span>;
        };
    "#;

    // When: parsing valid source code for each respective language
    let rust_tree =
        parse(GrammarLanguage::Rust, valid_rust).expect("parsing valid Rust should succeed");
    let ts_tree = parse(GrammarLanguage::TypeScript, valid_typescript)
        .expect("parsing valid TypeScript should succeed");
    let tsx_tree =
        parse(GrammarLanguage::Tsx, valid_tsx).expect("parsing valid TSX should succeed");

    // Then: syntax trees are produced without syntax error nodes
    assert!(!rust_tree.root_node().has_error());
    assert_eq!(rust_tree.root_node().kind(), "source_file");

    assert!(!ts_tree.root_node().has_error());
    assert_eq!(ts_tree.root_node().kind(), "program");

    assert!(!tsx_tree.root_node().has_error());
    assert_eq!(tsx_tree.root_node().kind(), "program");
}

#[test]
fn test_fault_tolerant_parse_when_syntax_contains_syntax_errors() {
    // Given: deliberately broken code snippets with unclosed blocks and missing operands
    let broken_rust = "pub fn incomplete_func( { let x = ;";
    let broken_ts = "function malformed(a, { return a + ;";

    // When: parsing broken source code under fault-tolerant parser
    let rust_tree = parse(GrammarLanguage::Rust, broken_rust)
        .expect("fault-tolerant parser must produce a tree without crashing");
    let ts_tree = parse(GrammarLanguage::TypeScript, broken_ts)
        .expect("fault-tolerant parser must produce a tree without crashing");

    // Then: parser produces valid trees containing ERROR nodes indicating syntax errors
    assert!(
        rust_tree.root_node().has_error(),
        "expected Rust AST to contain ERROR nodes for broken syntax"
    );
    assert!(
        ts_tree.root_node().has_error(),
        "expected TypeScript AST to contain ERROR nodes for broken syntax"
    );
}

#[test]
fn test_rust_discovery_when_standard_test_function_contains_assertions() {
    // Given: valid Rust module with a standard #[test] containing diverse assertion macros
    let source = r#"
        #[test]
        fn test_arithmetic_operations() {
            let a = 15;
            let b = 30;
            assert!(a < b);
            assert_eq!(a + 15, b);
            assert_ne!(a, b);
            debug_assert!(b > 0);
        }
    "#;
    let tree = parse(GrammarLanguage::Rust, source).expect("parsing Rust source should succeed");

    // When: discovering test declarations via AST query
    let tests = discover_tests(&tree, source, GrammarLanguage::Rust)
        .expect("test discovery query should succeed");

    // Then: discovers exactly one test with correct metadata and count of 4 assertions
    assert_eq!(tests.len(), 1);
    let test = &tests[0];
    assert_eq!(test.name, "test_arithmetic_operations");
    assert_eq!(test.assertion_count, 4);
    assert!(!test.is_skipped);
    assert!(!test.is_focused);
}

#[test]
fn test_rust_discovery_when_async_tokio_test_present() {
    // Given: Rust source code containing an asynchronous test annotated with #[tokio::test]
    let source = r#"
        #[tokio::test]
        async fn test_async_fetch_payload() {
            let response = mock_client().get("/status").await;
            assert_eq!(response.status_code(), 200);
        }
    "#;
    let tree = parse(GrammarLanguage::Rust, source).expect("parsing Rust source should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::Rust)
        .expect("test discovery query should succeed");

    // Then: discovers the async test function with 1 assertion
    assert_eq!(tests.len(), 1);
    let test = &tests[0];
    assert_eq!(test.name, "test_async_fetch_payload");
    assert_eq!(test.assertion_count, 1);
    assert!(!test.is_skipped);
    assert!(!test.is_focused);
}

#[test]
fn test_rust_discovery_when_test_annotated_with_ignore() {
    // Given: Rust source code with tests marked #[ignore] before and after #[test]
    let source = r#"
        #[test]
        #[ignore]
        fn test_slow_integration_run() {
            assert!(execute_heavy_job());
        }

        #[ignore]
        #[test]
        fn test_temporarily_disabled_feature() {
            assert_eq!(cached_count(), 0);
        }
    "#;
    let tree = parse(GrammarLanguage::Rust, source).expect("parsing Rust source should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::Rust)
        .expect("test discovery query should succeed");

    // Then: both tests are discovered and both are marked as skipped
    assert_eq!(tests.len(), 2);
    let test1 = &tests[0];
    let test2 = &tests[1];

    assert_eq!(test1.name, "test_slow_integration_run");
    assert!(test1.is_skipped);
    assert_eq!(test1.assertion_count, 1);

    assert_eq!(test2.name, "test_temporarily_disabled_feature");
    assert!(test2.is_skipped);
    assert_eq!(test2.assertion_count, 1);
}

#[test]
fn test_rust_discovery_when_non_test_functions_and_multiple_assertions_present() {
    // Given: Rust source with regular public and private helper functions alongside a test
    let source = r#"
        pub fn helper_calculate(base: i32, multiplier: i32) -> i32 {
            assert!(base > 0);
            base * multiplier
        }

        fn internal_validate(data: &str) {
            assert_ne!(data.len(), 0);
        }

        #[test]
        fn test_verified_feature() {
            let result = helper_calculate(5, 2);
            assert_eq!(result, 10);
            assert!(result > 0);
        }
    "#;
    let tree = parse(GrammarLanguage::Rust, source).expect("parsing Rust source should succeed");

    // When: running test discovery query
    let tests = discover_tests(&tree, source, GrammarLanguage::Rust)
        .expect("test discovery query should succeed");

    // Then: only the function annotated with #[test] is returned; helper functions are ignored
    assert_eq!(tests.len(), 1);
    let test = &tests[0];
    assert_eq!(test.name, "test_verified_feature");
    assert_eq!(test.assertion_count, 2);
}

#[test]
fn test_ts_discovery_when_standard_test_and_it_functions_present() {
    // Given: TypeScript test file declaring tests via test() and it() call expressions
    let source = r#"
        test("adds two numbers correctly", () => {
            const sum = 10 + 20;
            expect(sum).toBe(30);
        });

        it("subtracts numbers accurately", () => {
            const diff = 50 - 25;
            expect(diff).toEqual(25);
        });
    "#;
    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing TypeScript should succeed");

    // When: discovering tests in TypeScript AST
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery query should succeed");

    // Then: both test() and it() declarations are discovered with exact names and 1 assertion each
    assert_eq!(tests.len(), 2);
    assert_eq!(tests[0].name, "adds two numbers correctly");
    assert_eq!(tests[0].assertion_count, 1);
    assert!(!tests[0].is_skipped);
    assert!(!tests[0].is_focused);

    assert_eq!(tests[1].name, "subtracts numbers accurately");
    assert_eq!(tests[1].assertion_count, 1);
    assert!(!tests[1].is_skipped);
    assert!(!tests[1].is_focused);
}

#[test]
fn test_ts_discovery_when_tests_skipped_via_skip_xtest_and_xit() {
    // Given: TypeScript tests skipped using test.skip(), it.skip(), xtest(), and xit()
    let source = r#"
        test.skip("skipped via member expression test.skip", () => {
            expect(true).toBe(true);
        });

        it.skip("skipped via member expression it.skip", () => {
            expect(false).toBe(false);
        });

        xtest("skipped via xtest function alias", () => {
            expect(1).toBe(1);
        });

        xit("skipped via xit function alias", () => {
            expect(2).toBe(2);
        });
    "#;
    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing TypeScript should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery query should succeed");

    // Then: all 4 skipped variations are discovered and marked with is_skipped = true
    assert_eq!(tests.len(), 4);
    assert_eq!(tests[0].name, "skipped via member expression test.skip");
    assert!(tests[0].is_skipped);

    assert_eq!(tests[1].name, "skipped via member expression it.skip");
    assert!(tests[1].is_skipped);

    assert_eq!(tests[2].name, "skipped via xtest function alias");
    assert!(tests[2].is_skipped);

    assert_eq!(tests[3].name, "skipped via xit function alias");
    assert!(tests[3].is_skipped);
}

#[test]
fn test_ts_discovery_when_tests_focused_via_only_and_fit() {
    // Given: TypeScript tests focused using test.only(), it.only(), and fit()
    let source = r#"
        test.only("focused test via test.only", () => {
            expect("sample").toHaveLength(6);
        });

        it.only("focused it via it.only", () => {
            expect([1, 2]).toContain(2);
        });

        fit("focused it via fit alias", () => {
            expect(true).toBeTruthy();
        });
    "#;
    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing TypeScript should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery query should succeed");

    // Then: all 3 focused variations are discovered and marked with is_focused = true
    assert_eq!(tests.len(), 3);
    assert_eq!(tests[0].name, "focused test via test.only");
    assert!(tests[0].is_focused);

    assert_eq!(tests[1].name, "focused it via it.only");
    assert!(tests[1].is_focused);

    assert_eq!(tests[2].name, "focused it via fit alias");
    assert!(tests[2].is_focused);
}

#[test]
fn test_ts_discovery_when_tests_nested_inside_describe_blocks() {
    // Given: TypeScript test suite with nested describe() blocks
    let source = r#"
        describe("AuthenticationService", () => {
            test("accepts valid bearer credentials", () => {
                expect(authResponse.status).toBe(200);
            });

            describe("TokenRenewal", () => {
                it("refreshes expired access tokens", () => {
                    expect(refreshed.token).toBeDefined();
                });
            });
        });
    "#;
    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing TypeScript should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery query should succeed");

    // Then: both nested test declarations are discovered with their respective suite paths
    assert_eq!(tests.len(), 2);

    let test1 = &tests[0];
    assert_eq!(test1.name, "accepts valid bearer credentials");
    assert_eq!(test1.suite_name.as_deref(), Some("AuthenticationService"));
    assert_eq!(test1.assertion_count, 1);

    let test2 = &tests[1];
    assert_eq!(test2.name, "refreshes expired access tokens");
    assert!(
        test2
            .suite_name
            .as_deref()
            .is_some_and(|s| s.contains("AuthenticationService") && s.contains("TokenRenewal")),
        "nested suite name should capture enclosing describe hierarchy: {:?}",
        test2.suite_name
    );
    assert_eq!(test2.assertion_count, 1);
}

#[test]
fn test_ts_discovery_when_diverse_assertion_frameworks_counted() {
    // Given: TypeScript test function using assertions from Jest/Vitest, Node assert, and AVA (t.*)
    let source = r#"
        test("comprehensive assertions across frameworks", (t) => {
            expect(val).toBe(42);
            assert(flag === true);
            assert.strictEqual(status, "active");
            assert.deepEqual(payload, expectedPayload);
            t.is(count, 10);
            t.true(ready);
        });
    "#;
    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing TypeScript should succeed");

    // When: discovering tests in the AST
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery query should succeed");

    // Then: discovers 1 test with all 6 assertions counted across assertion styles
    assert_eq!(tests.len(), 1);
    let test = &tests[0];
    assert_eq!(test.name, "comprehensive assertions across frameworks");
    assert_eq!(test.assertion_count, 6);
}

#[test]
fn test_structural_fingerprint_determinism_when_same_code_hashed() {
    // Given: identical Rust test code parsed in two separate parser runs
    let source = r#"
        fn test_sample_case() {
            let mut items = vec![1, 2, 3];
            items.push(4);
            assert_eq!(items.len(), 4);
        }
    "#;
    let tree1 =
        parse(GrammarLanguage::Rust, source).expect("first parse run of Rust should succeed");
    let tree2 =
        parse(GrammarLanguage::Rust, source).expect("second parse run of Rust should succeed");

    // When: computing structural fingerprint for both trees
    let fp1 = compute_structural_fingerprint(&tree1.root_node());
    let fp2 = compute_structural_fingerprint(&tree2.root_node());

    // Then: both produce an identical 64-character lowercase hex BLAKE3 digest
    assert_eq!(fp1.len(), 64);
    assert!(
        fp1.chars().all(|c| c.is_ascii_hexdigit()),
        "fingerprint must be a valid hex digest: '{fp1}'"
    );
    assert_eq!(fp1, fp2);
}

#[test]
fn test_structural_fingerprint_invariance_when_formatting_and_whitespace_altered() {
    // Given: compact single-line Rust test vs spaced and indented multiline variant
    let compact_source = "fn test_target(){let a=10;let b=20;assert_eq!(a+b,30);}";
    let spaced_source = r#"
        fn     test_target(   )   {

            let    a   =   10  ;

            let    b   =   20  ;

            assert_eq!(   a  +  b ,  30 )  ;

        }
    "#;
    let tree_compact =
        parse(GrammarLanguage::Rust, compact_source).expect("parsing compact code should succeed");
    let tree_spaced =
        parse(GrammarLanguage::Rust, spaced_source).expect("parsing spaced code should succeed");

    // When: computing structural fingerprints for both ASTs
    let fp_compact = compute_structural_fingerprint(&tree_compact.root_node());
    let fp_spaced = compute_structural_fingerprint(&tree_spaced.root_node());

    // Then: fingerprints are strictly identical regardless of formatting, spacing, or newlines
    assert_eq!(fp_compact, fp_spaced);
}

#[test]
fn test_structural_fingerprint_invariance_when_variable_identifiers_renamed() {
    // Given: Rust test function with variable 'a' vs identical structure with variable 'b'
    let rust_source_a = "fn test_case() { let a = 1; assert_eq!(a, 1); }";
    let rust_source_b = "fn test_case() { let b = 1; assert_eq!(b, 1); }";

    let tree_rust_a =
        parse(GrammarLanguage::Rust, rust_source_a).expect("parsing Rust code 'a' should succeed");
    let tree_rust_b =
        parse(GrammarLanguage::Rust, rust_source_b).expect("parsing Rust code 'b' should succeed");

    // Given: TypeScript test with identifier 'foo' vs identical structure with identifier 'bar'
    let ts_source_a = "test('case', () => { const foo = 1; expect(foo).toBe(1); });";
    let ts_source_b = "test('case', () => { const bar = 1; expect(bar).toBe(1); });";

    let tree_ts_a = parse(GrammarLanguage::TypeScript, ts_source_a)
        .expect("parsing TypeScript code 'foo' should succeed");
    let tree_ts_b = parse(GrammarLanguage::TypeScript, ts_source_b)
        .expect("parsing TypeScript code 'bar' should succeed");

    // When: computing structural fingerprints across renamed variants
    let fp_rust_a = compute_structural_fingerprint(&tree_rust_a.root_node());
    let fp_rust_b = compute_structural_fingerprint(&tree_rust_b.root_node());

    let fp_ts_a = compute_structural_fingerprint(&tree_ts_a.root_node());
    let fp_ts_b = compute_structural_fingerprint(&tree_ts_b.root_node());

    // Then: renaming variable identifiers preserves identical structural AST node kind fingerprints
    assert_eq!(
        fp_rust_a, fp_rust_b,
        "Rust variable rename must produce identical structural fingerprint"
    );
    assert_eq!(
        fp_ts_a, fp_ts_b,
        "TypeScript variable rename must produce identical structural fingerprint"
    );
}

#[test]
fn test_structural_fingerprint_sensitivity_when_test_body_mutated_or_hollowed() {
    // Given: original test body with assertion, body with assertion removed, and hollowed empty body
    let original_source = "fn test_hollowing_probe() { let a = 1; assert_eq!(a, 1); }";
    let stripped_source = "fn test_hollowing_probe() { let a = 1; }";
    let empty_source = "fn test_hollowing_probe() {}";

    let tree_orig = parse(GrammarLanguage::Rust, original_source)
        .expect("parsing original Rust code should succeed");
    let tree_stripped = parse(GrammarLanguage::Rust, stripped_source)
        .expect("parsing stripped Rust code should succeed");
    let tree_empty =
        parse(GrammarLanguage::Rust, empty_source).expect("parsing empty Rust code should succeed");

    // When: computing fingerprints for each altered version
    let fp_orig = compute_structural_fingerprint(&tree_orig.root_node());
    let fp_stripped = compute_structural_fingerprint(&tree_stripped.root_node());
    let fp_empty = compute_structural_fingerprint(&tree_empty.root_node());

    // Then: all three versions produce distinct structural fingerprints
    assert_ne!(
        fp_orig, fp_stripped,
        "removing assertions must change the structural fingerprint"
    );
    assert_ne!(
        fp_orig, fp_empty,
        "emptying test body must change the structural fingerprint"
    );
    assert_ne!(
        fp_stripped, fp_empty,
        "stripped body vs empty body must have distinct structural fingerprints"
    );
}
