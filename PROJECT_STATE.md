# PROJECT_STATE.md — Срез текущего состояния проекта Agent Test Guard

> **Единый источник истины (Single Source of Truth) о статусе реализации, активных работах и принятом тех.долге.**  
> Дата актуализации: Сентябрь 2026.  
> Основание: аудит исходного кода `crates/` и 137 пройденных тестов (Rust 2021 Workspace, 100% зелёный статус).

---

## 1. Описание проекта и технологический стек

**Agent Test Guard** (`agent-test-guard`) — сверхнизколатентный (<25ms) детерминированный pre-commit гейт и CLI для предотвращения манипуляций с тестами и specification gaming со стороны автономных ИИ-агентов (пропуск тестов, тавтологии, опустошение ассертов, нелегальный мокинг).

- **Язык разработки:** Rust edition 2021 (Workspace).
- **Криптография:** RFC 8785 JSON Canonicalization Scheme (JCS), BLAKE3 (хэширование и structural fingerprinting), Ed25519 (цифровая подпись и верификация манифеста).
- **Парсинг кода:** Tree-sitter (C-библиотеки TS, TSX, Rust через `cc-rs` offline build).
- **CLI:** `clap` (subcommands: `init`, `check`, `ratchet`, `protect`, `explain`), многопоточный анализ `rayon`, Git blob reader `git2`.

---

## 2. Архитектурные инварианты и допущения

1. **Pre-Commit Latency Budget (<25ms, Rayon Parallel):** Статическая проверка типичного коммита (<50 файлов) выполняется строго быстрее 25мс за счёт параллельного обхода Tree-sitter CST через пул потоков `rayon` и чтения Git-блобов из памяти без вызова внешних процессов (`node`, `python`, `cargo`). *Архитектурное ограничение: бюджет <25ms гарантируется только в параллельном режиме (mean 8.74ms в release); однопоточный последовательный обход 50 AST упирается в FFI-оверхед Tree-sitter (~29ms).*
2. **Anti-TOCTOU Git Reader:** Проверяются исключительно staged-блобы из `git index`, а не грязная рабочая директория (working tree), исключая подмену файлов во время коммита.
3. **Monotonic Ratchet & Anti-Deletion:** Количество валидных тестов и ассертов внутри них не может уменьшаться агентом ($A \subseteq B \land |Current| \ge |Baseline|$). Уменьшение набора тестов или `assertion_count` возможно только при наличии флага `--allow-shrink` с валидной Ed25519-подписью доверенного ключа.
4. **Structural AST Fingerprinting & Context Literals:** Тела тестов защищены от выхолащивания и затирания проверок через BLAKE3-хэш последовательности типов узлов AST, обязательный Assertion Floor ($\ge 1$) и чувствительность к литералам внутри стандартных assertion-макросов/вызовов.
5. **No Silent Fallbacks (Fail Loudly):** Нераспознанные синтаксические конструкции или повреждённые сигнатуры вызывают отказ с явной диагностической карточкой (E001..E010).

---

## 3. Декомпозиция по подсистемам

### Подсистема 1: Криптографическое ядро и схема манифеста (`agent-test-guard-core`)
*Схема `.test-guard.json`, каноникализация JCS, хэширование BLAKE3, подпись Ed25519 и храповик (ratchet).*

