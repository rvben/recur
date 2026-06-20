use serde_json::{Value, json};

pub fn build_schema() -> Value {
    json!({
        "clispec": "0.2",
        "name": "recur",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "A human-friendly cron job manager",
        "global_args": [
            {"name": "--json", "type": "boolean", "required": false, "description": "Output as JSON (-j); auto-enabled when stdout is not a terminal"},
            {"name": "--quiet", "type": "boolean", "required": false, "description": "Suppress output, rely on the exit code only (-q)"},
            {"name": "--fields", "type": "string", "required": false, "description": "Filter JSON output to specific comma-separated dot-paths"}
        ],
        "commands": [
            {
                "name": "list",
                "description": "List all cron jobs with human-readable schedules",
                "mutating": false,
                "args": [
                    {"name": "--user", "type": "string", "required": false, "description": "Show jobs for a specific user, -u (requires root for other users)"},
                    {"name": "--all", "type": "boolean", "required": false, "description": "Show all users' cron jobs, -a (requires root)"}
                ],
                "output_fields": [
                    {"name": "user", "type": "string", "description": "Username that owns the job"},
                    {"name": "schedule", "type": "string", "description": "Raw cron expression (5 fields)"},
                    {"name": "command", "type": "string", "description": "The command to execute"},
                    {"name": "description", "type": "string", "description": "Human-readable schedule description"},
                    {"name": "source", "type": "string", "description": "Where the job is defined (UserCrontab, SystemCrontab, CronD)"}
                ]
            },
            {
                "name": "explain",
                "description": "Explain a cron expression in plain English",
                "mutating": false,
                "args": [
                    {"name": "expression", "type": "string", "required": true, "description": "Cron expression (5 space-separated fields)"}
                ],
                "output_fields": [
                    {"name": "expression", "type": "string", "description": "The input cron expression"},
                    {"name": "description", "type": "string", "description": "Plain English explanation"}
                ]
            },
            {
                "name": "check",
                "description": "Check cron jobs for issues (missing scripts, permission problems, overlapping schedules)",
                "mutating": false,
                "args": [
                    {"name": "--user", "type": "string", "required": false, "description": "Check jobs for a specific user (-u)"},
                    {"name": "--all", "type": "boolean", "required": false, "description": "Check all users' cron jobs, -a (requires root)"},
                    {"name": "--dry-run", "type": "boolean", "required": false, "description": "Preview what would be checked without executing"}
                ],
                "output_fields": [
                    {"name": "jobs_checked", "type": "integer", "description": "Number of jobs inspected"},
                    {"name": "issues", "type": "array", "description": "Issues found; each has severity (Error|Warning), job_command, user, message"}
                ],
                "notes": "Exits 2 when issues are found and 0 when clean, so it works as a pre-flight gate (pair with --quiet)."
            },
            {
                "name": "timeline",
                "description": "Show a visual timeline of when jobs run in the next N hours",
                "mutating": false,
                "args": [
                    {"name": "--hours", "type": "integer", "required": false, "default": 24, "description": "Number of hours to show"},
                    {"name": "--user", "type": "string", "required": false, "description": "Show jobs for a specific user (-u)"},
                    {"name": "--all", "type": "boolean", "required": false, "description": "Show all users' cron jobs, -a (requires root)"}
                ],
                "output_fields": [
                    {"name": "start", "type": "string", "description": "ISO 8601 timestamp of the window start"},
                    {"name": "end", "type": "string", "description": "ISO 8601 timestamp of the window end"},
                    {"name": "hours", "type": "integer", "description": "Span in hours"},
                    {"name": "events", "type": "array", "description": "Scheduled runs; each has time (ISO 8601), user, command, schedule"}
                ]
            },
            {
                "name": "schema",
                "description": "Output this machine-readable clispec contract as JSON",
                "mutating": false,
                "args": [],
                "output_fields": []
            },
            {
                "name": "completions",
                "description": "Generate shell completions",
                "mutating": false,
                "args": [
                    {"name": "shell", "type": "string", "required": true, "description": "Shell to generate for (bash, zsh, fish, elvish, powershell)"}
                ],
                "output_fields": []
            }
        ],
        "outcomes": [
            {"kind": "issues_found", "exit_code": 2, "retryable": false, "description": "check found one or more issues (missing scripts, permission problems)"}
        ],
        "errors": [
            {"kind": "runtime", "exit_code": 1, "retryable": false, "message": "Runtime error, e.g. an unreadable crontab or invalid cron expression", "hint": "Run recur --help"}
        ],
        "cron_reference": {
            "format": "minute hour day_of_month month day_of_week",
            "fields": {
                "minute": "0-59",
                "hour": "0-23",
                "day_of_month": "1-31",
                "month": "1-12 or jan-dec",
                "day_of_week": "0-7 (0 and 7 are Sunday) or sun-sat",
            },
            "special_characters": {
                "*": "any value",
                ",": "value list separator (e.g. 1,3,5)",
                "-": "range (e.g. 1-5)",
                "/": "step values (e.g. */5 means every 5)",
            },
            "common_patterns": {
                "* * * * *": "every minute",
                "0 * * * *": "every hour",
                "0 0 * * *": "daily at midnight",
                "*/5 * * * *": "every 5 minutes",
                "0 0 * * 0": "weekly on Sunday",
                "0 0 1 * *": "monthly on the 1st",
                "0 0 1 1 *": "yearly on Jan 1st",
            },
        },
    })
}
