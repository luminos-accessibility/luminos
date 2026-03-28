---
name: github-cli
description: Comprehensive reference for using the GitHub CLI (`gh`) tool to manage repositories, PRs, issues, releases, Actions, and the GitHub API. Use this skill whenever you need to interact with GitHub from the command line — creating or reviewing PRs, managing issues, checking CI status, making API calls, searching GitHub, managing releases, or any `gh` command usage. Also use when the user mentions GitHub operations, wants to automate GitHub workflows, or asks about gh flags/syntax.
---

# GitHub CLI (`gh`) Reference

The GitHub CLI (`gh` v2.81.0+) lets you work with GitHub entirely from the terminal — PRs, issues, releases, Actions, API calls, and more. It auto-detects the current repo from your git remote, handles authentication, and supports rich JSON output formatting.

## Command Map

Pick the right command for the job:

| Task | Command | Reference |
|------|---------|-----------|
| Pull requests | `gh pr` | [pr-commands.md](references/pr-commands.md) |
| Issues | `gh issue` | [issue-commands.md](references/issue-commands.md) |
| Repositories | `gh repo` | [repo-commands.md](references/repo-commands.md) |
| Raw API calls | `gh api` | [api-command.md](references/api-command.md) |
| CI/CD & Actions | `gh run`, `gh workflow` | [actions-commands.md](references/actions-commands.md) |
| Releases | `gh release` | [release-commands.md](references/release-commands.md) |
| Search across GitHub | `gh search` | [search-commands.md](references/search-commands.md) |
| Auth & config | `gh auth`, `gh config` | [auth-and-config.md](references/auth-and-config.md) |
| Output formatting | `--json`, `--jq`, `--template` | [formatting.md](references/formatting.md) |
| Other (gist, label, secret, variable, etc.) | Various | [other-commands.md](references/other-commands.md) |

## Key Concepts

### Repository Context

Most commands auto-detect the repo from the current git directory. Override with:
- `-R owner/repo` flag on any command
- `GH_REPO=owner/repo` environment variable
- `gh repo set-default owner/repo` to set permanently

### Non-Interactive Mode

For scripting and automation, always supply required arguments via flags rather than relying on interactive prompts. Set `GH_PROMPT_DISABLED=1` to ensure no prompts hang your scripts.

### Argument Formats

PRs and issues accept multiple identifier formats:
- By number: `123`
- By URL: `https://github.com/owner/repo/pull/123`
- PRs also by branch name: `feature-branch` or `owner:feature-branch`

## Common Workflows

### PR Lifecycle

```bash
# Create a PR with title and body
gh pr create --title "feat: add feature" --body "Description here"

# Create PR using commit messages as title/body
gh pr create --fill

# Create PR from HEREDOC body (best for multi-line)
gh pr create --title "the title" --body "$(cat <<'EOF'
## Summary
- Change 1
- Change 2

## Test plan
- [ ] Tested locally
EOF
)"

# Check out someone's PR locally
gh pr checkout 123

# View PR details and CI status
gh pr view 123
gh pr checks 123

# Review and merge
gh pr review 123 --approve
gh pr merge 123 --squash --delete-branch

# View PR comments
gh api repos/{owner}/{repo}/pulls/123/comments
```

### Issue Management

```bash
# Create an issue
gh issue create --title "Bug: X doesn't work" --body "Steps to reproduce..."

# List open issues with label
gh issue list --label bug --assignee @me

# Close with comment
gh issue close 123 --comment "Fixed in #456" --reason completed

# Create branch linked to issue
gh issue develop 123 --checkout
```

### CI/CD Monitoring

```bash
# List recent workflow runs
gh run list --limit 5

# Watch a run until it finishes
gh run watch <run-id>

# View failed job logs
gh run view <run-id> --log-failed

# Re-run only failed jobs
gh run rerun <run-id> --failed

# Trigger a workflow manually
gh workflow run deploy.yml -f environment=staging
```

### Release Management

```bash
# Create release with auto-generated notes
gh release create v1.0.0 --generate-notes

# Create release with assets
gh release create v1.0.0 ./dist/*.tar.gz --title "v1.0.0" --notes "Release notes"

# Download release assets
gh release download v1.0.0 --pattern '*.deb'
```

