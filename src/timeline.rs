use chrono::{Datelike, Local, TimeDelta};
use serde::Serialize;

use crate::cron::CronJob;
use crate::util::truncate;

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub time: String,
    pub user: String,
    pub command: String,
    pub schedule: String,
}

#[derive(Debug, Serialize)]
pub struct Timeline {
    pub start: String,
    pub end: String,
    pub hours: u32,
    pub events: Vec<TimelineEvent>,
}

/// Build timeline by iterating only over matching (hour, minute) pairs per job,
/// then checking date constraints. Much faster than walking every minute.
pub fn build_timeline(jobs: &[CronJob], hours: u32) -> Timeline {
    let now = Local::now();
    let end = now + TimeDelta::hours(i64::from(hours));

    let mut events = Vec::new();

    for job in jobs {
        let fields: Vec<&str> = job.schedule.split_whitespace().collect();
        if fields.len() != 5 {
            continue;
        }

        let minutes = expand_field(fields[0], 0, 59);
        let hours_f = expand_field(fields[1], 0, 23);
        let doms = expand_field(fields[2], 1, 31);
        let months = expand_field(fields[3], 1, 12);
        let dows = expand_dow_field(fields[4]);

        // Walk day-by-day, then only check matching (hour, minute) pairs
        let mut day = now.date_naive();
        let end_date = end.date_naive();

        while day <= end_date {
            let month = day.month();
            let dom = day.day();
            let dow = day.weekday().num_days_from_sunday();

            if !months.contains(&month) || !doms.contains(&dom) || !dows.contains(&dow) {
                day += TimeDelta::days(1);
                continue;
            }

            for &h in &hours_f {
                for &m in &minutes {
                    let Some(naive_time) = chrono::NaiveTime::from_hms_opt(h, m, 0) else {
                        continue;
                    };
                    let naive_dt = day.and_time(naive_time);
                    let Some(local_dt) = naive_dt.and_local_timezone(now.timezone()).single()
                    else {
                        continue;
                    };

                    if local_dt >= now && local_dt < end {
                        events.push(TimelineEvent {
                            time: local_dt.format("%Y-%m-%dT%H:%M").to_string(),
                            user: job.user.clone(),
                            command: job.command.clone(),
                            schedule: job.schedule.clone(),
                        });
                    }
                }
            }

            day += TimeDelta::days(1);
        }
    }

    events.sort_by(|a, b| a.time.cmp(&b.time));

    Timeline {
        start: now.format("%Y-%m-%dT%H:%M").to_string(),
        end: end.format("%Y-%m-%dT%H:%M").to_string(),
        hours,
        events,
    }
}

pub fn print_timeline(timeline: &Timeline) {
    use owo_colors::OwoColorize;
    use std::io::IsTerminal;

    let color = std::io::stdout().is_terminal();

    if timeline.events.is_empty() {
        println!("No scheduled jobs in the next {} hours.", timeline.hours);
        return;
    }

    if color {
        println!(
            "{}",
            format!(
                " Timeline: {} to {} ({} hours)",
                timeline.start, timeline.end, timeline.hours
            )
            .bold()
        );
        println!("{}", "\u{2500}".repeat(80).dimmed());
    } else {
        println!(
            " Timeline: {} to {} ({} hours)",
            timeline.start, timeline.end, timeline.hours
        );
        println!("{}", "-".repeat(80));
    }

    let mut last_hour = String::new();
    for event in &timeline.events {
        let hour_str = &event.time[..13]; // YYYY-MM-DDTHH
        if hour_str != last_hour {
            if !last_hour.is_empty() {
                println!();
            }
            let display_time = hour_str.replace('T', " ");
            if color {
                println!(" {}", format!("{display_time}:00").cyan());
            } else {
                println!(" {display_time}:00");
            }
            last_hour = hour_str.to_string();
        }

        let time_part = &event.time[11..16]; // HH:MM
        let cmd_short = truncate(&event.command, 60);
        if color {
            println!(
                "   {} {} {}",
                time_part.green(),
                event.user.dimmed(),
                cmd_short
            );
        } else {
            println!("   {} {} {}", time_part, event.user, cmd_short);
        }
    }

    println!();
    if color {
        println!(
            "{}",
            format!(" {} event(s) scheduled", timeline.events.len()).dimmed()
        );
    }
}

/// Expand a cron field into a set of matching values.
fn expand_field(field: &str, min: u32, max: u32) -> Vec<u32> {
    let mut values = Vec::new();

    for part in field.split(',') {
        if part == "*" {
            return (min..=max).collect();
        } else if let Some(step) = part.strip_prefix("*/") {
            if let Ok(s) = step.parse::<u32>() {
                let mut v = min;
                while v <= max {
                    values.push(v);
                    v += s;
                }
            }
        } else if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2
                && let (Ok(start), Ok(end)) = (range[0].parse::<u32>(), range[1].parse::<u32>())
            {
                for v in start..=end {
                    values.push(v);
                }
            }
        } else if let Ok(v) = part.parse::<u32>() {
            values.push(v);
        } else if let Some(v) = parse_month_name(part) {
            values.push(v);
        }
    }

    values
}

