// crates/core/tests/audit_ratchet_content_integrity_test.rs
//
// РЕГРЕССИОННЫЙ ТЕСТ на архитектурный пробел, найденный при code review
// crates/core/src/ratchet.rs: verify_ratchet НЕ требует авторизации
// (allow_shrink + подпись) для теста, попавшего в категорию "modified" —
// гейты (MissingTests / TestCountDecreased / требование подписи)
// применяются ТОЛЬКО к полному удалению теста (removed) или к падению
// агрегатного total_tests. Тест, у которого при ТОМ ЖЕ id просто
// изменились fingerprint/assertion_count (например, агент вырезал часть
// ассертов, оставив хотя бы один — сам тест не удалён, общий счётчик
// тестов не упал), сейчас проходит verify_ratchet без единой ошибки и
// без подписи — попадает в RatchetDiff.modified как чисто
// информационная запись.
//
// Это прямо противоречит заявленному в PROJECT_STATE.md Инварианту 3
// (Monotonic Ratchet & Anti-Deletion / "Sybil Test Substitution"):
// защита сейчас реально работает только против удаления/переименования
// теста, но НЕ против выхолащивания содержимого теста на месте под тем
// же ID — то есть именно того способа подгонки тестов, ради защиты от
// которого этот инвариант описан. Это не зависит от TD-11 (слепота
// fingerprint к литеральным значениям) — даже когда TD-11 починят и
// fingerprint начнёт меняться от подмены значений, verify_ratchet
// всё равно пропустит такую модификацию молча, потому что "modified"
// в принципе ничего не гейтит.
//
// ВАЖНО ДЛЯ АГЕНТА, ИСПРАВЛЯЮЩЕГО КОД:
// Тест bug_... сейчас (до фикса) должен ПАДАТЬ. НЕ МЕНЯЙТЕ САМ ТЕСТ —
// не удаляйте assert, не ослабляйте его, не подгоняйте ожидание под
// текущее поведение ("modified и так просто логируется — это нормально").
// Контрольные тесты control_unchanged и control_growth ДОЛЖНЫ оставаться
// зелёными и после фикса — фикс обязан гейтить именно УМЕНЬШЕНИЕ
// assertion_count у существующего теста, а не любое изменение
// fingerprint вообще (иначе легитимное усиление теста — добавление
// новых ассертов — тоже будет требовать подписи, что избыточно и
// сломает нормальный рабочий процесс). Если непонятно, где провести
// границу — остановитесь и спросите человека, не решайте это правкой
// самого теста.

use agent_test_guard_core::ratchet::{verify_ratchet, RatchetOptions};
use agent_test_guard_core::schema::{
    BaselineConfig, BaselineManifest, RatchetState, TestFileBaseline, TestItemBaseline,
    TestLanguage, TestTier,
};

fn manifest_with_one_test(fingerprint: &str, assertion_count: usize) -> BaselineManifest {
    let test = TestItemBaseline {
        id: "tests/payment_test.rs::test_charges_card_once".to_string(),
        name: "test_charges_card_once".to_string(),
        fingerprint: fingerprint.to_string(),
        assertion_count,
    };
    let file = TestFileBaseline {
        path: "tests/payment_test.rs".to_string(),
        language: TestLanguage::Rust,
        tier: TestTier::Tier1Unit,
        blake3_hash: "a".repeat(64),
        tests: vec![test],
    };
    BaselineManifest {
        schema_version: 1,
        config: BaselineConfig {
            min_assertions_per_test: 1,
        },
        ratchet: RatchetState {
            total_tests: 1,
            files_count: 1,
        },
        files: vec![file],
        signature: None,
    }
}

#[test]
fn bug_verify_ratchet_must_not_silently_accept_unsigned_assertion_count_decrease() {
    // Baseline: тест с тремя ассертами.
    let baseline = manifest_with_one_test(&"1".repeat(64), 3);

    // Current: ТОТ ЖЕ id, но ассерты выхолощены до одного (2 удалено),
    // fingerprint соответственно другой. Тест не удалён (id тот же),
    // total_tests/files_count не изменились (1 и 1) — по нынешней логике
    // это НЕ считается "shrink" и не требует allow_shrink/подписи.
    let current = manifest_with_one_test(&"2".repeat(64), 1);

    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let result = verify_ratchet(&baseline, &current, &options);

    assert!(
        result.is_err(),
        "BUG: verify_ratchet вернул Ok(...) для теста, у которого при том же \
         id УМЕНЬШИЛОСЬ assertion_count (3 -> 1) и изменился fingerprint, \
         без allow_shrink и без подписи. Такая тихая модификация теста на \
         месте (без формального удаления/переименования) сейчас никак не \
         гейтится — modified-записи в RatchetDiff не требуют авторизации, \
         в отличие от added/removed. Got: {:?}",
        result
    );
}

#[test]
fn control_verify_ratchet_accepts_genuinely_unchanged_test() {
    let baseline = manifest_with_one_test(&"1".repeat(64), 3);
    let current = manifest_with_one_test(&"1".repeat(64), 3);

    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let result = verify_ratchet(&baseline, &current, &options);
    assert!(
        result.is_ok(),
        "genuinely unchanged manifest must pass verify_ratchet: {:?}",
        result
    );
}

#[test]
fn control_verify_ratchet_accepts_legitimate_assertion_growth_without_signature() {
    // Добавление НОВЫХ ассертов (усиление теста) — легитимное улучшение,
    // не должно требовать подписи/allow_shrink наравне с ослаблением.
    let baseline = manifest_with_one_test(&"1".repeat(64), 3);
    let current = manifest_with_one_test(&"3".repeat(64), 5);

    let options = RatchetOptions {
        allow_shrink: false,
        trusted_keys: vec![],
    };

    let result = verify_ratchet(&baseline, &current, &options);
    assert!(
        result.is_ok(),
        "increasing assertion_count for an existing test id must NOT require \
         signature/allow_shrink — legitimate test strengthening should not be \
         gated the same way as weakening. Got: {:?}",
        result
    );
}
