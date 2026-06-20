use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    about = "A human-friendly cron job manager",
    version,
    after_long_help = "\
Examples:
  recur list
  recur list --user root
  recur explain \"*/5 * * * *\"
  recur check
  recur timeline
  recur schema"
)]
pub struct Cli {
    /// Output as JSON
    #[arg(long, short = 'j', global = true)]
    pub json: bool,

    /// Suppress output, rely on exit code only
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Filter JSON output to specific fields (comma-separated)
    #[arg(long, global = true)]
    pub fields: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all cron jobs with human-readable schedules
    List {
        /// Show jobs for a specific user (requires root for other users)
        #[arg(long, short = 'u')]
        user: Option<String>,

        /// Show all users' cron jobs (requires root)
        #[arg(long, short = 'a')]
        all: bool,
    },

    /// Explain a cron expression in plain English
    Explain {
        /// Cron expression (e.g. "*/5 * * * *")
        expression: String,
    },

    /// Check cron jobs for issues (missing scripts, permission problems)
    Check {
        /// Check jobs for a specific user
        #[arg(long, short = 'u')]
        user: Option<String>,

        /// Check all users' cron jobs (requires root)
        #[arg(long, short = 'a')]
        all: bool,

        /// Preview what would be checked without executing
        #[arg(long)]
        dry_run: bool,
    },

    /// Show a visual timeline of when jobs run
    Timeline {
        /// Number of hours to show (default: 24)
        #[arg(long, default_value = "24")]
        hours: u32,

        /// Show jobs for a specific user
        #[arg(long, short = 'u')]
        user: Option<String>,

        /// Show all users' cron jobs (requires root)
        #[arg(long, short = 'a')]
        all: bool,
    },

    /// Output full command schema as JSON (for AI agents and tooling)
    Schema,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

pub fn print_completions(shell: Shell) {
    let mut cmd = Cli::command();
    clap_complete::generate(shell, &mut cmd, "recur", &mut std::io::stdout());
}
