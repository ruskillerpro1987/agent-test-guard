# Отчет независимого аудита тестовой базы: False-Positive, моки и мутационный анализ

> **Дата проведения**: 16 сентября 2026  
> **Репозиторий**: `agent-test-guard` (Rust 2021 Workspace)  
> **Инспектор**: Antigravity (с прямым выполнением и верификацией в терминале сессии)  
> **Предпосылка**: Тесты дают ложноположительные (false-positive) результаты и маскируют реальные сбои.

---

## 1. Фактическое состояние сьюта на момент аудита

При прямом запуске всего воркспейса в текущей сессии (`cargo test --workspace --no-fail-fast`) установлено, что тестовый сьют **не является монолитно зелёным**: 4 тестовых таргета падают (суммарно 7 упавших тестов-маркеров дефектов):

```text
error: 4 targets failed:
    `-p agent-test-guard --test check_command_e2e_test`
    `-p agent-test-guard --test latency_test`
    `-p agent-test-guard-ast --test bugfix_td11_td12_test`
    `-p agent-test-guard-core --test audit_ratchet_content_integrity_test`
```

### Детализация упавших тестов:
1. `crates/cli/tests/check_command_e2e_test.rs`:
   - `bug_check_staged_must_detect_e001_violation_via_real_git_index` — **FAILED**.
   - Причина: `test-guard check --staged` возвращает exit code 0 на коммите с нарушением E001 (`#[ignore]`), так как в `main.rs` находится `println!`-заглушка.
2. `crates/cli/tests/latency_test.rs`:
   - `test_latency_50_staged_files` — **FAILED**.
   - Причина: последовательный обход 50 файлов в debug-профиле превышает бюджет 25мс (среднее время ~99.8мс).
3. `crates/ast/tests/bugfix_td11_td12_test.rs`:
   - Все 4 теста упали:
     - `bug_td11_fingerprint_must_differ_when_asserted_literal_value_changes` — **FAILED** (слепота хэша к замене 200 -> 500 в `assert_eq!`).
     - `bug_td12_describe_skip_member_expression_must_mark_nested_tests_as_skipped` — **FAILED**.
     - `bug_td12_xdescribe_suite_must_mark_nested_tests_as_skipped` — **FAILED**.
     - `bug_td12_fdescribe_suite_must_mark_nested_tests_as_focused` — **FAILED**.
4. `crates/core/tests/audit_ratchet_content_integrity_test.rs`:
   - `bug_verify_ratchet_must_not_silently_accept_unsigned_assertion_count_decrease` — **FAILED** (тихое уменьшение `assertion_count` при неизменном `test_id` принимается без Ed25519-подписи).

---

## 2. Пустые, неполные и инвертированные Assertions

### 2.1. 10 инвертированных тестов («дефект зафиксирован как норма»)
В сьюте обнаружено 10 тестов, которые на заведомо некорректном коде проверяют `assert!(diags.is_empty())`, закрепляя баги движка как нормативное поведение:

| Файл | Тест | Ложный Assert | Суть маскируемого дефекта |
|---|---|---|---|
| `crates/ast/tests/audit_r2_test.rs:285` | `test_audit_fault_tolerance_syntax_error_silent_pass` | `assert!(res.is_ok())` | Битое дерево с синтаксической ошибкой молча возвращает `Ok(tests)`, нарушая Fail Loudly |
| `crates/rules/tests/audit_r3_test.rs:54` | `test_audit_e001_2_cfg_attr_ignore` | `assert!(diags.is_empty())` | Пропуск `#[cfg_attr(test, ignore)]` в Rust |
| `crates/rules/tests/audit_r3_test.rs:79` | `test_audit_e001_3_dynamic_skip_via_variable` | `assert!(diags.is_empty())` | Пропуск динамического `describe[s]` в TS |
| `crates/rules/tests/audit_r3_test.rs:138` | `test_audit_e002_2_constant_computed_assert_eq` | `assert!(diags.is_empty())` | Пропуск арифметической тавтологии `assert_eq!(1+1, 2)` |
| `crates/rules/tests/audit_r3_test.rs:163` | `test_audit_e002_3_assert_tautology_or_complex_condition` | `assert!(diags.is_empty())` | Пропуск тавтологии в дизъюнкции `assert!(1==1 \|\| cond)` |
| `crates/rules/tests/audit_r3_test.rs:190` | `test_audit_e003_1_type_assertion_typeof` | `assert!(diags.is_empty())` | Пропуск поверхностной проверки типа `expect(typeof x).toBe('string')` |
| `crates/rules/tests/audit_r3_test.rs:214` | `test_audit_e003_2_unreachable_assertion_after_return` | `assert!(diags.is_empty())` | Учёт недостижимого ассерта после `return;` в assertion floor |
| `crates/rules/tests/audit_r3_test.rs:238` | `test_audit_e003_3_dead_conditional_branch` | `assert!(diags.is_empty())` | Учёт ассерта внутри мёртвой ветки `if false { ... }` |
| `crates/rules/tests/audit_r3_test.rs:263` | `test_audit_e003_3b_bare_dummy_expect` | `assert!(diags.is_empty())` | Учёт пустого вызова `expect()` без матчера |
| `crates/rules/tests/audit_r3_test.rs:283` | `test_audit_e003_2_ts_unreachable_assertion` | `assert!(diags.is_empty())` | Учёт ассерта после `return` в TypeScript |

