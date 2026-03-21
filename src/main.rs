mod check;
mod cli;
mod cron;
mod output;

use std::io::IsTerminal;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() {
    if let Err(err) = run() {
        let json_mode = std::env::args().any(|a| a == "--json" || a == "-j")
            || !std::io::stdout().is_terminal();

        if json_mode {
            let envelope = serde_json::json!({
                "ok": false,
                "error": { "message": err.to_string() },
            });
            println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
        } else {
            eprintln!("Error: {err}");
        }

        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let json_output = cli.json || !std::io::stdout().is_terminal();

    match cli.command {
        Command::List { user, all } => cmd_list(user.as_deref(), all, json_output),
        Command::Explain { expression } => cmd_explain(&expression, json_output),
        Command::Check { user, all } => cmd_check(user.as_deref(), all, json_output),
        Command::Timeline { hours } => cmd_timeline(hours, json_output),
    }
}

fn cmd_list(user: Option<&str>, all: bool, json_output: bool) -> Result<()> {
    let jobs = cron::list_all_jobs(user, all)?;

    if json_output {
        output::print_json_success(&jobs);
    } else {
        output::print_jobs_table(&jobs);
    }

    Ok(())
}

fn cmd_explain(expression: &str, json_output: bool) -> Result<()> {
    let description = cron::explain_schedule(expression);

    if json_output {
        output::print_json_success(&serde_json::json!({
            "expression": expression,
            "description": description,
        }));
    } else {
        output::print_explain(expression, &description);
    }

    Ok(())
}

fn cmd_check(user: Option<&str>, all: bool, json_output: bool) -> Result<()> {
    let jobs = cron::list_all_jobs(user, all)?;
    let issues = check::check_jobs(&jobs);

    if json_output {
        output::print_json_success(&serde_json::json!({
            "jobs_checked": jobs.len(),
            "issues": issues,
        }));
    } else {
        output::print_issues(&issues);
    }

    Ok(())
}

fn cmd_timeline(_hours: u32, _json_output: bool) -> Result<()> {
    eprintln!("Timeline not yet implemented");
    Ok(())
}
