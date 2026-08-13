use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
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

    /// Output format; auto uses text on a terminal and JSON when piped
    #[arg(long, short = 'o', value_enum, default_value = "auto", global = true)]
    pub output: OutputFormat,

    /// Suppress output, rely on exit code only
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Filter JSON output to specific fields (comma-separated)
    #[arg(long, global = true)]
    pub fields: Option<String>,

    /// Maximum records returned by list
    #[arg(long, default_value_t = 100, global = true)]
    pub limit: usize,

    /// Records to skip before returning list results
    #[arg(long, default_value_t = 0, global = true)]
    pub offset: usize,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Auto,
    Json,
    Text,
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
