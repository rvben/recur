use serde::Serialize;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::cron::CronJob;

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub severity: Severity,
    pub job_command: String,
    pub user: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARN"),
        }
    }
}

/// Check a list of cron jobs for common issues.
pub fn check_jobs(jobs: &[CronJob]) -> Vec<Issue> {
    let mut issues = Vec::new();

    for job in jobs {
        // Extract the actual executable from the command
        let executable = extract_executable(&job.command);

        // Check if the script/binary exists
        if let Some(path) = executable
            && !path.starts_with('$')
            && !path.starts_with('(')
        {
            let p = Path::new(path);
            if p.is_absolute() {
                if !p.exists() {
                    issues.push(Issue {
                        severity: Severity::Error,
                        job_command: job.command.clone(),
                        user: job.user.clone(),
                        message: format!("executable not found: {path}"),
                    });
                } else if p.is_file() {
                    // Check if executable
                    if let Ok(meta) = p.metadata() {
                        let perms = meta.permissions();
                        if perms.mode() & 0o111 == 0 {
                            issues.push(Issue {
                                severity: Severity::Error,
                                job_command: job.command.clone(),
                                user: job.user.clone(),
                                message: format!("file not executable: {path}"),
                            });
                        }
                    }
                }
            }
        }

        // Check for common mistakes
        if job.schedule == "* * * * *" && !job.command.contains("sleep") {
            issues.push(Issue {
                severity: Severity::Warning,
                job_command: job.command.clone(),
                user: job.user.clone(),
                message: "runs every minute — is this intentional?".to_string(),
            });
        }

        // Check for no output redirection (mail noise)
        if !job.command.contains('>')
            && !job.command.contains(">/dev/null")
            && !job.command.contains("logger")
            && !job.command.contains("2>&1")
        {
            issues.push(Issue {
                severity: Severity::Warning,
                job_command: job.command.clone(),
                user: job.user.clone(),
                message: "no output redirection — output will be mailed".to_string(),
            });
        }
    }

    // Check for overlapping schedules (same schedule, same user)
    for i in 0..jobs.len() {
        for j in (i + 1)..jobs.len() {
            if jobs[i].schedule == jobs[j].schedule && jobs[i].user == jobs[j].user {
                issues.push(Issue {
                    severity: Severity::Warning,
                    job_command: jobs[i].command.clone(),
                    user: jobs[i].user.clone(),
                    message: format!(
                        "same schedule as another job: {}",
                        truncate(&jobs[j].command, 60)
                    ),
                });
            }
        }
    }

    issues
}

/// Extract the first executable path from a cron command.
fn extract_executable(command: &str) -> Option<&str> {
    let cmd = command.trim();

    // Handle commands starting with env vars or redirections
    let cmd = if cmd.starts_with("cd ") {
        // cd /some/dir && /path/to/script
        cmd.split("&&").nth(1).map(str::trim).unwrap_or(cmd)
    } else {
        cmd
    };

    // Handle sudo, nice, ionice prefixes
    let cmd = cmd
        .strip_prefix("sudo ")
        .or_else(|| cmd.strip_prefix("nice "))
        .or_else(|| cmd.strip_prefix("ionice "))
        .unwrap_or(cmd)
        .trim();

    // Skip shell invocations
    let cmd = cmd
        .strip_prefix("/bin/sh -c ")
        .or_else(|| cmd.strip_prefix("/bin/bash -c "))
        .unwrap_or(cmd)
        .trim()
        .trim_start_matches('\'')
        .trim_start_matches('"');

    // Get first word
    cmd.split_whitespace().next()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::CronSource;

    fn make_job(schedule: &str, command: &str, user: &str) -> CronJob {
        CronJob {
            user: user.to_string(),
            schedule: schedule.to_string(),
            command: command.to_string(),
            description: String::new(),
            source: CronSource::UserCrontab,
        }
    }

    #[test]
    fn detects_missing_executable() {
        let jobs = vec![make_job(
            "0 * * * *",
            "/nonexistent/path/script.sh",
            "ruben",
        )];
        let issues = check_jobs(&jobs);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("executable not found"))
        );
    }

    #[test]
    fn warns_every_minute() {
        let jobs = vec![make_job("* * * * *", "echo hello", "ruben")];
        let issues = check_jobs(&jobs);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("runs every minute"))
        );
    }

    #[test]
    fn warns_no_output_redirect() {
        let jobs = vec![make_job("0 * * * *", "/usr/bin/something", "ruben")];
        let issues = check_jobs(&jobs);
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("output will be mailed"))
        );
    }

    #[test]
    fn no_warning_with_redirect() {
        let jobs = vec![make_job(
            "0 * * * *",
            "/usr/bin/something > /dev/null 2>&1",
            "ruben",
        )];
        let issues = check_jobs(&jobs);
        assert!(
            !issues
                .iter()
                .any(|i| i.message.contains("output will be mailed"))
        );
    }

    #[test]
    fn detects_overlapping_schedules() {
        let jobs = vec![
            make_job("0 3 * * *", "/usr/bin/job1", "ruben"),
            make_job("0 3 * * *", "/usr/bin/job2", "ruben"),
        ];
        let issues = check_jobs(&jobs);
        assert!(issues.iter().any(|i| i.message.contains("same schedule")));
    }

    #[test]
    fn no_overlap_different_users() {
        let jobs = vec![
            make_job("0 3 * * *", "/usr/bin/job1", "ruben"),
            make_job("0 3 * * *", "/usr/bin/job2", "root"),
        ];
        let issues = check_jobs(&jobs);
        assert!(!issues.iter().any(|i| i.message.contains("same schedule")));
    }

    #[test]
    fn extract_executable_simple() {
        assert_eq!(
            extract_executable("/usr/bin/backup.sh --full"),
            Some("/usr/bin/backup.sh")
        );
    }

    #[test]
    fn extract_executable_with_cd() {
        assert_eq!(
            extract_executable("cd /var/app && /usr/bin/run.sh"),
            Some("/usr/bin/run.sh")
        );
    }

    #[test]
    fn extract_executable_with_sudo() {
        assert_eq!(
            extract_executable("sudo /usr/bin/cleanup.sh"),
            Some("/usr/bin/cleanup.sh")
        );
    }
}
