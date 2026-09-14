use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::anti_skip::AntiSkipRule;
use agent_test_guard_rules::diagnostic::{RuleCode, Severity};

#[test]
fn test_diagnostic_json_serialization_card() {
    use agent_test_guard_rules::diagnostic::{Diagnostic, Span};

    let diagnostic = Diagnostic::new(
        RuleCode::E001,
        Severity::Error,
        "tests/auth.test.ts",
        Span::new(10, 1, 10, 25, 120, 145),
        "Test skipping detected via `it.skip`",
        "Remove `.skip` or resolve the underlying failure instead of skipping.",
    );

    let json_val = diagnostic.to_json_value();
    assert_eq!(json_val["code"], "E001");
    assert_eq!(json_val["severity"], "error");
    assert_eq!(json_val["file_path"], "tests/auth.test.ts");
    assert_eq!(json_val["span"]["start_line"], 10);
    assert_eq!(json_val["span"]["start_col"], 1);
    assert_eq!(json_val["span"]["end_line"], 10);
    assert_eq!(json_val["span"]["end_col"], 25);
    assert!(json_val["message"].as_str().unwrap().contains("it.skip"));
    assert!(json_val["fix_hint"].as_str().unwrap().contains("Remove `.skip`"));
}

#[test]
fn test_rust_anti_skip_when_test_is_clean() {
    let code = r#"
        #[test]
        fn test_addition() {
            assert_eq!(2 + 2, 4);
        }
    "#;
    let diagnostics = AntiSkipRule::check_source("src/lib.rs", code, GrammarLanguage::Rust)
        .expect("Analysis should succeed");
    assert!(diagnostics.is_empty(), "Expected 0 diagnostics for clean Rust test");
}

#[test]
fn test_rust_anti_skip_when_test_has_ignore_attribute() {
    let code = r#"
        #[test]
        #[ignore]
        fn test_flaky_service() {
            assert!(true);
        }
    "#;
    let diagnostics = AntiSkipRule::check_source("tests/integration.rs", code, GrammarLanguage::Rust)
        .expect("Analysis should succeed");
    assert_eq!(diagnostics.len(), 1);
    let d = &diagnostics[0];
    assert_eq!(d.code, RuleCode::E001);
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(d.file_path, "tests/integration.rs");
    assert!(d.message.contains("`#[ignore]`"));
    assert!(d.fix_hint.contains("Remove `#[ignore]`"));
}

#[test]
fn test_rust_anti_skip_when_ignore_attribute_reversed() {
    let code = r#"
        #[ignore = "known bug"]
        #[test]
        fn test_broken() {
            assert!(false);
        }
    "#;
    let diagnostics = AntiSkipRule::check_source("tests/unit.rs", code, GrammarLanguage::Rust)
        .expect("Analysis should succeed");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, RuleCode::E001);
}

#[test]
fn test_ts_anti_skip_when_test_is_clean() {
    let code = r#"
        describe("Calculator", () => {
            it("adds numbers", () => {
                expect(1 + 1).toBe(2);
            });
        });
    "#;
    let diagnostics = AntiSkipRule::check_source("test/calc.test.ts", code, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");
    assert!(diagnostics.is_empty());
}

#[test]
fn test_ts_anti_skip_when_test_skipped_via_methods_and_prefixes() {
    let code = r#"
        test.skip("skipped test method", () => {
            expect(true).toBe(true);
        });

        it.skip("skipped it method", () => {
            expect(1).toBe(1);
        });

        xtest("skipped via xtest prefix", () => {
            expect(2).toBe(2);
        });

        xit("skipped via xit prefix", () => {
            expect(3).toBe(3);
        });
    "#;
    let diagnostics = AntiSkipRule::check_source("test/suite.test.ts", code, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");
    assert_eq!(diagnostics.len(), 4);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E001);
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("skip") || diag.message.contains("xtest") || diag.message.contains("xit"));
    }
}

#[test]
fn test_ts_anti_skip_when_focused_via_only_and_prefixes() {
    let code = r#"
        test.only("focused test method", () => {
            expect(true).toBe(true);
        });

        it.only("focused it method", () => {
            expect(1).toBe(1);
        });

        fit("focused fit prefix", () => {
            expect(2).toBe(2);
        });
    "#;
    let diagnostics = AntiSkipRule::check_source("test/focus.test.ts", code, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");
    assert_eq!(diagnostics.len(), 3);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E001);
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.message.contains("focus") || diag.message.contains("only") || diag.message.contains("fit"));
    }
}

#[test]
fn test_ts_anti_skip_when_describe_skipped_or_focused() {
    let code = r#"
        describe.skip("skipped suite", () => {
            it("test 1", () => {});
        });

        describe.only("focused suite", () => {
            it("test 2", () => {});
        });

        xdescribe("xdescribe suite", () => {
            it("test 3", () => {});
        });

        fdescribe("fdescribe suite", () => {
            it("test 4", () => {});
        });
    "#;
    let diagnostics = AntiSkipRule::check_source("test/describe.test.ts", code, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");
    assert_eq!(diagnostics.len(), 4);
    for diag in &diagnostics {
        assert_eq!(diag.code, RuleCode::E001);
    }
}