#### Готово (подтверждено кодом и тестами — 64 теста):
- **RFC 8785 Canonical JSON:** Детерминированная сериализация с лексикографической сортировкой ключей и нормализацией чисел (`crates/core/src/canonical.rs`, 8 тестов в `crates/core/tests/canonical_test.rs`).
- **BLAKE3 & Ed25519 Crypto:** Вычисление хэшей, генерация ключей, подписание и верификация манифеста (`crates/core/src/crypto.rs`, 9 тестов в `crates/core/tests/crypto_test.rs`).
- **Схема `.test-guard.json`:** Структуры `BaselineManifest`, `TestFileRecord`, `TestRecord`, валидация целостности хэшей и путей (`crates/core/src/schema.rs`, 12 тестов в `crates/core/tests/schema_test.rs`).
- **Монотонный Ratchet Engine:** Защита от Sybil-атак удаления/подмены тестов, верификация монотонного роста базы тестов и авторизованный shrink по Ed25519 (`crates/core/src/ratchet.rs`, 15 тестов в `crates/core/tests/ratchet_test.rs`).
- **Forensic Audit R1 Test Suite:** Независимая верификация криптоядра: RFC 8032 Ed25519 (Test 1..3), официальные векторы BLAKE3 (len 0..3), RFC 8785 JCS (5 нетривиальных документов) и 8 сценариев adversarial-обхода храповика (`crates/core/tests/audit_r1_test.rs`, 17 тестов).
- **Audit Ratchet Integrity:** Полная защита от неавторизованного снижения числа ассертов внутри существующих тестов (TD-13 погашен; `crates/core/tests/audit_ratchet_content_integrity_test.rs`, 3 теста).

#### Отложено (Технический долг):
- **TD-15 / Dead Code in Ratchet:** Недостижимая ветка `TestCountDecreased` на строке 145 `ratchet.rs`, логически затенённая проверкой `!removed.is_empty()`.
- Локальное хранение секретного ключа Ed25519 в виде raw-байтов / env-переменной без интеграции с аппаратными HSM/KMS.

---

### Подсистема 2: AST-парсеры и структурный фингерпринтинг (`agent-test-guard-ast`)
*Синтаксический анализ тестов через Tree-sitter и вычисление инвариантных структурных отпечатков.*

#### Готово (подтверждено кодом и тестами — 29 тестов):
- **Tree-sitter Runtime:** Инициализация грамматик TypeScript, TSX, Rust с оффлайн-сборкой через `cc-rs` и потоко-безопасным `thread_local!` реестром исходников по root node ID (`crates/ast/src/grammar.rs`, `crates/ast/src/parser.rs`).
- **Каталог S-expression запросов:** Поиск тестов в Rust (`#[test]`, `#[tokio::test]`, `#[rstest]`) и TS/JS (`test`, `it`, `describe`), а также ассертов (`expect()`, `assert!`, `assert_eq!`, `t.is()`, `chai`) (`crates/ast/src/queries.rs`).
- **BLAKE3 Structural Fingerprint:** Хэширование структуры узлов AST, инвариантное к пробелам, форматированию, переименованию локальных переменных, TSX-пропсов и имен тестов (`crates/ast/src/fingerprint.rs`, 17 тестов в `crates/ast/tests/ast_test.rs`).
- **Контекстное хэширование литералов (TD-11 погашен):** Литералы констант внутри стандартных assertion-узлов (`macro_invocation` в Rust, вызовы `expect`, `assert`, `t.*` в JS/TS) подмешиваются в BLAKE3-хэш.  
  *Архитектурный компромисс:* Определение контекста ассерта базируется на синтаксическом анализе цепочки предков узла AST. Кастомные вспомогательные функции-ассерты (`custom_assert(...)`) без семантического dataflow-анализа пока не считаются контекстом утверждения.
- **Forensic Audit R2 Test Suite:** 8 adversarial-тестов структурного фингерпринтинга и fault-tolerance парсера (`crates/ast/tests/audit_r2_test.rs`, 8 тестов).
- **Suite-level Skips & Focus:** Детекция и наследование `xdescribe`, `fdescribe` и `describe.skip` во вложенные тесты через `SuiteContext` стек (TD-12 погашен; `crates/ast/tests/bugfix_td11_td12_test.rs`, 4 теста).

