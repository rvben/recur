use std::io::IsTerminal;

use owo_colors::OwoColorize;

use crate::check::{Issue, Severity};
use crate::cron::CronJob;
use crate::util::truncate;

pub fn use_color() -> bool {
    std::io::stdout().is_terminal()
}

pub fn print_jobs_table(jobs: &[CronJob]) {
    if jobs.is_empty() {
        eprintln!("No cron jobs found.");
        return;
    }

    let color = use_color();

    let header = format!(
        " {:<12} {:<20} {:<34} {}",
        "User", "Schedule", "Description", "Command"
    );
    if color {
        println!("{}", header.bold());
        println!("{}", "\u{2500}".repeat(90).dimmed());
    } else {
        println!("{header}");
        println!("{}", "-".repeat(90));
    }

    for job in jobs {
        let cmd_display = truncate(&job.command, 40);
        if color {
            println!(
                " {:<12} {:<20} {:<34} {}",
                job.user.dimmed(),
                job.schedule.cyan(),
                job.description.green(),
                cmd_display.dimmed(),
            );
        } else {
            println!(
                " {:<12} {:<20} {:<34} {}",
                job.user, job.schedule, job.description, cmd_display,
            );
        }
    }

    if color {
        println!();
        println!("{}", format!("{} job(s) found", jobs.len()).dimmed());
    }
}

pub fn print_issues(issues: &[Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    let color = use_color();

    for issue in issues {
        let severity_str = match issue.severity {
            Severity::Error => {
                if color {
                    "ERROR".red().bold().to_string()
                } else {
                    "ERROR".to_string()
                }
            }
            Severity::Warning => {
                if color {
                    " WARN".yellow().to_string()
                } else {
                    " WARN".to_string()
                }
            }
        };

        let cmd_display = truncate(&issue.job_command, 50);
        if color {
            println!(
                " {} {} {}",
                severity_str,
                issue.message,
                cmd_display.dimmed(),
            );
        } else {
            println!(" {} {} {}", severity_str, issue.message, cmd_display);
        }
    }

    let errors = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();
    let warnings = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();

    println!();
    if color {
        if errors > 0 {
            print!("{} ", format!("{errors} error(s)").red());
        }
        if warnings > 0 {
            print!("{} ", format!("{warnings} warning(s)").yellow());
        }
        println!();
    } else {
        println!("{errors} error(s), {warnings} warning(s)");
    }
}

pub fn print_explain(expression: &str, description: &str) {
    let color = use_color();
    if color {
        println!("  {} {}", "Expression:".dimmed(), expression.cyan());
        println!("  {}    {}", "Schedule:".dimmed(), description.green());
    } else {
        println!("  Expression: {expression}");
        println!("  Schedule:   {description}");
    }
}
