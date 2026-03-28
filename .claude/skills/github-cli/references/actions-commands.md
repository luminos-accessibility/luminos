# gh run & gh workflow — GitHub Actions Commands

## Table of Contents
- [run list](#run-list)
- [run view](#run-view)
- [run watch](#run-watch)
- [run rerun](#run-rerun)
- [run cancel](#run-cancel)
- [run download](#run-download)
- [run delete](#run-delete)
- [workflow list](#workflow-list)
- [workflow view](#workflow-view)
- [workflow run](#workflow-run)
- [workflow enable / disable](#workflow-enable--disable)
- [cache list / delete](#cache-list--delete)

All commands accept `-R owner/repo`.

---

## run list

List recent workflow runs. Aliases: `gh run ls`.

```
gh run list [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--workflow` | `-w` | Filter by workflow name or filename |
| `--branch` | `-b` | Filter by branch |
| `--user` | `-u` | Filter by triggering user |
| `--event` | `-e` | Filter by event (push, pull_request, etc.) |
| `--status` | `-s` | Filter: queued, completed, in_progress, success, failure, cancelled, etc. |
| `--commit` | `-c` | Filter by commit SHA |
| `--created` | | Filter by creation date |
| `--limit` | `-L` | Max results (default 20) |
| `--all` | `-a` | Include disabled workflows |
| `--json` | | JSON output |
| `--jq` / `--template` | | Output formatting |

**JSON fields:** attempt, conclusion, createdAt, databaseId, displayTitle, event, headBranch, headSha, name, number, startedAt, status, updatedAt, url, workflowDatabaseId, workflowName

**Examples:**
```bash
gh run list --limit 5
gh run list --workflow ci.yml --branch main
gh run list --status failure --limit 10
gh run list --json databaseId,conclusion,displayTitle --jq '.[] | select(.conclusion == "failure")'
```

---

## run view

View workflow run details.

```
gh run view [<run-id>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--job` | `-j` | View specific job by ID |
| `--log` | | View full log |
| `--log-failed` | | View log for failed steps only |
| `--verbose` | `-v` | Show job steps |
| `--exit-status` | | Exit non-zero if run failed |
| `--attempt` | `-a` | Specific attempt number |
| `--json` | | JSON output |
| `--web` | `-w` | Open in browser |

**JSON fields:** attempt, conclusion, createdAt, databaseId, displayTitle, event, headBranch, headSha, jobs, name, number, startedAt, status, updatedAt, url, workflowDatabaseId, workflowName

**Examples:**
```bash
gh run view 12345
gh run view 12345 --log-failed
gh run view --job 456789 --log
gh run view 12345 --attempt 3
gh run view 12345 --exit-status && echo "passed"

# Get job IDs for rerun
gh run view 12345 --json jobs --jq '.jobs[] | {name, databaseId}'
```

---

## run watch

Watch a run until it completes with live progress.

```
gh run watch <run-id> [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--compact` | Show only relevant/failed steps |
| `--exit-status` | Exit non-zero if run fails |
| `--interval` (`-i`) | Refresh interval in seconds (default 3) |

**Examples:**
```bash
gh run watch                              # Interactive selection
gh run watch --compact
gh run watch 12345 && notify-send 'done!' # Notify on completion
```

Note: Does not support fine-grained PATs (requires `checks:read`).

---

## run rerun

Rerun an entire run, only failed jobs, or a specific job.

```
gh run rerun [<run-id>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--failed` | | Rerun only failed jobs + dependencies |
| `--job` | `-j` | Rerun specific job by database ID |
| `--debug` | `-d` | Enable debug logging |

**Important:** The `--job` flag requires the `databaseId`, NOT the number from the URL. Get it with:
```bash
gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId}'
```

---

## run cancel

```bash
gh run cancel [<run-id>] [--force]
```

---

## run download

Download artifacts from a workflow run.

```
gh run download [<run-id>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--name` | `-n` | Download by artifact name (repeatable) |
| `--pattern` | `-p` | Download by glob pattern (repeatable) |
| `--dir` | `-D` | Target directory (default ".") |

**Examples:**
```bash
gh run download <run-id>                         # All artifacts
gh run download <run-id> -n coverage-report      # By name
gh run download -n artifact1 -n artifact2        # Multiple (latest run)
gh run download <run-id> -p '*.zip' -D ./output  # By pattern
```

---

## run delete

```bash
gh run delete [<run-id>]
```

---

## workflow list

List workflows. Aliases: `gh workflow ls`.

```
gh workflow list [flags]
```

**Flags:** `--all` (`-a`, include disabled), `--limit` (`-L`, default 50), `--json`, `--jq`, `--template`

**JSON fields:** id, name, path, state

---

## workflow view

```bash
gh workflow view [<workflow-id> | <workflow-name> | <filename>] [--web] [--json] [--ref <branch>]
```

---

## workflow run

Trigger a `workflow_dispatch` event.

```
gh workflow run [<workflow-id> | <workflow-name>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--raw-field` | `-f` | String input: `key=value` |
| `--field` | `-F` | Typed input: `key=value` (with `@file` support) |
| `--json` | | Read inputs as JSON from stdin |
| `--ref` | `-r` | Branch/tag with the workflow version |

**Examples:**
```bash
gh workflow run deploy.yml
gh workflow run deploy.yml -f environment=staging -f debug=true
gh workflow run deploy.yml --ref my-branch
echo '{"environment":"prod"}' | gh workflow run deploy.yml --json
```

---

## workflow enable / disable

```bash
gh workflow enable <workflow-id>
gh workflow disable <workflow-id>
```

---

## cache list / delete

Manage GitHub Actions caches.

```bash
gh cache list [-R owner/repo]
gh cache delete <cache-id> [-R owner/repo]
gh cache delete --all [-R owner/repo]
```
