=== INDEPENDENT AUDIT REPORT: agent-test-guard (pre-Phase-2.4 baseline) ===

СВОДКА:
Криптографическое ядро (crates/core) и инварианты храповика полностью доказали математическую и криптографическую стойкость. В то же время статический CST-анализ тестов (crates/ast) и правила детекции читерства (crates/rules: E001–E003) содержат критические бреши (тихий пропуск синтаксических ошибок, обходы через cfg_attr, дизъюнкции, недостижимый код и вычисляемые константы).

R1 — Криптографическое ядро:
  Ed25519 vs RFC 8032: PASS (RFC 8032 §7.1: Test 1 len 0, Test 2 len 1 0x72, Test 3 len 2 0xaf82 — keygen, sign_hex, verify_hex, tampered payload/sig rejection)
  JCS vs RFC 8785: PASS (5 нетривиальных сценариев: UTF-16 surrogate code unit key sorting §3.2.3, wire-format byte parity §3.2.2/§3.2.4, deeply nested empty arrays/objects, IEEE 754 number normalization -0.0->0, minimal slash/control escaping)
  BLAKE3 vs official vectors: PASS (официальные векторы BLAKE3 team для len 0, 1, 2, 3: hash, keyed_hash "whats the Elvish word for friend", derive_key)
  Ratchet anti-shrink bypass attempt: ПОЙМАН (все 8 adversarial-попыток обмана пресечены: удаление тестов, переименование, подмена ID, инфляция счётчика, shrink без подписи, подпись чужим ключом, повреждённая подпись, replay-атака подписи)

