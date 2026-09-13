use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            println!("Initializing Agent Test Guard baseline...");
        }
        Some(Commands::Check { staged, deep }) => {
            println!("Running verification check (staged: {}, deep: {})...", staged, deep);
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
