use anyhow::{Context, Result};
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct CronJob {
    pub user: String,
    pub schedule: String,
    pub command: String,
    pub description: String,
    pub source: CronSource,
}

#[derive(Debug, Clone, Serialize)]
pub enum CronSource {
    UserCrontab,
    SystemCrontab,
    CronD(String),
}

impl std::fmt::Display for CronSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserCrontab => write!(f, "crontab"),
            Self::SystemCrontab => write!(f, "/etc/crontab"),
            Self::CronD(name) => write!(f, "/etc/cron.d/{name}"),
        }
    }
}

/// Parse a crontab line into a CronJob, skipping comments and empty lines.
fn parse_crontab_line(line: &str, user: &str, source: CronSource) -> Option<CronJob> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.contains('=') {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(6, char::is_whitespace).collect();
    if parts.len() < 6 {
        return None;
    }

    let schedule = format!(
        "{} {} {} {} {}",
        parts[0], parts[1], parts[2], parts[3], parts[4]
    );
    let command = parts[5].trim().to_string();
    let description = explain_schedule(&schedule);

    Some(CronJob {
        user: user.to_string(),
        schedule,
        command,
        description,
        source,
    })
}

/// Parse a system crontab line (has user field between schedule and command).
fn parse_system_crontab_line(line: &str, source: CronSource) -> Option<CronJob> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.contains('=') {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(7, char::is_whitespace).collect();
    if parts.len() < 7 {
        return None;
    }

    let schedule = format!(
        "{} {} {} {} {}",
        parts[0], parts[1], parts[2], parts[3], parts[4]
    );
    let user = parts[5].to_string();
    let command = parts[6].trim().to_string();
    let description = explain_schedule(&schedule);

    Some(CronJob {
        user,
        schedule,
        command,
        description,
        source,
    })
}

/// List cron jobs for the current user.
pub fn list_user_crontab(user: Option<&str>) -> Result<Vec<CronJob>> {
    let mut cmd = Command::new("crontab");
    cmd.arg("-l");
    if let Some(u) = user {
        cmd.arg("-u").arg(u);
    }

    let output = cmd.output().context("failed to run crontab")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no crontab for") {
            return Ok(Vec::new());
        }
        anyhow::bail!("crontab -l failed: {}", stderr.trim());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let effective_user = user.map(String::from).unwrap_or_else(whoami);

    let jobs = content
        .lines()
        .filter_map(|line| parse_crontab_line(line, &effective_user, CronSource::UserCrontab))
        .collect();

    Ok(jobs)
}

/// List system cron jobs from /etc/crontab.
pub fn list_system_crontab() -> Result<Vec<CronJob>> {
    let content = match std::fs::read_to_string("/etc/crontab") {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let jobs = content
        .lines()
        .filter_map(|line| parse_system_crontab_line(line, CronSource::SystemCrontab))
        .collect();

    Ok(jobs)
}

/// List cron jobs from /etc/cron.d/.
pub fn list_cron_d() -> Result<Vec<CronJob>> {
    let dir = match std::fs::read_dir("/etc/cron.d") {
        Ok(d) => d,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut jobs = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();

        // Skip dotfiles and backup files
        if filename.starts_with('.') || filename.ends_with('~') || filename.contains(".dpkg-") {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some(job) =
                    parse_system_crontab_line(line, CronSource::CronD(filename.clone()))
                {
                    jobs.push(job);
                }
            }
        }
    }

    Ok(jobs)
}

/// List all cron jobs from all sources.
pub fn list_all_jobs(user: Option<&str>, all_users: bool) -> Result<Vec<CronJob>> {
    let mut jobs = Vec::new();

    if all_users {
        // Try to read all users' crontabs (requires root)
        if let Ok(entries) = std::fs::read_dir("/var/spool/cron/crontabs")
            .or_else(|_| std::fs::read_dir("/var/spool/cron"))
        {
            for entry in entries.flatten() {
                let username = entry.file_name().to_string_lossy().to_string();
                if let Ok(user_jobs) = list_user_crontab(Some(&username)) {
                    jobs.extend(user_jobs);
                }
            }
        } else {
            // Fallback: just list current user
            jobs.extend(list_user_crontab(None)?);
        }
    } else {
        jobs.extend(list_user_crontab(user)?);
    }

    // Always include system crontabs
    jobs.extend(list_system_crontab()?);
    jobs.extend(list_cron_d()?);

    Ok(jobs)
}