R2 — AST fingerprinting:
  Пример 1 (Rust assert!(true) vs assert!(((true))) vs let flag = true; assert!(flag)): ПОЙМАНО структурно (разные хэши из-за вложенности token_tree/let_declaration), но ПРОПУЩЕНО семантически (тавтология маскируется синтаксисом)
  Пример 2 (TS переименование локальных идентификаторов): ПОЙМАНО (хэш инвариантен к именам переменных: 98eea0d4...)
  Пример 3 (TS/Rust дублирование теста под другим именем): ПОЙМАНО структурно (хэш 1:1 совпадает), но уязвимость для храповика (ratchet не запрещает дубликаты фингерпринтов разных test_id)
  Пример 4 (Rust добавление пустых блоков { { } } и комментариев //, /* */): ДЕФЕКТ (комментарии парсятся Tree-sitter как named CST nodes и искажают фингерпринт)
  Пример 5 (TSX переименование JSX-компонентов и пропсов): ПРОПУЩЕНО (семантическая слепота фингерпринта: компоненты парсятся как generic identifier)
  Fault-tolerance на битом коде: FAIL (при незакрытой скобке/синтаксическом мусоре parse() возвращает Ok(Tree) с has_error() == true, а discover_tests() молча возвращает Ok(tests), нарушая Инвариант 5 "Fail Loudly")

R3 — Правила E001-E003:
  E001.1 (вложенный describe.skip внутри чистого набора): ПОЙМАНО (рекурсивный обход inspect_js_member_expression генерирует E001)
  E001.2 (условный #[cfg_attr(test, ignore)] в Rust): ПРОПУЩЕНО (Критический дефект: инспектор проверяет только первый дочерний идентификатор cfg_attr, игнорируя аргумент ignore)
  E001.3 (динамический .skip через переменную describe[s]): ПРОПУЩЕНО (Критический дефект: обращение через скобки парсится как subscript_expression и отбрасывается правилом)
  E002.1 (тавтология через промежуточную переменную let x = compute(); assert_eq!(x, x)): ПОЙМАНО (синтаксическое сравнение l == r перехватывает одинаковые имена идентификаторов)
  E002.2 (вычисляемое константное выражение assert_eq!(1 + 1, 2)): ПРОПУЩЕНО (Критический дефект: отсутствие constant folding; выражения с операторами не признаются литералами)
  E002.3 (тавтология внутри дизъюнкции assert!(1 == 1 || complex_condition)): ПРОПУЩЕНО (Критический дефект: find_binary_op не парсит логические || / &&)
  E003.1 (ассерт на тип вместо значения expect(typeof x).toBe('string')): ПРОПУЩЕНО (Критический дефект: правило считает любой вызов expect() валидным без оценки глубины проверки)
  E003.2 (ассерт в недостижимом коде после return / panic!): ПРОПУЩЕНО (Критический дефект: отсутствие анализа достижимости CFG; мёртвые ассерты засчитываются в assertion floor)
  E003.3 (ассерт в мёртвой ветке if false { assert!(...) } и bare expect()): ПРОПУЩЕНО (Критический дефект: безусловный обход AST засчитывает ассерты в неисполняемых ветках и пустые вызовы expect())

R4 — Латентность:
  Замеренные min/max/mean на 50 staged файлах (25 Rust + 25 TS):
    - CLI Process (test-guard check --staged): Min: 6.81 ms | Max: 12.86 ms | Mean: 8.33 ms (PASS, но в CLI пока выводится заглушка)
    - Real In-Process AST Engine (последовательно, 1 поток): Min: 24.34 ms | Max: 72.95 ms | Mean: 37.83 ms (FAIL бюджета)
    - Real In-Process AST Engine (параллельно, Rayon): Min: 6.30 ms | Max: 14.43 ms | Mean: 8.74 ms (PASS бюджета)
  Соответствие бюджету <25ms: PASS (при использовании Rayon multi-threading / 8.74ms mean; в однопоточном режиме FAIL / 37.83ms)

R5 — Mock boundary (если применимо): NOT_APPLICABLE (модуль crates/rules/src/anti_mock.rs содержит только заголовочный комментарий; реализация запланирована на Фазу 2.4)

НАЙДЕННЫЕ ДЕФЕКТЫ (только подтверждённые воспроизведением):
  1. [Critical] Нарушение инварианта No Silent Fallbacks в AST-парсере — crates/ast/src/parser.rs:20 и queries.rs:33. Воспроизведение: файл с незакрытой фигурной скобкой успешно парсится в Ok(Tree) и возвращает Ok(tests), tree.root_node().has_error() игнорируется.
  2. [Critical] Пропуск #[cfg_attr(test, ignore)] в E001 — crates/rules/src/anti_skip.rs:80-92. Воспроизведение: inspect_rust_function читает named_child(0) атрибута (cfg_attr) и не заходит в аргументы.
  3. [Critical] Пропуск динамического .skip в E001 — crates/rules/src/anti_skip.rs:142-200. Воспроизведение: test[s] парсится как subscript_expression и игнорируется.
  4. [Critical] Пропуск константных тавтологий и логических дизъюнкций в E002 — crates/rules/src/anti_tautology.rs:82-94, 231-250. Воспроизведение: assert_eq!(1 + 1, 2) и assert!(1 == 1 || cond) не детектируются из-за отсутствия constant folding и парсинга ||.
  5. [Critical] Пропуск недостижимых ассертов и мёртвого кода в E003 — crates/rules/src/anti_hollowing.rs:121-135, 170-186. Воспроизведение: ассерты после return;, внутри if false { ... }, а также bare expect() без аргументов и матчера засчитываются в assertion floor.
  6. [High] Расхождение CLI и реального пайплайна проверки — crates/cli/src/main.rs:46-51. Воспроизведение: test-guard check --staged выполняет println! вместо вызова rules engine и git reader.
  7. [Medium] Включение комментариев в BLAKE3 Structural Fingerprint — crates/ast/src/fingerprint.rs:16. Воспроизведение: добавление // comment меняет отпечаток, так как node.is_named() == true.

VERDICT: BASELINE NEEDS REWORK (переход к Фазе 2.4 блокируют критические дефекты в E001-E003 и тихий пропуск синтаксических ошибок в crates/ast, требующие исправления перед началом разработки anti-mock изоляции)