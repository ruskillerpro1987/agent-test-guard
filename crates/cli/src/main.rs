#[cfg(windows)]
#[link(name = "advapi32")]
extern "C" {}

use std::path::Path;
use clap::{Parser, Subcommand};
use git2::{Repository, StatusOptions};

use agent_test_guard_ast::grammar::detect_language;
use agent_test_guard_rules::anti_skip::AntiSkipRule;
use agent_test_guard_rules::anti_tautology::AntiTautologyRule;
use agent_test_guard_rules::diagnostic::Diagnostic;

#[derive(Parser, Debug)]
#[command(name = "test-guard")]
#[command(author, version, about = "Deterministic Anti-Reward-Hacking & Test Authenticity Gate for AI Coding Agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize test baseline for current workspace
    Init,
    /// Verify test authenticity and ratchet invariants
    Check {
        /// Check staged git blobs only (<25ms pre-commit gate)
        #[arg(long)]
        staged: bool,
        /// Run deep CI verification including dynamic test runner dry-runs
        #[arg(long)]
        deep: bool,
    },
    /// Advance baseline count monotonically
    Ratchet {
        /// Allow baseline count shrink (requires Ed25519 signature)
        #[arg(long)]
        allow_shrink: bool,
    },
    /// Apply local OS write-protection and install Git pre-commit hooks
    Protect,
    /// Explain a diagnostic rule code and its remediation
    Explain {
        /// Diagnostic error code (e.g., E001, E002)
        code: String,
    },
}

fn check_staged_repo(repo_path: &Path) -> Result<Vec<Diagnostic>, String> {
    let repo = Repository::discover(repo_path)
        .map_err(|e| format!("Failed to discover git repository: {e}"))?;

    let index = repo
        .index()
        .map_err(|e| format!("Failed to read git index: {e}"))?;

    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(false);

    let statuses = repo
        .statuses(Some(&mut status_opts))
        .map_err(|e| format!("Failed to fetch git statuses: {e}"))?;

    let mut diagnostics = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        if status.is_index_new() || status.is_index_modified() {
            let path_str = entry
                .path()
                .ok_or_else(|| "Invalid UTF-8 path in git index".to_string())?;

            let Some(lang) = detect_language(path_str) else {
                continue;
            };

            let index_entry = index
                .get_path(Path::new(path_str), 0)
                .ok_or_else(|| format!("Staged file missing from index: {path_str}"))?;

            let blob = repo
                .find_blob(index_entry.id)
                .map_err(|e| format!("Failed to find blob for {path_str}: {e}"))?;

            let source = match std::str::from_utf8(blob.content()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if let Ok(mut diags) = AntiSkipRule::check_source(path_str, source, lang) {
                diagnostics.append(&mut diags);
            }

            if let Ok(mut diags) = AntiTautologyRule::check_source(path_str, source, lang) {
                diagnostics.append(&mut diags);
            }
        }
    }

    Ok(diagnostics)
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            println!("Initializing Agent Test Guard baseline...");
        }
        Some(Commands::Check { staged, deep }) => {
            if staged {
                let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
                match check_staged_repo(&current_dir) {
                    Ok(diagnostics) => {
                        if !diagnostics.is_empty() {
                            for diag in &diagnostics {
                                eprintln!(
                                    "[{}] {}:{}:{} - {}",
                                    diag.code, diag.file_path, diag.span.start_line, diag.span.start_col, diag.message
                                );
                            }
                            std::process::exit(1);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error inspecting staged files: {err}");
                        std::process::exit(1);
                    }
                }
            } else {
                println!(
                    "Running verification check (staged: {}, deep: {})...",
                    staged, deep
                );
            }
        }
        Some(Commands::Ratchet { allow_shrink }) => {
            println!("Ratcheting baseline (allow_shrink: {})...", allow_shrink);
        }
        Some(Commands::Protect) => {
            println!("Installing protection hooks and applying OS write-locks...");
        }
        Some(Commands::Explain { code }) => {
            println!("Explaining rule {}", code);
        }
        None => {
            println!("Agent Test Guard v0.1.0. Run with --help for usage.");
        }
    }
}