### Raw API Access

The `gh api` command is extremely powerful for anything not covered by built-in commands. See [api-command.md](references/api-command.md) for full details.

```bash
# GET request (default)
gh api repos/{owner}/{repo}/contributors

# POST with fields
gh api repos/{owner}/{repo}/issues -f title="Bug" -f body="Details"

# GraphQL query
gh api graphql -f query='{ viewer { login } }'

# Filter with jq
gh api repos/{owner}/{repo}/releases --jq '.[0].tag_name'

# Paginate all results
gh api repos/{owner}/{repo}/stargazers --paginate --jq '.[].login'
```

### JSON Output & Filtering

Most list/view commands support structured output:

```bash
# Get PR data as JSON
gh pr list --json number,title,author --jq '.[] | "\(.number) \(.title)"'

# Get specific fields from a PR
gh pr view 123 --json title,body,reviews,mergeable

# Format as table
gh pr list --json number,title,headRefName,updatedAt --template \
  '{{range .}}{{tablerow (printf "#%v" .number) .title .headRefName (timeago .updatedAt)}}{{end}}'
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `GH_TOKEN` / `GITHUB_TOKEN` | Auth token for github.com |
| `GH_ENTERPRISE_TOKEN` | Auth token for GitHub Enterprise |
| `GH_HOST` | Target GitHub hostname |
| `GH_REPO` | Override repository (`OWNER/REPO`) |
| `GH_DEBUG=api` | Log full HTTP requests/responses |
| `GH_PROMPT_DISABLED=1` | Disable interactive prompts |
| `GH_PAGER` / `PAGER` | Terminal pager for output |
| `NO_COLOR` | Disable ANSI color output |
| `GH_FORCE_TTY` | Force terminal output when redirected |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error |
| 2 | Command canceled |
| 4 | Authentication required |
| 8 | Checks pending (from `gh pr checks`) |

## GitHub Actions CI/CD Integration

In GitHub Actions workflows, gh is pre-installed. Authenticate with the built-in token:

```yaml
steps:
  - uses: actions/checkout@v4
  - name: Create PR
    env:
      GH_TOKEN: ${{ github.token }}
    run: gh pr create --fill
```

Note: `GITHUB_TOKEN` in Actions has limited default permissions. Operations targeting other repos or elevated permissions need a PAT.

## Known Gotchas

1. **`gh run rerun --job` needs database IDs, not URL IDs.** The number in `actions/runs/<id>/jobs/<number>` is NOT the job ID. Get correct IDs with:
   ```bash
   gh run view <run-id> --json jobs --jq '.jobs[] | {name, databaseId}'
   ```

2. **`--slurp` cannot combine with `--jq` or `--template`.** Workaround: pipe through external `jq`.

3. **`gh pr create` may still prompt** even with flags. Always provide both `--title` and `--body` (or `--fill`) for non-interactive use.

4. **GraphQL pagination silently returns one page** if the query doesn't define `$endCursor: String` and fetch `pageInfo { hasNextPage endCursor }`.

5. **`gh repo edit --visibility`** requires the `--accept-visibility-change-consequences` flag as a safety mechanism.

6. **`GH_PROMPT_DISABLED=1`** is more reliable than `gh config set prompt disabled` for suppressing all prompts.

## Tips

- **Use `--web` / `-w`** on most commands to open the result in a browser instead.
- **Use `gh api`** for any GitHub operation not covered by built-in commands — it handles auth, pagination, and output formatting.
- **HEREDOC for bodies**: When passing multi-line text to `--body`, use `"$(cat <<'EOF' ... EOF)"` to preserve formatting and avoid shell escaping issues.
- **`@me` shorthand**: Use `--assignee @me` or `--author @me` to reference the authenticated user.
- **Dry run**: `gh pr create --dry-run` prints what would happen without creating the PR.
- **Aliases**: Create shortcuts with `gh alias set` (e.g., `gh alias set pv 'pr view'`).
- **`gh api` vs curl**: Always prefer `gh api` over raw curl — it handles authentication, base URLs, pagination, and JSON formatting automatically.
