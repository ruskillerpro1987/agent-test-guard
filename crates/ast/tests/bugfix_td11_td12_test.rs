// crates/ast/tests/bugfix_td11_td12_test.rs
//
// РЕГРЕССИОННЫЕ ТЕСТЫ на конкретные баги, найденные при независимом code
// review crates/ast/src/fingerprint.rs и crates/ast/src/queries.rs.
// Эти сценарии НЕ входили в ранее проведённый Forensic Audit R2/R3
// (audit_r2_test.rs / audit_r3_test.rs) и были им пропущены.
//
// ВАЖНО ДЛЯ АГЕНТА, ИСПРАВЛЯЮЩЕГО КОД:
// Эти тесты сейчас (до фикса) должны ПАДАТЬ. Это ожидаемо — они документируют
// реальные баги. Задача — поправить логику в crates/ast/src/fingerprint.rs
// и crates/ast/src/queries.rs так, чтобы тесты стали зелёными.
//
// НЕ МЕНЯЙТЕ САМИ ТЕСТЫ, чтобы они "прошли":
//   - не ослабляйте assert_ne!/assert! до assert!(true) или похожего;
//   - не удаляйте и не комментируйте отдельные проверки;
//   - не подменяйте ожидаемые значения под текущее (баговое) поведение
//     (например, не меняйте `assert!(tests[0].is_skipped)` на
//     `assert!(!tests[0].is_skipped)`, потому что "так сейчас работает").
// Если тест кажется вам неверным, избыточно строгим или основанным на
// неверном понимании требований — ОСТАНОВИТЕСЬ и спросите человека,
// а не правьте тест самостоятельно. Раздел "В процессе"/Tech Debt Ledger
// в PROJECT_STATE.md должен фиксировать реальное состояние кода, а не
// желаемое.

use agent_test_guard_ast::fingerprint::compute_structural_fingerprint;
use agent_test_guard_ast::grammar::GrammarLanguage;
use agent_test_guard_ast::parser::parse;
use agent_test_guard_ast::queries::discover_tests;

// =========================================================================
// БАГ TD-11: structural fingerprint слеп к литеральным значениям
// =========================================================================
//
// compute_structural_fingerprint (fingerprint.rs) хэширует ТОЛЬКО
// node.kind() (тип узла), но никогда текст/значение листовых узлов
// (integer_literal, string_fragment и т.д.). Из-за этого:
//
//     assert_eq!(response_status, 200);
//     assert_eq!(response_status, 500);
//
// дают ИДЕНТИЧНЫЙ BLAKE3-отпечаток — структура одинакова
// (macro_invocation -> ... -> integer_literal), а конкретное число 200/500 —
// это просто разный текст внутри узла одного и того же типа, который в хэш
// не попадает.
//
// Практическое следствие: anti-hollowing/ratchet, опирающиеся на сравнение
// structural fingerprint, НЕ ЗАМЕЧАЮТ, если кто-то тихо подменил ожидаемое
// значение в ассерте под фактическое (возможно, сломанное) поведение вместо
// того, чтобы исправить баг. Это ровно тот класс "подгонки тестов", для
// защиты от которого этот инструмент существует.

#[test]
fn bug_td11_fingerprint_must_differ_when_asserted_literal_value_changes() {
    let code_expected_200 = r#"
        #[test]
        fn test_status_code() {
            let response_status = 200;
            assert_eq!(response_status, 200);
        }
    "#;

    let code_expected_500 = r#"
        #[test]
        fn test_status_code() {
            let response_status = 200;
            assert_eq!(response_status, 500);
        }
    "#;

    let tree_200 =
        parse(GrammarLanguage::Rust, code_expected_200).expect("parsing valid Rust must succeed");
    let tree_500 =
        parse(GrammarLanguage::Rust, code_expected_500).expect("parsing valid Rust must succeed");

    let fp_200 = compute_structural_fingerprint(&tree_200.root_node());
    let fp_500 = compute_structural_fingerprint(&tree_500.root_node());

    // ОЖИДАЕМОЕ ПОВЕДЕНИЕ ПОСЛЕ ФИКСА: подмена ожидаемого значения в ассерте
    // обязана менять fingerprint — иначе ratchet считает такую подмену
    // "тем же тестом, ничего не изменилось".
    assert_ne!(
        fp_200, fp_500,
        "BUG TD-11: изменение ожидаемого значения в assert_eq! (200 -> 500) НЕ \
         изменило structural fingerprint. Guard слеп к подмене ожидаемого \
         значения ассерта при сохранении AST-структуры — классическая \
         'подгонка теста под фактическое поведение вместо исправления бага'."
    );
}