#### Отложено (Технический долг):
- **Грамматики Python (`pytest`) и Go (`testing`):** Отложены до Фазы 3 для сохранения компактности первой версии и фокусировки на TypeScript и Rust.
- **TD-08 / AST Syntax Error Silent Pass:** `parse()` и `discover_tests()` молча возвращают `Ok` с `tree.root_node().has_error() == true` вместо явного отказа (нарушение Инварианта 5).
- **TD-09 / Fingerprint Comment Sensitivity:** `node.is_named()` считает комментарии частью структуры, поэтому комментарии изменяют BLAKE3-хэш.

---

### Подсистема 3: Движок правил защиты от читерства (`agent-test-guard-rules`)
*Анализ тестов на пропуски, тавтологии, удаление проверок и подмену моков.*

#### Готово (подтверждено кодом и тестами — 39 тестов):
- **Диагностическая модель:** Структура `Diagnostic` с кодами `E001`..`E010`, тяжестью (Severity), спанами и подсказками по исправлению, сериализация в JSON-карточки для ИИ-агентов (`crates/rules/src/diagnostic.rs`, `crates/rules/src/engine.rs`).
- **E001 Anti-Skip Rule:** Детекция `#[ignore]` в Rust, а также `test.skip`, `it.skip`, `describe.skip`, `xtest`, `xit`, `xdescribe`, `test.only`, `it.only`, `fit`, `fdescribe` в TS/JS (`crates/rules/src/anti_skip.rs`, 8 тестов в `crates/rules/tests/anti_skip_test.rs`).
- **E002 Anti-Tautology Rule:** Детекция заведомо истинных проверок (`assert!(true)`, `assert_eq!(a, a)`, `assert_ne!(1, 2)`, `expect(x).toBe(x)`, `expect(true).toBeTruthy()`, `assert.strictEqual(x, x)`), очистка избыточных тавтологий (TD-15 погашен; `crates/rules/src/anti_tautology.rs`, 10 тестов в `crates/rules/tests/anti_tautology_test.rs`).
- **E003 Anti-Hollowing & Assertion Floor:** Детекция пустых тел тестов, тестов без утверждений и нарушения минимального порога ассертов (`crates/rules/src/anti_hollowing.rs`, 10 тестов в `crates/rules/tests/anti_hollowing_test.rs`).
- **Forensic Audit R3 Verification Suite:** 11 независимых adversarial-тестов попыток обхода правил E001-E003 с фиксацией пойманных и пропущенных паттернов (`crates/rules/tests/audit_r3_test.rs`, 11 тестов).

#### В процессе:
- **E004 Anti-Mock & Tier Boundary:** 3-уровневый карантин моков (запрет заглушек сетевых вызовов и подмены доменной логики в Tier 3 E2E) (`crates/rules/src/anti_mock.rs`).
- **E005 Import Validator:** Проверка целостности импортов и изоляция фейковых модулей (`crates/rules/src/import_validator.rs`).

#### Отложено (Технический долг):
- **Семантический AST-дифф:** Анализ смысловой разницы изменённого теста до и после правки (пока опирается на structural hash и порог ассертов).
- **TD-05 / E001 Bypass Gaps:** Обход `#[cfg_attr(..., ignore)]` и динамический skip `describe[s]`.
- **TD-06 / E002 Constant Folding & Disjunction:** Обход через вычисляемые выражения (`1 + 1 == 2`) и логические дизъюнкции (`1 == 1 || complex`).
- **TD-07 / E003 Reachability & Assertion Quality:** Обход через недостижимые ассерты (после `return`/`panic!`), мёртвые ветки (`if false`), bare `expect()` и ассертов на типы (`typeof`).
- **TD-16 / Inverted Audit Assertions:** 9 тестов в `audit_r3_test.rs` и 1 тест в `audit_r2_test.rs` ложно утверждают `assert!(diags.is_empty())` при наличии дефектов.

---

### Подсистема 4: CLI-интерфейс и Git Integration (`agent-test-guard-cli`)
*CLI-утилита `test-guard` для локального запуска и использования в git pre-commit.*