### 2.2. Дизъюнкции в `matches!` как сокрытие неработающих веток
В `crates/core/tests/ratchet_test.rs`:
- Строки 153–158 (`test_verify_ratchet_fails_when_test_count_decreased_without_allow_shrink`):
  ```rust
  assert!(matches!(result, Err(RatchetError::TestCountDecreased { .. } | RatchetError::MissingTests(..))));
  ```
- Строки 437–440 (`test_advance_baseline_fails_when_unauthorized_shrink_attempted`):
  ```rust
  assert!(matches!(result, Err(RatchetError::MissingTests(..) | RatchetError::TestCountDecreased { .. })));
  ```
В обоих случаях срабатывает только `MissingTests`. Ветка `TestCountDecreased` никогда не возбуждается, но тест проходит за счёт `|`.

### 2.3. Подавление диагностик в бенчмарках
В `crates/cli/tests/latency_test.rs` (L51-L53):
```rust
let _ = AntiSkipRule::check_tree(path, &tree, content, *lang);
let _ = AntiTautologyRule::check_tree(path, &tree, content, *lang);
let _ = AntiHollowingRule::check_tree(path, &tree, content, *lang);
```
Результаты проверок отбрасываются оператором `let _ =`. Если правила сгенерируют ложные срабатывания (false positives) на 50 файлах бенчмарка, тест этого не заметит.

---

## 3. Моки, заглушки и структурный мёртвый код

### 3.1. Заглушки вместо реализации CLI
В `crates/cli/src/main.rs` команды CLI `init`, `check`, `ratchet`, `protect`, `explain` являются `println!`-заглушками.  
Тест `control_check_staged_must_pass_clean_repo` завершается успешно `ok` только потому, что бинарник завершается с кодом 0, не выполняя ни одной проверки.

### 3.2. Мёртвый код в `crates/core/src/ratchet.rs`
```rust
// crates/core/src/ratchet.rs:121-132
if !removed.is_empty() && !options.allow_shrink {
    return Err(RatchetError::MissingTests(
        removed.iter().map(|t| t.id.clone()).collect(),
    ));
}

if current.ratchet.total_tests < baseline.ratchet.total_tests && !options.allow_shrink {
    return Err(RatchetError::TestCountDecreased {
        baseline: baseline.ratchet.total_tests,
        current: current.ratchet.total_tests,
    });
}
```
Поскольку `BaselineManifest::validate` требует, чтобы `total_tests` равнялся сумме тестов во всех файлах, любое уменьшение общего количества тестов означает, что как минимум один тест из `baseline` отсутствует в `current`.  
Следовательно, `removed` **всегда не пуст**! Ветка на строке 127 физически недостижима для валидных манифестов.

### 3.3. Мёртвое подвыражение в `crates/rules/src/anti_tautology.rs:88`
```rust
"assert_eq" | "debug_assert_eq" if args.len() >= 2 => {
    let (l, r) = (args[0].trim(), args[1].trim());
    l == r || (is_constant_literal(l) && is_constant_literal(r) && l == r)
}
```
Второе подвыражение `(is_constant_literal(l) && is_constant_literal(r) && l == r)` требует `l == r`. Но если `l == r`, то первое подвыражение уже вернуло `true`. Второе выражение является 100% мёртвым кодом.

---

## 4. Мутационный анализ: Эксперименты и выживаемость мутантов

Всего проверено 10 целевых мутаций. Все 10 мутаций **выжили** (тесты остались зелёными).