// =========================================================================
// БАГ TD-12: xdescribe / fdescribe / describe.skip не реализованы
// =========================================================================
//
// classify_js_call (queries.rs) не распознаёт идентификаторы `xdescribe` и
// `fdescribe` вообще (падают в ветку `_ => None`), и не проверяет свойство
// `.skip` в member_expression-вызовах вида `describe.skip(...)` (ветка
// матчит `("describe", _)` без проверки prop). При этом PROJECT_STATE.md и
// session_summary.md явно заявляют детекцию xdescribe/fdescribe/describe.skip
// как часть правила E001 Anti-Skip.
//
// Следствие: тесты внутри xdescribe/fdescribe/describe.skip блока сейчас
// обнаруживаются (discover_tests их не теряет), но как ОБЫЧНЫЕ, НЕ
// помеченные is_skipped/is_focused — то есть механизм E001 для
// suite-уровня пропуска сейчас не работает, несмотря на заявленную
// поддержку в документации.

#[test]
fn bug_td12_xdescribe_suite_must_mark_nested_tests_as_skipped() {
    let source = r#"
        xdescribe("LegacyPaymentFlow", () => {
            it("charges the card exactly once", () => {
                expect(chargeCount).toBe(1);
            });
        });
    "#;

    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing valid TypeScript must succeed");
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery must succeed");

    assert_eq!(
        tests.len(),
        1,
        "expected exactly one nested test to be discovered inside xdescribe"
    );
    assert!(
        tests[0].is_skipped,
        "BUG TD-12: тест внутри `xdescribe(...)` не помечен is_skipped=true. \
         xdescribe явно заявлен как поддерживаемый skip-паттерн в \
         PROJECT_STATE.md/session_summary.md, но classify_js_call в \
         queries.rs не содержит ветки для идентификатора 'xdescribe'."
    );
}

#[test]
fn bug_td12_fdescribe_suite_must_mark_nested_tests_as_focused() {
    let source = r#"
        fdescribe("OnlyThisSuiteRightNow", () => {
            it("runs in isolation", () => {
                expect(true).toBe(true);
            });
        });
    "#;

    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing valid TypeScript must succeed");
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery must succeed");

    assert_eq!(
        tests.len(),
        1,
        "expected exactly one nested test to be discovered inside fdescribe"
    );
    assert!(
        tests[0].is_focused,
        "BUG TD-12: тест внутри `fdescribe(...)` не помечен is_focused=true. \
         fdescribe заявлен в документации, но classify_js_call не содержит \
         ветки для идентификатора 'fdescribe'."
    );
}

#[test]
fn bug_td12_describe_skip_member_expression_must_mark_nested_tests_as_skipped() {
    let source = r#"
        describe.skip("DeprecatedModule", () => {
            test("still returns the old shape", () => {
                expect(shape).toEqual(oldShape);
            });
        });
    "#;

    let tree =
        parse(GrammarLanguage::TypeScript, source).expect("parsing valid TypeScript must succeed");
    let tests = discover_tests(&tree, source, GrammarLanguage::TypeScript)
        .expect("test discovery must succeed");

    assert_eq!(
        tests.len(),
        1,
        "expected exactly one nested test to be discovered inside describe.skip"
    );
    assert!(
        tests[0].is_skipped,
        "BUG TD-12: тест внутри `describe.skip(...)` не помечен is_skipped=true. \
         classify_js_call матчит member_expression ('describe', _) без \
         проверки свойства 'skip', теряя сигнал пропуска на уровне сьюта."
    );
}