/// Expand day-of-week field, handling 0 and 7 both as Sunday.
fn expand_dow_field(field: &str) -> Vec<u32> {
    let mut values = expand_field_dow_inner(field);
    if values.contains(&7) && !values.contains(&0) {
        values.push(0);
    }
    values
}

fn expand_field_dow_inner(field: &str) -> Vec<u32> {
    let mut values = Vec::new();

    for part in field.split(',') {
        if part == "*" {
            return (0..=6).collect();
        } else if let Some(step) = part.strip_prefix("*/") {
            if let Ok(s) = step.parse::<u32>() {
                let mut v = 0;
                while v <= 6 {
                    values.push(v);
                    v += s;
                }
            }
        } else if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                let start = parse_dow_value(range[0]).unwrap_or(0);
                let end = parse_dow_value(range[1]).unwrap_or(6);
                for v in start..=end {
                    values.push(v);
                }
            }
        } else if let Some(v) = parse_dow_value(part) {
            values.push(v);
        }
    }

    values
}

fn parse_dow_value(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "sun" => Some(0),
        "mon" => Some(1),
        "tue" => Some(2),
        "wed" => Some(3),
        "thu" => Some(4),
        "fri" => Some(5),
        "sat" => Some(6),
        _ => s.parse().ok(),
    }
}

fn parse_month_name(s: &str) -> Option<u32> {
    match s.to_lowercase().as_str() {
        "jan" => Some(1),
        "feb" => Some(2),
        "mar" => Some(3),
        "apr" => Some(4),
        "may" => Some(5),
        "jun" => Some(6),
        "jul" => Some(7),
        "aug" => Some(8),
        "sep" => Some(9),
        "oct" => Some(10),
        "nov" => Some(11),
        "dec" => Some(12),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::CronSource;

    fn make_job(schedule: &str, command: &str) -> CronJob {
        CronJob {
            user: "test".to_string(),
            schedule: schedule.to_string(),
            command: command.to_string(),
            description: String::new(),
            source: CronSource::UserCrontab,
        }
    }

    #[test]
    fn expand_star() {
        assert_eq!(expand_field("*", 0, 59).len(), 60);
    }

    #[test]
    fn expand_step() {
        assert_eq!(expand_field("*/15", 0, 59), vec![0, 15, 30, 45]);
    }

    #[test]
    fn expand_range() {
        assert_eq!(expand_field("1-5", 0, 6), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn expand_list() {
        assert_eq!(expand_field("1,3,5", 0, 6), vec![1, 3, 5]);
    }

    #[test]
    fn expand_dow_sunday() {
        let values = expand_dow_field("0");
        assert!(values.contains(&0));
        let values = expand_dow_field("7");
        assert!(values.contains(&0));
    }

    #[test]
    fn expand_combined_list_and_range() {
        assert_eq!(
            expand_field("1-3,7,10-12", 0, 12),
            vec![1, 2, 3, 7, 10, 11, 12]
        );
    }

    #[test]
    fn expand_step_from_range() {
        // */10 on minutes should give 0,10,20,30,40,50
        assert_eq!(expand_field("*/10", 0, 59), vec![0, 10, 20, 30, 40, 50]);
    }

    #[test]
    fn expand_month_names() {
        assert_eq!(expand_field("jan", 1, 12), vec![1]);
        assert_eq!(expand_field("dec", 1, 12), vec![12]);
    }

    #[test]
    fn expand_dow_names() {
        let values = expand_dow_field("mon");
        assert_eq!(values, vec![1]);
        let values = expand_dow_field("fri");
        assert_eq!(values, vec![5]);
    }

    #[test]
    fn expand_dow_range_names() {
        let values = expand_dow_field("mon-fri");
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn timeline_has_events() {
        let jobs = vec![make_job("* * * * *", "echo hello")];
        let timeline = build_timeline(&jobs, 1);
        assert!(!timeline.events.is_empty());
    }

    #[test]
    fn timeline_empty_for_no_jobs() {
        let timeline = build_timeline(&[], 24);
        assert!(timeline.events.is_empty());
    }

    #[test]
    fn timeline_events_are_sorted() {
        let jobs = vec![
            make_job("0 * * * *", "hourly_job"),
            make_job("30 * * * *", "half_hour_job"),
        ];
        let timeline = build_timeline(&jobs, 3);
        for window in timeline.events.windows(2) {
            assert!(window[0].time <= window[1].time);
        }
    }

    #[test]
    fn timeline_respects_hours_bound() {
        let jobs = vec![make_job("0 * * * *", "hourly")];
        let timeline = build_timeline(&jobs, 2);
        // Should have at most 2 events (one per hour)
        assert!(timeline.events.len() <= 2);
    }

    #[test]
    fn expand_empty_field_returns_empty() {
        assert!(expand_field("", 0, 59).is_empty());
    }

    #[test]
    fn expand_invalid_field_returns_empty() {
        assert!(expand_field("abc", 0, 59).is_empty());
    }
}