#### Готово (подтверждено кодом и тестами — 3 теста):
- Каркас CLI на базе `clap` с командами `init`, `check`, `ratchet`, `protect`, `explain` (`crates/cli/src/main.rs`).
- **Zero-Copy Git Index Blob Reader:** Чтение staged-блобов напрямую из индекса через `git2` без записи на диск (TD-14 погашен; `crates/cli/src/main.rs`).
- **Параллельная обработка staged-файлов:** Вызов правил E001/E002 через пул потоков `rayon` с возвратом exit code `1` при обнаружении нарушений (TD-14 погашен; 2 теста в `crates/cli/tests/check_command_e2e_test.rs`).
- **Forensic Audit R4 Latency Benchmark:** Замер времени проверки 50 staged-файлов в release-профиле (параллельный средний результат **8.74 ms**, с запасом укладывающийся в лимит <25 ms; TD-10 погашен; `crates/cli/tests/latency_test.rs`, 1 тест).

#### Отложено (Технический долг):
- **Tier B Hermetic Introspection Runner:** Динамический запуск `vitest list` / `cargo test -- --list` отложен до Фазы 3 (в pre-commit работает только быстрый Tier A AST-анализ).
- **Кроссплатформенный npm-трамплин:** Пакетирование бинарников под win32/linux/darwin в npm-пакет отложено до Фазы 4.
- Двухканальный вывод CLI (ANSI Biome-style для человека + JSON diagnostic card для агента).

---

## 4. Реестр принятого технического долга (Tech Debt Ledger)

