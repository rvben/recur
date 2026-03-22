mod check;
mod cli;
mod cron;
mod fields;
mod output;
mod schema;
mod timeline;
mod util;

use std::io::IsTerminal;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

/// Exit code: issues found by `check`
const EXIT_ISSUES: u8 = 2;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(err) => {
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

            ExitCode::from(1)
        }
    }
}

fn run() -> Result<u8> {
    let cli = Cli::parse();
    let fields = cli.fields.as_deref();
    let json_output = cli.json || fields.is_some() || !std::io::stdout().is_terminal();
    let quiet = cli.quiet;

    match cli.command {
        Command::List { user, all } => {
            cmd_list(user.as_deref(), all, json_output, quiet, fields)?;
            Ok(0)
        }
        Command::Explain { expression } => {
            cmd_explain(&expression, json_output, quiet, fields)?;
            Ok(0)
        }
        Command::Check { user, all, dry_run } => {
            cmd_check(user.as_deref(), all, dry_run, json_output, quiet, fields)
        }
        Command::Timeline { hours, user, all } => {
            cmd_timeline(user.as_deref(), all, hours, json_output, quiet, fields)?;
            Ok(0)
        }
        Command::Schema => {
            cmd_schema()?;
            Ok(0)
        }
        Command::Completions { shell } => {
            cli::print_completions(shell);
            Ok(0)
        }
    }
}

fn cmd_list(
    user: Option<&str>,
    all: bool,
    json_output: bool,
    quiet: bool,
    fields: Option<&str>,
) -> Result<()> {
    let jobs = cron::list_all_jobs(user, all)?;

    if quiet {
        return Ok(());
    }

    if json_output {
        print_json(&jobs, fields);
    } else {
        output::print_jobs_table(&jobs);
    }

    Ok(())
}

fn cmd_explain(
    expression: &str,
    json_output: bool,
    quiet: bool,
    fields: Option<&str>,
) -> Result<()> {
    let description = cron::explain_schedule(expression);

    if quiet {
        return Ok(());
    }

    if json_output {
        let data = serde_json::json!({
            "expression": expression,
            "description": description,
        });
        print_json(&data, fields);
    } else {
        output::print_explain(expression, &description);
    }

    Ok(())
}

fn cmd_check(
    user: Option<&str>,
    all: bool,
    dry_run: bool,
    json_output: bool,
    quiet: bool,
    fields: Option<&str>,
) -> Result<u8> {
    let jobs = cron::list_all_jobs(user, all)?;

    if dry_run {
        if !quiet {
            let preview: Vec<serde_json::Value> = jobs
                .iter()
                .map(|j| {
                    serde_json::json!({
                        "user": j.user,
                        "schedule": j.schedule,
                        "command": j.command,
                        "source": j.source.to_string(),
                    })
                })
                .collect();

            if json_output {
                let data = serde_json::json!({
                    "dry_run": true,
                    "jobs_to_check": jobs.len(),
                    "jobs": preview,
                });
                print_json(&data, fields);
            } else {
                println!("Dry run: would check {} job(s):", jobs.len());
                for job in &jobs {
                    println!("  {} [{}] {}", job.user, job.source, job.command);
                }
            }
        }
        return Ok(0);
    }

    let issues = check::check_jobs(&jobs);
    let has_issues = !issues.is_empty();

    if !quiet {
        if json_output {
            let data = serde_json::json!({
                "jobs_checked": jobs.len(),
                "issues": issues,
            });
            print_json_with_status(&data, fields, !has_issues);
        } else {
            output::print_issues(&issues);
        }
    }

    if has_issues { Ok(EXIT_ISSUES) } else { Ok(0) }
}

fn cmd_timeline(
    user: Option<&str>,
    all: bool,
    hours: u32,
    json_output: bool,
    quiet: bool,
    fields: Option<&str>,
) -> Result<()> {
    let jobs = cron::list_all_jobs(user, all)?;
    let tl = timeline::build_timeline(&jobs, hours);

    if quiet {
        return Ok(());
    }

    if json_output {
        print_json(&tl, fields);
    } else {
        timeline::print_timeline(&tl);
    }

    Ok(())
}

fn cmd_schema() -> Result<()> {
    let schema = schema::build_schema();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    Ok(())
}

/// Print JSON with optional field filtering via --fields.
fn print_json<T: serde::Serialize>(data: &T, fields_str: Option<&str>) {
    print_json_with_status(data, fields_str, true);
}

/// Print JSON with explicit ok status and optional field filtering.
fn print_json_with_status<T: serde::Serialize>(data: &T, fields_str: Option<&str>, ok: bool) {
    let value = serde_json::to_value(data).unwrap();

    let filtered = match fields_str {
        Some(f) => fields::filter_fields_with_status(&value, f, ok),
        None => serde_json::json!({ "ok": ok, "data": value }),
    };

    println!("{}", serde_json::to_string_pretty(&filtered).unwrap());
}
