# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2026-06-20

Initial release: a human-friendly cron job manager.

### Added

- `list` renders crontab schedules in plain English, with `--user` and `--all`.
- `explain` translates any cron expression to plain English.
- `check` flags issues such as missing scripts and permission problems, exiting 2 when any are found.
- `timeline` shows upcoming runs over a configurable window.
- `schema` emits a machine-readable contract for agents; structured output via `--json` and `--fields`.
