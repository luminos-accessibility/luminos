# gh pr — Pull Request Commands

## Table of Contents
- [pr create](#pr-create)
- [pr list](#pr-list)
- [pr view](#pr-view)
- [pr checkout](#pr-checkout)
- [pr checks](#pr-checks)
- [pr merge](#pr-merge)
- [pr review](#pr-review)
- [pr edit](#pr-edit)
- [pr close / reopen](#pr-close--reopen)
- [pr diff](#pr-diff)
- [pr comment](#pr-comment)
- [pr ready](#pr-ready)
- [pr update-branch](#pr-update-branch)
- [pr lock / unlock](#pr-lock--unlock)

All `gh pr` commands accept `-R owner/repo` to target a different repository.

PRs can be specified as: number (`123`), URL, or branch name (`feature-branch` or `owner:feature-branch`). Without an argument, most commands use the PR for the current branch.

---

## pr create

Create a pull request. Aliases: `gh pr new`.

```
gh pr create [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--title` | `-t` | PR title |
| `--body` | `-b` | PR body text |
| `--body-file` | `-F` | Read body from file (`-` for stdin) |
| `--base` | `-B` | Target branch (default: repo default branch) |
| `--head` | `-H` | Source branch (default: current branch) |
| `--draft` | `-d` | Create as draft |
| `--fill` | `-f` | Auto-fill title/body from commits |
| `--fill-first` | | Use first commit for title/body |
| `--fill-verbose` | | Use commit msg+body for description |
| `--assignee` | `-a` | Assign users (`@me` for self) |
| `--reviewer` | `-r` | Request reviewers (users or `org/team`) |
| `--label` | `-l` | Add labels |
| `--milestone` | `-m` | Add to milestone |
| `--project` | `-p` | Add to project |
| `--template` | `-T` | Use PR template file |
| `--web` | `-w` | Open browser to create |
| `--dry-run` | | Print details without creating |
| `--no-maintainer-edit` | | Disable maintainer push access |
| `--editor` | `-e` | Open text editor for title/body |
| `--recover` | | Recover from failed create |

**Examples:**
```bash
gh pr create --title "Fix bug" --body "Closes #123"
gh pr create --fill --draft
gh pr create --reviewer monalisa,myorg/team-name
gh pr create --base develop --head monalisa:feature
gh pr create --template "pull_request_template.md"
```

**Notes:**
- `--fill` uses commit messages; `--title`/`--body` override if both provided.
- Body text `Fixes #123` or `Closes #123` auto-links and closes the issue on merge.
- Adding to projects requires `project` scope: `gh auth refresh -s project`.
- The base branch can be configured per-branch: `git config branch.{name}.gh-merge-base {base}`.

---

## pr list

List pull requests. Default: open PRs only. Aliases: `gh pr ls`.

```
gh pr list [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--state` | `-s` | `open` (default), `closed`, `merged`, `all` |
| `--author` | `-A` | Filter by author (`@me` for self) |
| `--assignee` | `-a` | Filter by assignee |
| `--base` | `-B` | Filter by base branch |
| `--head` | `-H` | Filter by head branch |
| `--label` | `-l` | Filter by label (multiple = AND) |
| `--draft` | `-d` | Filter by draft state |
| `--search` | `-S` | Search with GitHub query syntax |
| `--limit` | `-L` | Max results (default 30) |
| `--json` | | Output JSON with specified fields |
| `--jq` | `-q` | Filter JSON with jq syntax |
| `--template` | `-t` | Format with Go template |
| `--web` | `-w` | Open in browser |
| `--app` | | Filter by GitHub App author |

**JSON fields:** additions, assignees, author, autoMergeRequest, baseRefName, baseRefOid, body, changedFiles, closed, closedAt, closingIssuesReferences, comments, commits, createdAt, deletions, files, fullDatabaseId, headRefName, headRefOid, headRepository, headRepositoryOwner, id, isCrossRepository, isDraft, labels, latestReviews, maintainerCanModify, mergeCommit, mergeStateStatus, mergeable, mergedAt, mergedBy, milestone, number, potentialMergeCommit, projectCards, projectItems, reactionGroups, reviewDecision, reviewRequests, reviews, state, statusCheckRollup, title, updatedAt, url

**Examples:**
```bash
gh pr list --author "@me"
gh pr list --label bug --label "priority 1"
gh pr list --search "status:success review:required"
gh pr list --search "<SHA>" --state merged   # Find PR for a commit
gh pr list --json number,title,author --jq '.[].title'
```

---

## pr view

Display PR details. Without argument, shows PR for current branch.

```
gh pr view [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--comments` | `-c` | Show PR comments |
| `--json` | | Output JSON |
| `--jq` | `-q` | jq filter |
| `--template` | `-t` | Go template |
| `--web` | `-w` | Open in browser |

**JSON fields:** Same as `pr list`.

---

## pr checkout

Check out a PR branch locally. Aliases: `gh pr co`.

```
gh pr checkout [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--branch` | `-b` | Local branch name (default: head branch name) |
| `--detach` | | Checkout with detached HEAD |
| `--force` | `-f` | Reset existing local branch |
| `--recurse-submodules` | | Update submodules |

**Examples:**
```bash
gh pr checkout 32
gh pr checkout feature-branch
gh pr checkout 32 --branch my-local-name
```

---

## pr checks

Show CI status for a PR. Without argument, uses current branch's PR.

```
gh pr checks [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--watch` | | Watch until checks finish |
| `--fail-fast` | | Exit watch on first failure |
| `--interval` | `-i` | Refresh interval in seconds (default 10) |
| `--required` | | Show only required checks |
| `--json` | | Output JSON |
| `--web` | `-w` | Open in browser |

**Exit code 8** = checks still pending.

**JSON fields:** bucket, completedAt, description, event, link, name, startedAt, state, workflow

---

## pr merge

Merge a PR. Without argument, merges current branch's PR.

```
gh pr merge [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--merge` | `-m` | Merge commit |
| `--squash` | `-s` | Squash and merge |
| `--rebase` | `-r` | Rebase and merge |
| `--auto` | | Auto-merge when requirements met |
| `--disable-auto` | | Disable auto-merge |
| `--delete-branch` | `-d` | Delete branch after merge |
| `--admin` | | Bypass merge queue/requirements |
| `--body` | `-b` | Merge commit body |
| `--subject` | `-t` | Merge commit subject |
| `--match-head-commit` | | Require head SHA match |

**Notes:**
- For repos with merge queues, no strategy flag needed — PR is queued automatically.
- `--auto` enables auto-merge when required checks pass.

---

## pr review

Add a review to a PR.

```
gh pr review [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--approve` | `-a` | Approve |
| `--comment` | `-c` | Comment review |
| `--request-changes` | `-r` | Request changes |
| `--body` | `-b` | Review body text |
| `--body-file` | `-F` | Read body from file |

**Examples:**
```bash
gh pr review --approve
gh pr review 123 -r -b "needs more tests"
gh pr review --comment -b "looks good overall"
```

---

## pr edit

Edit PR properties. Without argument, edits current branch's PR.

```
gh pr edit [<number> | <url> | <branch>] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--title` | Set new title |
| `--body` / `--body-file` | Set new body |
| `--base` | Change base branch |
| `--add-label` / `--remove-label` | Manage labels |
| `--add-assignee` / `--remove-assignee` | Manage assignees (`@me`, `@copilot`) |
| `--add-reviewer` / `--remove-reviewer` | Manage reviewers |
| `--add-project` / `--remove-project` | Manage projects |
| `--milestone` / `--remove-milestone` | Manage milestone |

---

## pr close / reopen

```bash
gh pr close {<number> | <url> | <branch>} [--comment "reason"] [--delete-branch]
gh pr reopen {<number> | <url> | <branch>} [--comment "reason"]
```

---

## pr diff

View changes in a PR.

```bash
gh pr diff [<number>] [--color always|never|auto] [--name-only] [--patch] [--web]
```

---

## pr comment

```bash
gh pr comment [<number>] --body "comment text"
gh pr comment [<number>] --body-file comment.md
gh pr comment [<number>] --edit-last --body "updated"
gh pr comment [<number>] --delete-last [--yes]
```

---

## pr ready

Mark PR as ready for review (or convert back to draft).

```bash
gh pr ready [<number>]         # Mark ready
gh pr ready [<number>] --undo  # Convert to draft
```

---

## pr update-branch

Update PR branch with latest base branch changes.

```bash
gh pr update-branch [<number>]           # Merge base into PR branch
gh pr update-branch [<number>] --rebase  # Rebase PR on top of base
```

---

## pr lock / unlock

```bash
gh pr lock {<number>} [--reason off_topic|resolved|spam|too_heated]
gh pr unlock {<number>}
```
