// crates/cli/tests/check_command_e2e_test.rs
//
// РЕГРЕССИОННЫЙ E2E-ТЕСТ на найденный при code review дефект:
// `test-guard check --staged` (crates/cli/src/main.rs) сейчас — println!-
// заглушка, не читающая git index и не вызывающая rules engine
// (crates/rules). Заведомо нарушающий staged-файл проходит проверку
// "успешно", потому что никакая проверка на самом деле не выполняется.
//
// ВАЖНО ДЛЯ АГЕНТА, ИСПРАВЛЯЮЩЕГО КОД:
// Тест сейчас (до фикса) должен ПАДАТЬ. НЕ МЕНЯЙТЕ САМ ТЕСТ, чтобы он
// "прошёл" — не убирайте проверку exit code, не подменяйте фикстуру
// bad_test.rs на что-то менее строгое, не удаляйте проверку кода "E001"
// в выводе. Если считаете, что формат вывода/exit code должен быть
// другим — остановитесь и спросите человека, не переписывайте тест сами.
//
// ПРИМЕЧАНИЕ: если бинарный таргет в crates/cli/Cargo.toml называется не
// "test-guard" (проверьте секцию [[bin]] или имя пакета, если [[bin]]
// отсутствует), поправьте имя в env!("CARGO_BIN_EXE_...") ниже на
// реальное. Само по себе несовпадение имени — это ошибка компиляции
// теста, а не повод его удалять.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_temp_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("agent-test-guard-e2e-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn run_ok(cmd: &mut Command) {
    let status = cmd.status().expect("failed to spawn process");
    assert!(status.success(), "setup command failed: {:?}", cmd);
}

#[test]
fn bug_check_staged_must_detect_e001_violation_via_real_git_index() {
    let repo = unique_temp_dir("check-staged");

    run_ok(Command::new("git").arg("init").arg("-q").current_dir(&repo));
    run_ok(
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo),
    );
    run_ok(
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo),
    );

    // Заведомо нарушающий файл: #[ignore] должен триггерить E001 Anti-Skip.
    let bad_file = repo.join("bad_test.rs");
    fs::write(
        &bad_file,
        r#"
            #[test]
            #[ignore]
            fn test_should_be_flagged() {
                assert!(true);
            }
        "#,
    )
    .unwrap();

    // Контрольный чистый файл — не должен давать ложных срабатываний.
    let good_file = repo.join("good_test.rs");
    fs::write(
        &good_file,
        r#"
            #[test]
            fn test_is_fine() {
                assert_eq!(2 + 2, 4);
            }
        "#,
    )
    .unwrap();

    run_ok(Command::new("git").arg("add").arg(".").current_dir(&repo));

    let bin = env!("CARGO_BIN_EXE_test-guard");
    let output = Command::new(bin)
        .arg("check")
        .arg("--staged")
        .current_dir(&repo)
        .output()
        .expect("failed to run test-guard binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    fs::remove_dir_all(&repo).ok();

    assert!(
        !output.status.success(),
        "BUG: `test-guard check --staged` вернул успешный exit code для staged-\
         файла с #[ignore], хотя это должно быть нарушением E001 Anti-Skip. \
         Текущая реализация Check в main.rs — println!-заглушка, не \
         вызывающая rules engine и не читающая git index. Raw output:\n{combined}"
    );
    assert!(
        combined.contains("E001"),
        "BUG: вывод `test-guard check --staged` не содержит кода диагностики \
         E001 для заведомого нарушения anti-skip. Raw output:\n{combined}"
    );
}

#[test]
fn control_check_staged_must_pass_clean_repo() {
    let repo = unique_temp_dir("check-staged-clean");

    run_ok(Command::new("git").arg("init").arg("-q").current_dir(&repo));
    run_ok(
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo),
    );
    run_ok(
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo),
    );

    let good_file = repo.join("good_test.rs");
    fs::write(
        &good_file,
        r#"
            #[test]
            fn test_is_fine() {
                assert_eq!(2 + 2, 4);
            }
        "#,
    )
    .unwrap();

    run_ok(Command::new("git").arg("add").arg(".").current_dir(&repo));

    let bin = env!("CARGO_BIN_EXE_test-guard");
    let output = Command::new(bin)
        .arg("check")
        .arg("--staged")
        .current_dir(&repo)
        .output()
        .expect("failed to run test-guard binary");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::remove_dir_all(&repo).ok();

    assert!(
        output.status.success(),
        "`test-guard check --staged` must exit 0 on a clean staged repo with \
         no violations. Raw output:\n{combined}"
    );
}