/// Get the current username.
fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Explain a cron schedule in plain English.
pub fn explain_schedule(schedule: &str) -> String {
    let parts: Vec<&str> = schedule.split_whitespace().collect();
    if parts.len() != 5 {
        return "invalid cron expression".to_string();
    }

    let (minute, hour, dom, month, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    // Handle common patterns first
    match (minute, hour, dom, month, dow) {
        ("*", "*", "*", "*", "*") => return "every minute".to_string(),
        ("0", "*", "*", "*", "*") => return "every hour".to_string(),
        ("0", "0", "*", "*", "*") => return "daily at midnight".to_string(),
        ("0", "0", "1", "*", "*") => return "monthly on the 1st at midnight".to_string(),
        ("0", "0", "1", "1", "*") => return "yearly on Jan 1st at midnight".to_string(),
        ("0", "0", "*", "*", "0") => return "weekly on Sunday at midnight".to_string(),
        ("0", "0", "*", "*", "1") => return "weekly on Monday at midnight".to_string(),
        _ => {}
    }

    let mut parts_desc = Vec::new();

    // Minute
    match minute {
        "*" => {}
        m if m.starts_with("*/") => {
            let interval = &m[2..];
            parts_desc.push(format!("every {interval} minutes"));
        }
        m => {
            parts_desc.push(format!("at minute {m}"));
        }
    }

    // Hour
    match hour {
        "*" => {
            if !minute.starts_with("*/") && minute != "*" {
                parts_desc.push("every hour".to_string());
            }
        }
        h if h.starts_with("*/") => {
            let interval = &h[2..];
            parts_desc.push(format!("every {interval} hours"));
        }
        h => {
            // Replace "at minute X" with "at HH:MM" for readability
            if let Ok(hour_num) = h.parse::<u32>() {
                if let Some(min_desc) = parts_desc.first() {
                    if let Some(min_str) = min_desc.strip_prefix("at minute ") {
                        if let Ok(min_num) = min_str.parse::<u32>() {
                            parts_desc.clear();
                            parts_desc.push(format!("at {:02}:{:02}", hour_num, min_num));
                        } else {
                            parts_desc.push(format!("at hour {h}"));
                        }
                    } else {
                        parts_desc.push(format!("at hour {h}"));
                    }
                } else {
                    // minute was *, so it runs every minute during this hour
                    parts_desc.push(format!("during hour {h}"));
                }
            } else {
                parts_desc.push(format!("at hour {h}"));
            }
        }
    }

    // Day of month
    match dom {
        "*" => {}
        d if d.starts_with("*/") => {
            let interval = &d[2..];
            parts_desc.push(format!("every {interval} days"));
        }
        d if d.contains(',') => {
            parts_desc.push(format!("on days {d}"));
        }
        d => {
            parts_desc.push(format!("on day {d}"));
        }
    }

    // Month
    match month {
        "*" => {}
        m if m.contains(',') => {
            parts_desc.push(format!("in months {}", explain_months(m)));
        }
        m => {
            if let Some(name) = month_name(m) {
                parts_desc.push(format!("in {name}"));
            } else {
                parts_desc.push(format!("in month {m}"));
            }
        }
    }

    // Day of week
    match dow {
        "*" => {}
        d if d.contains(',') => {
            let days: Vec<&str> = d.split(',').collect();
            let day_names: Vec<String> = days
                .iter()
                .map(|d| dow_name(d).unwrap_or_else(|| d.to_string()))
                .collect();
            parts_desc.push(format!("on {}", day_names.join(", ")));
        }
        d if d.contains('-') => {
            let range: Vec<&str> = d.split('-').collect();
            if range.len() == 2 {
                let start = dow_name(range[0]).unwrap_or_else(|| range[0].to_string());
                let end = dow_name(range[1]).unwrap_or_else(|| range[1].to_string());
                parts_desc.push(format!("on {start} through {end}"));
            }
        }
        d => {
            if let Some(name) = dow_name(d) {
                parts_desc.push(format!("on {name}"));
            } else {
                parts_desc.push(format!("on weekday {d}"));
            }
        }
    }

    if parts_desc.is_empty() {
        "every minute".to_string()
    } else {
        parts_desc.join(", ")
    }
}

fn month_name(m: &str) -> Option<String> {
    match m {
        "1" | "jan" | "JAN" => Some("January".to_string()),
        "2" | "feb" | "FEB" => Some("February".to_string()),
        "3" | "mar" | "MAR" => Some("March".to_string()),
        "4" | "apr" | "APR" => Some("April".to_string()),
        "5" | "may" | "MAY" => Some("May".to_string()),
        "6" | "jun" | "JUN" => Some("June".to_string()),
        "7" | "jul" | "JUL" => Some("July".to_string()),
        "8" | "aug" | "AUG" => Some("August".to_string()),
        "9" | "sep" | "SEP" => Some("September".to_string()),
        "10" | "oct" | "OCT" => Some("October".to_string()),
        "11" | "nov" | "NOV" => Some("November".to_string()),
        "12" | "dec" | "DEC" => Some("December".to_string()),
        _ => None,
    }
}

fn explain_months(m: &str) -> String {
    m.split(',')
        .map(|p| month_name(p).unwrap_or_else(|| p.to_string()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn dow_name(d: &str) -> Option<String> {
    match d {
        "0" | "7" | "sun" | "SUN" => Some("Sunday".to_string()),
        "1" | "mon" | "MON" => Some("Monday".to_string()),
        "2" | "tue" | "TUE" => Some("Tuesday".to_string()),
        "3" | "wed" | "WED" => Some("Wednesday".to_string()),
        "4" | "thu" | "THU" => Some("Thursday".to_string()),
        "5" | "fri" | "FRI" => Some("Friday".to_string()),
        "6" | "sat" | "SAT" => Some("Saturday".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_every_minute() {
        assert_eq!(explain_schedule("* * * * *"), "every minute");
    }

    #[test]
    fn explain_every_hour() {
        assert_eq!(explain_schedule("0 * * * *"), "every hour");
    }

    #[test]
    fn explain_daily_midnight() {
        assert_eq!(explain_schedule("0 0 * * *"), "daily at midnight");
    }

    #[test]
    fn explain_every_5_minutes() {
        assert_eq!(explain_schedule("*/5 * * * *"), "every 5 minutes");
    }

    #[test]
    fn explain_specific_time() {
        assert_eq!(explain_schedule("30 2 * * *"), "at 02:30");
    }

    #[test]
    fn explain_weekly_monday() {
        assert_eq!(
            explain_schedule("0 0 * * 1"),
            "weekly on Monday at midnight"
        );
    }

    #[test]
    fn explain_monthly() {
        assert_eq!(
            explain_schedule("0 0 1 * *"),
            "monthly on the 1st at midnight"
        );
    }

    #[test]
    fn explain_specific_day_and_time() {
        assert_eq!(
            explain_schedule("0 3 * * 1,3,5"),
            "at 03:00, on Monday, Wednesday, Friday"
        );
    }

    #[test]
    fn explain_complex() {
        assert_eq!(explain_schedule("30 4 1,15 * *"), "at 04:30, on days 1,15");
    }

    #[test]
    fn explain_yearly() {
        assert_eq!(
            explain_schedule("0 0 1 1 *"),
            "yearly on Jan 1st at midnight"
        );
    }

    #[test]
    fn explain_weekday_range() {
        assert_eq!(
            explain_schedule("0 9 * * 1-5"),
            "at 09:00, on Monday through Friday"
        );
    }

    #[test]
    fn explain_with_month() {
        assert_eq!(explain_schedule("0 6 * 3 *"), "at 06:00, in March");
    }

    #[test]
    fn parse_crontab_line_valid() {
        let job = parse_crontab_line(
            "*/5 * * * * /usr/local/bin/backup.sh",
            "ruben",
            CronSource::UserCrontab,
        );
        assert!(job.is_some());
        let job = job.unwrap();
        assert_eq!(job.schedule, "*/5 * * * *");
        assert_eq!(job.command, "/usr/local/bin/backup.sh");
        assert_eq!(job.user, "ruben");
        assert_eq!(job.description, "every 5 minutes");
    }

    #[test]
    fn parse_crontab_line_skips_comments() {
        assert!(
            parse_crontab_line("# this is a comment", "ruben", CronSource::UserCrontab).is_none()
        );
    }

    #[test]
    fn parse_crontab_line_skips_env_vars() {
        assert!(parse_crontab_line("SHELL=/bin/bash", "ruben", CronSource::UserCrontab).is_none());
    }

    #[test]
    fn parse_crontab_line_skips_empty() {
        assert!(parse_crontab_line("", "ruben", CronSource::UserCrontab).is_none());
    }

    #[test]
    fn parse_system_line_valid() {
        let job = parse_system_crontab_line(
            "0 3 * * * root /usr/sbin/logrotate",
            CronSource::SystemCrontab,
        );
        assert!(job.is_some());
        let job = job.unwrap();
        assert_eq!(job.user, "root");
        assert_eq!(job.command, "/usr/sbin/logrotate");
        assert_eq!(job.schedule, "0 3 * * *");
    }
}
