use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    about = "A human-friendly cron job manager",
    version,
    after_long_help = "\
Examples:
  ogni list
  ogni list --user root
  ogni explain \"*/5 * * * *\"
  ogni check
  ogni timeline"
)]
pub struct Cli {
    /// Output as JSON
    #[arg(long, short = 'j', global = true)]
    pub json: bool,

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
    },

    /// Show a visual timeline of when jobs run
    Timeline {
        /// Number of hours to show (default: 24)
        #[arg(long, default_value = "24")]
        hours: u32,
    },
}
