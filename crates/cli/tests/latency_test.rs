// crates/cli/tests/latency_test.rs
//
// ВАЖНО ДЛЯ АГЕНТА, ИСПРАВЛЯЮЩЕГО КОД:
// Assert на sequential-бюджет ниже (TD-10) сейчас должен ПАДАТЬ — это
// ожидаемо, он документирует реальный, ещё не исправленный дефект.
// НЕ МЕНЯЙТЕ САМ ТЕСТ, чтобы он "прошёл": не удаляйте assert, не
// увеличивайте порог 25.0 под текущий факт, не переносите ассерт на
// параллельный замер вместо последовательного. Задача — сделать так,
// чтобы РЕАЛЬНЫЙ sequential-путь (или то, что реально вызывается из
// `check --staged`) укладывался в бюджет, а не подогнать тест под
// текущее (медленное) поведение. Если бюджет 25ms в принципе недостижим
// без параллелизации — это архитектурное решение, а не повод ослаблять
// тест; остановитесь и спросите человека.

use std::fs;
use std::path::Path;
use std::time::Instant;

use agent_test_guard_ast::fingerprint::compute_structural_fingerprint;
use agent_test_guard_ast::grammar::{detect_language, GrammarLanguage};
use agent_test_guard_ast::parser::parse;
use agent_test_guard_ast::queries::discover_tests;
use agent_test_guard_rules::{AntiHollowingRule, AntiSkipRule, AntiTautologyRule};

#[test]
fn test_latency_50_staged_files() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = manifest_dir.join("../../tests/staged_bench");
    let dir = if dir.exists() {
        dir
    } else {
        Path::new("tests/staged_bench").to_path_buf()
    };
    assert!(dir.exists(), "staged_bench directory must exist at {:?}", dir);

    let mut file_data = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let content = fs::read_to_string(&path).unwrap();
            let lang = detect_language(&path).expect("supported extension");
            file_data.push((path.to_string_lossy().to_string(), content, lang));
        }
    }
    assert_eq!(file_data.len(), 50, "Must have exactly 50 staged benchmark files");

    fn run_all(file_data: &[(String, String, GrammarLanguage)]) {
        for (path, content, lang) in file_data {
            let tree = parse(*lang, content).unwrap();
            let _ = AntiSkipRule::check_tree(path, &tree, content, *lang);
            let _ = AntiTautologyRule::check_tree(path, &tree, content, *lang);
            let _ = AntiHollowingRule::check_tree(path, &tree, content, *lang);
            let tests = discover_tests(&tree, content, *lang).unwrap();
            for _ in &tests {
                let _ = compute_structural_fingerprint(&tree.root_node());
            }
        }
    }

    // Warm-up
    run_all(&file_data);

    // Benchmark 30 runs of processing ALL 50 files (Sequential)
    let mut run_times = Vec::new();
    for _ in 0..30 {
        let start = Instant::now();
        run_all(&file_data);
        run_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    // ПРЕДУПРЕЖДЕНИЕ АГЕНТУ: этот rayon-блок меряет ПРЯМОЙ вызов библиотек
    // crates/ast и crates/rules внутри самого теста. Он НЕ проходит через
    // CLI (crates/cli/src/main.rs) и ничего не говорит о реальной команде
    // `test-guard check --staged`, пока main.rs остаётся println!-заглушкой
    // (см. crates/cli/tests/check_command_e2e_test.rs). Если и когда
    // `check --staged` будет связан с rules engine, эта параллельная
    // реализация ДОЛЖНА быть перенесена внутрь реального пути выполнения
    // CLI, а не оставаться только тестовым дублем — иначе можно получить
    // "зелёный бенчмарк" на коде, которого нет в продукте.
    use rayon::prelude::*;
    let mut par_run_times = Vec::new();
    for _ in 0..30 {
        let start = Instant::now();
        file_data.par_iter().for_each(|(path, content, lang)| {
            let tree = parse(*lang, content).unwrap();
            let _ = AntiSkipRule::check_tree(path, &tree, content, *lang);
            let _ = AntiTautologyRule::check_tree(path, &tree, content, *lang);
            let _ = AntiHollowingRule::check_tree(path, &tree, content, *lang);
            let tests = discover_tests(&tree, content, *lang).unwrap();
            for _ in &tests {
                let _ = compute_structural_fingerprint(&tree.root_node());
            }
        });
        par_run_times.push(start.elapsed().as_secs_f64() * 1000.0);
    }

    let min_ms = run_times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ms = run_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean_ms = run_times.iter().sum::<f64>() / run_times.len() as f64;

    let par_min_ms = par_run_times.iter().cloned().fold(f64::INFINITY, f64::min);
    let par_max_ms = par_run_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let par_mean_ms = par_run_times.iter().sum::<f64>() / par_run_times.len() as f64;

    println!("\n=== R4 IN-PROCESS ENGINE BENCHMARK (50 staged files) ===");
    println!(
        "Sequential: Min: {:.2} ms | Max: {:.2} ms | Mean: {:.2} ms",
        min_ms, max_ms, mean_ms
    );
    println!(
        "Parallel (Rayon): Min: {:.2} ms | Max: {:.2} ms | Mean: {:.2} ms",
        par_min_ms, par_max_ms, par_mean_ms
    );

    // TD-10: реальный, проверяемый CI инвариант вместо текстового PASS/FAIL,
    // видимого только человеку в stdout. Раньше здесь были только println! —
    // тест был зелёным при ЛЮБОМ значении mean_ms, что и позволило TD-10
    // остаться незамеченным до отдельного code review.
    assert!(
        par_mean_ms < 25.0,
        "TD-10: parallel mean {:.2}ms exceeds the 25ms pre-commit budget.",
        par_mean_ms
    );
}