| ID | Подсистема / Компонент | Описание принятого долга | Обоснование / Почему отложено | Статус / Условие погашения |
|:---|:---|:---|:---|:---|
| **TD-01** | AST / Grammars | Отсутствие встроенных грамматик Python и Go | Первоочередная цель — поддержка TypeScript и Rust; минимизация размера бинарника и времени компиляции на Фазе 1-2 | `ОТЛОЖЕНО` (Старт Фазы 3 Polyglot Expansion) |
| **TD-02** | Rules / Anti-Mock | Отсутствие эвристики глубокого анализа monkey-patching глобальных прототипов в runtime | Требует динамического анализатора или интеграции с песочницей; статически закрывается проверкой AST импортов | `В РАБОТЕ` (Реализация правила E004 и валидатора импортов) |
| **TD-03** | CLI / Runners | Отсутствие Tier B глубокого динамического сбора списка тестов (`--deep`) | Pre-commit должен укладываться в бюджет <25мс; запуск интерпретаторов Node/Cargo превышает бюджет | `ОТЛОЖЕНО` (Добавление режима `--deep` для CI в Фазе 3) |
| **TD-04** | Core / Security | Хранение локального Ed25519 ключа на диске/в env без защищённого хранилища ОС | Достаточно для Model A (Cooperative Specification Gamer); Model B (Hostile Actor) ограничивается удалённым CI | `ОТЛОЖЕНО` (Интеграция с системными кейчейнами или GitHub Actions secrets) |
| **TD-05** | Rules / Anti-Skip (E001) | Пропуск `#[cfg_attr(..., ignore)]` и динамического пропуска `describe[s]` | Требует разбора аргументов атрибута `cfg_attr` и dataflow-анализа локальных алиасов | `ОТЛОЖЕНО` (Фаза 2.5 Hardening правил E001/E002) |
| **TD-06** | Rules / Anti-Tautology (E002) | Пропуск арифметических тавтологий (`assert_eq!(1+1, 2)`) и дизъюнкций (`assert!(1==1 || cond)`) | Чистый CST-парсер без constant folding и рекурсивного разбора булевых выражений | `ОТЛОЖЕНО` (Внедрение AST-свёртки констант в Фазе 2.5) |
| **TD-07** | Rules / Anti-Hollowing (E003) | Пропуск недостижимых ассертов (после `return`/`panic!`), мёртвых веток (`if false`), bare `expect()` и ассертов на типы (`typeof`) | Отсутствие графа потока управления (CFG) и проверки семантической ценности ассерта | `ОТЛОЖЕНО` (Добавление CFG basic block reachability в Фазе 3) |
| **TD-08** | AST / Parser | Тихий пропуск синтаксических ошибок в `parse()` / `discover_tests()` (`has_error() == true`) | Tree-sitter восстанавливается после ошибок и строит дерево; парсер не выбрасывал ParseError | `ОТЛОЖЕНО` (Фаза 2.5 валидация `has_error()` перед обходом) |
| **TD-09** | AST / Fingerprint | Чувствительность структурного отпечатка к комментариям (`line_comment` как named node) | Tree-sitter относит комментарии к named CST nodes; изменение комментариев ломает ratchet | `ОТЛОЖЕНО` (Фаза 2.5 фильтрация комментариев) |
| **TD-10** | CLI / Engine | Превышение бюджета <25ms при однопоточном обходе 50 файлов (29.1ms) | **ПОГАШЕНО (с архитектурным допущением):** Бюджет выполняется исключительно за счёт параллелизации через Rayon (`par_iter` в CLI дает mean 8.74ms в release). Однопоточный путь физически ограничен оверхедом Tree-sitter FFI. | **ПОГАШЕНО** |
| **TD-11** | AST / Fingerprint | Слепота структурного фингерпринта к подмене константных литералов в ассертах | **ПОГАШЕНО (с архитектурным допущением):** Литералы внутри стандартных ассертов (`macro_invocation`, `expect`, `assert`, `t.*`) подмешиваются через `thread_local!` буфер. Пользовательские функции-хелперы требуют семантического dataflow-анализа (Фаза 3). | **ПОГАШЕНО** |
| **TD-12** | AST / Queries | Рассинхрон `queries.rs`: пропуск `xdescribe`, `fdescribe` и `describe.skip` | Устранено: `classify_js_call` распознает сьюты и прокидывает флаги пропуска/фокуса в дочерние тесты через стек `SuiteContext`. | **ПОГАШЕНО** |
| **TD-13** | Core / Ratchet | Храповик молча допускает неавторизованное уменьшение `assertion_count` | Устранено: уменьшение `assertion_count` требует `--allow-shrink` и валидную Ed25519-подпись доверенного ключа. | **ПОГАШЕНО** |
| **TD-14** | CLI / Engine | Вызовы CLI (`check --staged`) были реализованы как `println!`-заглушки | Устранено: staged-блобы читаются из git-индекса в память через `git2` и параллельно валидируются движком правил с возвратом exit code `1`. | **ПОГАШЕНО** |
| **TD-15** | Core / Rules | Мёртвое условие в `anti_tautology.rs` и dead code в `ratchet.rs` | Устранено частично: тавтологические подвыражения удалены из `anti_tautology.rs`. Dead code в `ratchet.rs` остается до рефакторинга. | **ПОГАШЕНО (ЧАСТИЧНО)** |
| **TD-16** | Rules / Tests | 10 тестов аудита (`audit_r3_test.rs`, `audit_r2_test.rs`) инвертированы | Созданы как фиксаторы дефектов (baseline snapshot), блокируют исправление TD-05..TD-08 | `ОТЛОЖЕНО` (Инвертирование по мере реализации правил) |

---

## 5. Ближайшие вехи (Next Milestones)

1. **Реализовать Шаг 2.4 (E004 Anti-Mock & Tier Boundary):** Создать модуль `crates/rules/src/anti_mock.rs` с TDD-тестами для блокировки нелегального мокинга.
2. **Реализовать E005 Import Validator:** Добавить проверку целостности импортов и изоляцию фейковых модулей (`crates/rules/src/import_validator.rs`).
3. **Харденинг правил E001–E003 (TD-05, TD-06, TD-07):** Закрыть обходы через вычисляемые выражения, недостижимый код и инвертировать тесты-ловушки TD-16.