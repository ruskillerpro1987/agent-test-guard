# PROJECT_STATE.md — Срез текущего состояния проекта Agent Test Guard

> **Единый источник истины (Single Source of Truth) о статусе реализации, активных работах и принятом тех.долге.**  
> Дата актуализации: Сентябрь 2026.  
> Основание: аудит исходного кода `crates/` и 89 пройденных тестов (Rust 2021 Workspace).

---

## 1. Описание проекта и технологический стек

**Agent Test Guard** (`agent-test-guard`) — сверхнизколатентный (<25ms) детерминированный pre-commit гейт и CLI для предотвращения манипуляций с тестами и specification gaming со стороны автономных ИИ-агентов (пропуск тестов, тавтологии, опустошение ассертов, нелегальный мокинг).

- **Язык разработки:** Rust edition 2021 (Workspace).
- **Криптография:** RFC 8785 JSON Canonicalization Scheme (JCS), BLAKE3 (хэширование и structural fingerprinting), Ed25519 (цифровая подпись и верификация манифеста).
- **Парсинг кода:** Tree-sitter (C-библиотеки TS, TSX, Rust через `cc-rs` offline build).
- **CLI:** `clap` (subcommands: `init`, `check`, `ratchet`, `protect`, `explain`).

---

## 2. Архитектурные инварианты (Non-Negotiable)

1. **Pre-Commit Latency Budget (<25ms):** Статическая проверка типичного коммита (<50 файлов) выполняется строго быстрее 25мс за счёт прямого обхода Tree-sitter CST и чтения Git-блобов из памяти без вызова внешних процессов (`node`, `python`, `cargo`).
2. **Anti-TOCTOU Git Reader:** Проверяются исключительно staged-блобы из `git index`, а не грязная рабочая директория (working tree), исключая подмену файлов во время коммита.
3. **Monotonic Ratchet & Anti-Deletion:** Количество валидных тестов не может уменьшаться агентом ($A \subseteq B \land |Current| \ge |Baseline|$). Уменьшение набора тестов возможно только при наличии флага `--allow-shrink` с валидной Ed25519-подписью доверенного ключа.
4. **Structural AST Fingerprinting:** Тела тестов защищены от выхолащивания и затирания проверок через BLAKE3-хэш последовательности типов узлов AST и обязательный Assertion Floor ($\ge 1$).
5. **No Silent Fallbacks (Fail Loudly):** Нераспознанные синтаксические конструкции или повреждённые сигнатуры вызывают отказ с явной диагностической карточкой (E001..E010).

---

## 3. Декомпозиция по подсистемам

### Подсистема 1: Криптографическое ядро и схема манифеста (`agent-test-guard-core`)
*Схема `.test-guard.json`, каноникализация JCS, хэширование BLAKE3, подпись Ed25519 и храповик (ratchet).*

#### Готово (подтверждено кодом и тестами — 44 теста):
- **RFC 8785 Canonical JSON:** Детерминированная сериализация с лексикографической сортировкой ключей и нормализацией чисел (`crates/core/src/canonical.rs`, 8 тестов в `crates/core/tests/canonical_test.rs`).
- **BLAKE3 & Ed25519 Crypto:** Вычисление хэшей, генерация ключей, подписание и верификация манифеста (`crates/core/src/crypto.rs`, 9 тестов в `crates/core/tests/crypto_test.rs`).
- **Схема `.test-guard.json`:** Структуры `BaselineManifest`, `TestFileRecord`, `TestRecord`, валидация целостности хэшей и путей (`crates/core/src/schema.rs`, 12 тестов в `crates/core/tests/schema_test.rs`).
- **Монотонный Ratchet Engine:** Защита от Sybil-атак удаления/подмены тестов, верификация монотонного роста базы тестов и авторизованный shrink по Ed25519 (`crates/core/src/ratchet.rs`, 15 тестов в `crates/core/tests/ratchet_test.rs`).

#### В процессе:
- Интеграция с Git index blob reader для сквозной проверки хешей без записи на диск.

#### Отложено (Технический долг):
- Локальное хранение секретного ключа Ed25519 в виде raw-байтов / env-переменной без интеграции с аппаратными HSM/KMS.

---

### Подсистема 2: AST-парсеры и структурный фингерпринтинг (`agent-test-guard-ast`)
*Синтаксический анализ тестов через Tree-sitter и вычисление инвариантных структурных отпечатков.*

#### Готово (подтверждено кодом и тестами — 17 тестов):
- **Tree-sitter Runtime:** Инициализация грамматик TypeScript, TSX, Rust с оффлайн-сборкой через `cc-rs` (`crates/ast/src/grammar.rs`, `crates/ast/src/parser.rs`).
- **Каталог S-expression запросов:** Поиск тестов в Rust (`#[test]`, `#[tokio::test]`, `#[rstest]`) и TS/JS (`test`, `it`, `describe`), а также ассертов (`expect()`, `assert!`, `assert_eq!`, `t.is()`, `chai`) (`crates/ast/src/queries.rs`).
- **BLAKE3 Structural Fingerprint:** Хэширование последовательности типов узлов AST, инвариантное к форматированию, переносам строк и переименованию локальных переменных (`crates/ast/src/fingerprint.rs`, 17 тестов в `crates/ast/tests/ast_test.rs`).

#### В процессе:
- Повторное использование кэша распарсенных деревьев между различными правилами в едином проходе.

