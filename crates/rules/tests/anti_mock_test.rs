use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_rules::anti_mock::{AntiMockRule, TestTier};
use agent_test_guard_rules::diagnostic::RuleCode;

#[test]
fn test_tier1_allows_mocking() {
    let source = r#"
        import { vi } from 'vitest';
        vi.mock('./api', () => ({ fetchUser: () => ({ id: 1 }) }));
        test('unit test', () => { expect(1).toBe(1); });
    "#;

    let diags = AntiMockRule::new(TestTier::Tier1Unit)
        .evaluate_source("tests/unit/user.test.ts", source, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");

    assert!(diags.is_empty(), "Tier 1 must allow mocks");
}

#[test]
fn test_tier3_catches_ts_mock_calls() {
    let source = r#"
        import { vi } from 'vitest';
        vi.mock('./stripe/payment');
        test('e2e checkout flow', () => {
            expect(true).toBe(true);
        });
    "#;

    let diags = AntiMockRule::check_source("tests/e2e/checkout.test.ts", source, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].code, RuleCode::E004);
    assert!(diags[0].message.contains("vi.mock"));
}

#[test]
fn test_tier3_catches_nock_or_sinon_import() {
    let source = r#"
        import nock from 'nock';
        test('e2e network flow', () => {
            nock('https://api.domain.com').get('/status').reply(200);
        });
    "#;

    let diags = AntiMockRule::check_source("e2e/network.test.ts", source, GrammarLanguage::TypeScript)
        .expect("Analysis should succeed");

    assert!(!diags.is_empty());
    assert_eq!(diags[0].code, RuleCode::E004);
    assert!(diags[0].message.contains("nock"));
}

#[test]
fn test_tier3_catches_rust_automock_and_macro() {
    let source = r#"
        #[automock]
        trait PaymentGateway {
            fn charge(&self, amount: u64) -> bool;
        }

        #[test]
        fn test_e2e_payment() {
            mock! {
                MyService {}
            }
            assert!(true);
        }
    "#;

    let diags = AntiMockRule::check_source("tests/e2e/gateway.rs", source, GrammarLanguage::Rust)
        .expect("Analysis should succeed");

    assert_eq!(diags.len(), 2);
    assert_eq!(diags[0].code, RuleCode::E004);
    assert_eq!(diags[1].code, RuleCode::E004);
}