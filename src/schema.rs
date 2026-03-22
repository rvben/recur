use serde_json::{Value, json};

pub fn build_schema() -> Value {
    json!({
        "name": "ogni",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "A human-friendly cron job manager",
        "global_flags": {
            "--json / -j": "Output as JSON (auto-enabled when stdout is not a terminal)",
            "--quiet / -q": "Suppress output, rely on exit code only",
            "--fields": "Filter JSON output to specific fields (comma-separated dot-paths)",
        },
        "exit_codes": {
            "0": "success (or no issues found for check)",
            "1": "runtime error",
            "2": "issues found (check command only)",
        },
        "commands": {
            "list": {
                "description": "List all cron jobs with human-readable schedules",
                "args": {
                    "--user / -u": { "type": "string", "description": "Show jobs for a specific user (requires root for other users)" },
                    "--all / -a": { "type": "boolean", "description": "Show all users' cron jobs (requires root)" },
                },
                "output_schema": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "fields": {
                            "user": "string — username that owns the job",
                            "schedule": "string — raw cron expression (5 fields)",
                            "command": "string — the command to execute",
                            "description": "string — human-readable schedule description",
                            "source": "string — where the job is defined (UserCrontab, SystemCrontab, CronD)",
                        }
                    }
                },
                "examples": [
                    "ogni list",
                    "ogni list --user root",
                    "ogni list -a --json",
                    "ogni list --json --fields user,schedule,command",
                ],
            },
            "explain": {
                "description": "Explain a cron expression in plain English",
                "args": {
                    "expression": { "type": "string", "positional": true, "required": true, "description": "Cron expression (5 space-separated fields)" },
                },
                "output_schema": {
                    "type": "object",
                    "fields": {
                        "expression": "string — the input cron expression",
                        "description": "string — plain English explanation",
                    }
                },
                "examples": [
                    "ogni explain '*/5 * * * *'",
                    "ogni explain '0 3 * * 1-5'",
                    "ogni explain '0 0 1 1 *' --json",
                ],
            },
            "check": {
                "description": "Check cron jobs for issues (missing scripts, permission problems, overlapping schedules)",
                "args": {
                    "--user / -u": { "type": "string", "description": "Check jobs for a specific user" },
                    "--all / -a": { "type": "boolean", "description": "Check all users' cron jobs (requires root)" },
                    "--dry-run": { "type": "boolean", "description": "Preview what would be checked without executing" },
                },
                "output_schema": {
                    "type": "object",
                    "fields": {
                        "jobs_checked": "integer — number of jobs inspected",
                        "issues": {
                            "type": "array",
                            "items": {
                                "severity": "string — Error or Warning",
                                "job_command": "string — the problematic command",
                                "user": "string — owner of the job",
                                "message": "string — description of the issue",
                            }
                        }
                    }
                },
                "exit_codes": {
                    "0": "no issues found",
                    "1": "runtime error",
                    "2": "issues detected",
                },
                "examples": [
                    "ogni check",
                    "ogni check --quiet  # exit code 0=clean, 2=issues",
                    "ogni check --dry-run --json",
                    "ogni check -a --json --fields issues",
                ],
            },
            "timeline": {
                "description": "Show a visual timeline of when jobs run in the next N hours",
                "args": {
                    "--hours": { "type": "integer", "default": 24, "description": "Number of hours to show" },
                    "--user / -u": { "type": "string", "description": "Show jobs for a specific user" },
                    "--all / -a": { "type": "boolean", "description": "Show all users' cron jobs (requires root)" },
                },
                "output_schema": {
                    "type": "object",
                    "fields": {
                        "start": "string — ISO 8601 timestamp",
                        "end": "string — ISO 8601 timestamp",
                        "hours": "integer — span in hours",
                        "events": {
                            "type": "array",
                            "items": {
                                "time": "string — ISO 8601 timestamp",
                                "user": "string",
                                "command": "string",
                                "schedule": "string",
                            }
                        }
                    }
                },
                "examples": [
                    "ogni timeline",
                    "ogni timeline --hours 48",
                    "ogni timeline --json --fields events",
                ],
            },
            "schema": {
                "description": "Output full command schema as JSON (for AI agents and tooling)",
                "args": {},
                "examples": ["ogni schema"],
            },
            "completions": {
                "description": "Generate shell completions",
                "args": {
                    "shell": { "type": "string", "positional": true, "required": true, "description": "Shell to generate for (bash, zsh, fish, elvish, powershell)" },
                },
                "examples": [
                    "ogni completions bash",
                    "ogni completions zsh > ~/.zfunc/_ogni",
                    "ogni completions fish > ~/.config/fish/completions/ogni.fish",
                ],
            },
        },
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