#### Отложено (Технический долг):
- **Грамматики Python (`pytest`) и Go (`testing`):** Отложены до Фазы 3 для сохранения компактности первой версии и фокусировки на TypeScript и Rust.

---

### Подсистема 3: Движок правил защиты от читерства (`agent-test-guard-rules`)
*Анализ тестов на пропуски, тавтологии, удаление проверок и подмену моков.*

#### Готово (подтверждено кодом и тестами — 28 тестов):
- **Диагностическая модель:** Структура `Diagnostic` с кодами `E001`..`E010`, тяжестью (Severity), спанами и подсказками по исправлению, сериализация в JSON-карточки для ИИ-агентов (`crates/rules/src/diagnostic.rs`, `crates/rules/src/engine.rs`).
- **E001 Anti-Skip Rule:** Детекция `#[ignore]` в Rust, а также `test.skip`, `it.skip`, `describe.skip`, `xtest`, `xit`, `xdescribe`, `test.only`, `it.only`, `fit`, `fdescribe` в TS/JS (`crates/rules/src/anti_skip.rs`, 8 тестов в `crates/rules/tests/anti_skip_test.rs`).
- **E002 Anti-Tautology Rule:** Детекция заведомо истинных проверок (`assert!(true)`, `assert_eq!(a, a)`, `assert_ne!(1, 2)`, `expect(x).toBe(x)`, `expect(true).toBeTruthy()`, `assert.strictEqual(x, x)`) (`crates/rules/src/anti_tautology.rs`, 10 тестов в `crates/rules/tests/anti_tautology_test.rs`).
- **E003 Anti-Hollowing & Assertion Floor:** Детекция пустых тел тестов, тестов без утверждений и нарушения минимального порога ассертов (`crates/rules/src/anti_hollowing.rs`, 10 тестов в `crates/rules/tests/anti_hollowing_test.rs`).

#### В процессе:
- **E004 Anti-Mock & Tier Boundary:** 3-уровневый карантин моков (запрет заглушек сетевых вызовов и подмены доменной логики в Tier 3 E2E) (`crates/rules/src/anti_mock.rs`).
- **E005 Import Validator:** Проверка целостности импортов и изоляция фейковых модулей (`crates/rules/src/import_validator.rs`).

#### Отложено (Технический долг):
- **Семантический AST-дифф:** Анализ смысловой разницы изменённого теста до и после правки (пока опирается на structural hash и порог ассертов).

---

### Подсистема 4: CLI-интерфейс и Git Integration (`agent-test-guard-cli`)
*CLI-утилита `test-guard` для локального запуска и использования в git pre-commit.*

#### Готово:
- Каркас CLI на базе `clap` с командами `init`, `check`, `ratchet`, `protect`, `explain` (`crates/cli/src/main.rs`).

#### В процессе:
- Zero-copy Git index blob reader (`git2`) для инспекции staged-файлов в оперативной памяти без записи на диск.
- Двухканальный вывод (ANSI Biome-style для человека + JSON diagnostic card для агента).

#### Отложено (Технический долг):
- **Tier B Hermetic Introspection Runner:** Динамический запуск `vitest list` / `cargo test -- --list` отложен до Фазы 3 (в pre-commit работает только быстрый Tier A AST-анализ).
- **Кроссплатформенный npm-трамплин:** Пакетирование бинарников под win32/linux/darwin в npm-пакет отложено до Фазы 4.

---

## 4. Реестр принятого технического долга (Tech Debt Ledger)

| ID | Подсистема / Компонент | Описание принятого долга | Обоснование / Почему отложено | Условие погашения |
|:---|:---|:---|:---|:---|
| **TD-01** | AST / Grammars | Отсутствие встроенных грамматик Python и Go | Первоочередная цель — поддержка TypeScript и Rust; минимизация размера бинарника и времени компиляции на Фазе 1-2 | Старт Фазы 3 (Polyglot Expansion) |
| **TD-02** | Rules / Anti-Mock | Отсутствие эвристики глубокого анализа monkey-patching глобальных прототипов в runtime | Требует динамического анализатора или интеграции с песочницей; статически закрывается проверкой AST импортов | Реализация правила E004 и валидатора импортов |
| **TD-03** | CLI / Runners | Отсутствие Tier B глубокого динамического сбора списка тестов (`--deep`) | Pre-commit должен укладываться в бюджет <25мс; запуск интерпретаторов Node/Cargo превышает бюджет | Добавление режима `--deep` для CI-пайплайна в Фазе 3 |
| **TD-04** | Core / Security | Хранение локального Ed25519 ключа на диске/в env без защищённого хранилища ОС | Достаточно для Model A (Cooperative Specification Gamer); Model B (Hostile Actor) ограничивается удалённым CI | Переход к интеграции с системными кейчейнами или GitHub Actions secrets |

---

## 5. Ближайшие вехи (Next Milestones)

1. **Завершить Шаг 2.4 (E004 Anti-Mock & Tier Boundary):** Реализовать изоляцию моков в `crates/rules/src/anti_mock.rs` с TDD-тестами.
2. **Реализовать Zero-Copy Git Index Blob Reader:** Обеспечить чтение staged-блобов через `git2` в памяти (`check --staged`).
3. **Реализовать команды CLI `init` и `check`:** Связать правила с CLI и протестировать выполнение в пределах <25мс.
