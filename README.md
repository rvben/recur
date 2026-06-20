# recur

A human-friendly cron job manager.

`recur` reads your crontab and makes it legible: every schedule is rendered in plain English, every job can be checked for problems before it bites you, and the next runs are laid out on a timeline. It is built for humans at a terminal and for agents in a pipe: human-readable on a TTY, JSON when piped (or with `--json`), stable exit codes, and a `recur schema` contract (clispec v0.2).

## Install

```sh
cargo install recur
```

## Commands

List cron jobs with their schedules translated to plain English:

```sh
recur list                 # your crontab
recur list --user root     # another user's crontab (requires root)
recur list --all           # every user's crontab (requires root)
```

Explain a single cron expression without touching the crontab:

```sh
recur explain "*/5 * * * *"      # every 5 minutes
recur explain "0 3 * * 1-5"      # at 03:00, on Monday through Friday
recur explain "0 0 1 1 *"        # yearly on Jan 1st at midnight
```

Check jobs for issues (missing scripts, unreadable paths, permission problems). Exits `2` when issues are found, so it drops straight into CI or a pre-flight check:

```sh
recur check                # exit 0 = clean, 2 = issues found
recur check --dry-run      # preview what would be checked
recur check --all          # all users (requires root)
```

Show a timeline of upcoming runs:

```sh
recur timeline             # next 24 hours
recur timeline --hours 48
```

## Global flags

- `-j, --json`: force JSON output (auto-enabled when stdout is not a terminal).
- `-q, --quiet`: suppress output and rely on the exit code only.
- `--fields <list>`: filter JSON output to specific comma-separated dot-paths, e.g. `--fields user,schedule,command`.

## Output

Human-readable on a TTY; JSON when piped or with `--json`:

```json
$ recur explain "0 3 * * 1-5" --json
{
  "data": {
    "description": "at 03:00, on Monday through Friday",
    "expression": "0 3 * * 1-5"
  },
  "ok": true
}
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | success (or no issues found for `check`) |
| 1 | runtime error |
| 2 | issues found (`check` only) |

## Agent integration

`recur schema` prints the full machine-readable contract (commands, flags, exit codes, a cron-syntax reference, and examples) following clispec v0.2. It needs no network, auth, or config.