| ID | Целевой модуль и строка | Внесённая мутация | Результат прогона тестов | Статус мутанта |
|---|---|---|---|---|
| **M1** | `crates/core/src/ratchet.rs:127` | Отключение ветки: `if false && current.ratchet...` | `ratchet_test.rs`: 15/15 passed | **ВЫЖИЛ** (dead code, затенённый `!removed.is_empty()`) |
| **M2** | `crates/core/src/ratchet.rs:95` | Удаление проверки `\|\| old.assertion_count != test.assertion_count` | `ratchet_test.rs`: 15/15 passed | **ВЫЖИЛ** (все фикстуры имеют `assertion_count: 1`) |
| **M3** | `crates/rules/src/anti_hollowing.rs:153` | Удаление `"xtest" \| "xit" \| "fit"` | `anti_hollowing_test.rs`: 10/10 passed | **ВЫЖИЛ** (тестируются только `test` и `it`) |
| **M4** | `crates/rules/src/anti_hollowing.rs:196` | Удаление проверок bare `assert` и `assert.*`, `t.*` | `anti_hollowing_test.rs`: 10/10 passed | **ВЫЖИЛ** (в JS/TS проверяется только `expect`) |
| **M5** | `crates/rules/src/anti_hollowing.rs:118` | Удаление поддержки `#[rstest]` и `#[test_case]` | `anti_hollowing_test.rs`: 10/10 passed | **ВЫЖИЛ** (атрибуты `rstest` не проверяются) |
| **M6** | `crates/rules/src/anti_tautology.rs:84` | Удаление проверки тавтологии `!false` | `anti_tautology_test.rs`: 10/10 passed | **ВЫЖИЛ** (проверяется только `assert!(true)`) |
| **M7** | `crates/rules/src/anti_tautology.rs:90` | Удаление сопоставления с `debug_assert_ne` | `anti_tautology_test.rs`: 10/10 passed | **ВЫЖИЛ** (проверяется только `assert_ne!`) |
| **M8** | `crates/rules/src/anti_tautology.rs:125` | Удаление распознавания AVA `t.*` (`t.is`, `t.true`) | `anti_tautology_test.rs`: 10/10 passed | **ВЫЖИЛ** (нет тестов на AVA `t.*`) |
| **M9** | `crates/rules/src/anti_skip.rs:86` | Удаление проверки `::test` (например, `tokio::test`) | `anti_skip_test.rs`: 8/8 passed | **ВЫЖИЛ** (проверяется только стандартный `#[test]`) |
| **M10** | `crates/core/src/schema.rs:187-201` | Удаление валидации hex-длины ключа/подписи и алгоритма | `schema_test.rs`: 12/12 passed | **ВЫЖИЛ** (нет тестов на невалидный формат подписи) |

### Сырой протокол прямых запусков мутаций в текущей сессии:

#### Мутация M6 (`anti_tautology.rs:84` — удаление `!false`):
```text
   Compiling agent-test-guard-rules v0.1.0 (F:\AI\agent-test-guard\crates\rules)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.01s
     Running tests\anti_tautology_test.rs (target\debug\deps\anti_tautology_test-81c70f3d08fd1c9f.exe)

running 10 tests
test test_rust_assert_binary_tautology_detected ... ok
test test_rust_assert_true_literal_tautology_detected ... ok
test test_rust_clean_assertions_pass_without_diagnostics ... ok
test test_rust_assert_eq_self_comparison_tautology_detected ... ok
test test_ts_assert_tautologies_detected ... ok
test test_ts_clean_assertions_pass_without_diagnostics ... ok
test test_ts_expect_constant_matcher_tautologies_detected ... ok
test test_rust_assert_ne_distinct_literals_detected ... ok
test test_ts_expect_self_comparison_tautology_detected ... ok
test test_ts_not_inversion_literal_tautologies_detected ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

#### Мутация M1 (`ratchet.rs:127` — отключение `TestCountDecreased`):
```text
   Compiling agent-test-guard-core v0.1.0 (F:\AI\agent-test-guard\crates\core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.25s
     Running tests\ratchet_test.rs (target\debug\deps\ratchet_test-220ae4e84401f177.exe)

running 15 tests
test test_advance_baseline_fails_when_unauthorized_shrink_attempted ... ok
test test_verify_ratchet_fails_when_allow_shrink_true_without_signature ... ok
test test_verify_ratchet_fails_when_test_count_decreased_without_allow_shrink ... ok
test test_verify_ratchet_fails_when_test_deleted_without_allow_shrink ... ok
test test_verify_ratchet_fails_when_allow_shrink_true_with_untrusted_public_key ... ok
test test_verify_ratchet_when_identical_current_and_baseline ... ok
test test_verify_ratchet_fails_when_entire_file_deleted_without_allow_shrink ... ok
test test_verify_ratchet_fails_when_sybil_substitution_attack_detected ... ok
test test_verify_ratchet_tracks_modified_tests_when_ast_fingerprint_changes ... ok
test test_verify_ratchet_when_monotonic_addition_of_tests ... ok
test test_verify_ratchet_when_simultaneous_addition_and_fingerprint_modification ... ok
test test_verify_ratchet_when_tests_reordered_in_file ... ok
test test_verify_ratchet_fails_when_allow_shrink_true_with_invalid_signature ... ok
test test_verify_ratchet_succeeds_when_allow_shrink_true_with_valid_trusted_signature ... ok
test test_advance_baseline_succeeds_when_adding_tests_and_signs_if_key_provided ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## 5. Выводы и обязательные действия

1. **Реорганизация логики Ratchet**: перенести проверку `total_tests` до поэлементного поиска `removed` для обеспечения fail-fast и устранения dead code.
2. **Ликвидация CLI Mocking**: заменить `println!` в `crates/cli/src/main.rs` реальным пайплайном `git2 -> rules -> exit_code`.
3. **Устранение расхождения AST/Rules**: включить в `crates/ast/src/queries.rs` обработку `xdescribe`, `fdescribe` и корректный флаг `is_skipped` для `describe.skip`.
4. **Инвертирование тестов аудита**: по мере закрытия задач TD-05..TD-08 перевести все 10 инвертированных тестов на требование непустых диагностик (`!diags.is_empty()`).
5. **Пополнение тест-кейсов**: добавить проверки для мутаций M2–M10.
