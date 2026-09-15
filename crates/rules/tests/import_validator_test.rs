use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::diagnostic::RuleCode;
use agent_test_guard_rules::ImportValidatorRule;

#[test]
fn test_ts_clean_product_import_passes() {
    let source = r#"
        import { describe, it, expect } from 'vitest';
        import { calculateTotal } from '../src/calculator';

        describe('calc', () => {
            it('works', () => {
                expect(calculateTotal(10, 2)).toBe(12);
            });
        });
    "#;

    let diags = ImportValidatorRule::check_source("tests/calc.test.ts", source, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");

    assert!(diags.is_empty(), "Valid product import must produce 0 diagnostics");
}

#[test]
fn test_ts_circular_or_test_import_detected() {
    let source = r#"
        import { it, expect } from 'vitest';
        import { helper } from './other.test';

        it('dummy', () => { expect(1).toBe(1); });
    "#;

    let diags = ImportValidatorRule::check_source("tests/auth.test.ts", source, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, RuleCode::E005);
    assert!(diags[0].message.contains("Prohibited test-on-test"));
}

#[test]
fn test_rust_self_import_detected() {
    let source = r#"
        use tests::user_service_test;

        #[test]
        fn test_something() {
            assert!(true);
        }
    "#;

    let diags = ImportValidatorRule::check_source("tests/user_service_test.rs", source, GrammarLanguage::Rust)
        .expect("Analysis should succeed");

    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, RuleCode::E005);
    assert!(diags[0].message.contains("Circular or self-import"));
}