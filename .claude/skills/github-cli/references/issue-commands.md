# gh issue — Issue Commands

## Table of Contents
- [issue create](#issue-create)
- [issue list](#issue-list)
- [issue view](#issue-view)
- [issue edit](#issue-edit)
- [issue close / reopen](#issue-close--reopen)
- [issue comment](#issue-comment)
- [issue develop](#issue-develop)
- [issue delete](#issue-delete)
- [issue pin / unpin](#issue-pin--unpin)
- [issue transfer](#issue-transfer)
- [issue lock / unlock](#issue-lock--unlock)
- [issue status](#issue-status)

All `gh issue` commands accept `-R owner/repo` to target a different repository.

Issues can be specified by number (`123`) or URL.

---

## issue create

Create a new issue. Aliases: `gh issue new`.

```
gh issue create [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--title` | `-t` | Issue title |
| `--body` | `-b` | Issue body |
| `--body-file` | `-F` | Read body from file (`-` for stdin) |
| `--assignee` | `-a` | Assign users (`@me` for self, `@copilot` for Copilot) |
| `--label` | `-l` | Add labels |
| `--milestone` | `-m` | Add to milestone |
| `--project` | `-p` | Add to project |
| `--template` | `-T` | Use issue template by name |
| `--web` | `-w` | Open browser to create |
| `--editor` | `-e` | Open text editor |
| `--recover` | | Recover from failed create |

**Examples:**
```bash
gh issue create --title "Bug: X broken" --body "Steps to reproduce..."
gh issue create --label "bug,help wanted"
gh issue create --assignee monalisa,hubot
gh issue create --assignee "@me"
gh issue create --project "Roadmap"
gh issue create --template "Bug Report"
```

---

## issue list

List issues. Default: open issues only. Aliases: `gh issue ls`.

```
gh issue list [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--state` | `-s` | `open` (default), `closed`, `all` |
| `--author` | `-A` | Filter by author |
| `--assignee` | `-a` | Filter by assignee (`@me`) |
| `--label` | `-l` | Filter by label (multiple = AND) |
| `--milestone` | `-m` | Filter by milestone name or number |
| `--mention` | | Filter by user mention |
| `--search` | `-S` | Search with GitHub query syntax |
| `--limit` | `-L` | Max results (default 30) |
| `--json` | | Output JSON |
| `--jq` | `-q` | jq filter |
| `--template` | `-t` | Go template |
| `--web` | `-w` | Open in browser |
| `--app` | | Filter by GitHub App author |

**JSON fields:** assignees, author, body, closed, closedAt, closedByPullRequestsReferences, comments, createdAt, id, isPinned, labels, milestone, number, projectCards, projectItems, reactionGroups, state, stateReason, title, updatedAt, url

**Examples:**
```bash
gh issue list --label "bug" --label "help wanted"
gh issue list --assignee "@me" --state all
gh issue list --milestone "The big 1.0"
gh issue list --search "error no:assignee sort:created-asc"
gh issue list --json number,title,labels --jq '.[] | select(.labels | length > 0)'
```

---

## issue view

Display issue details.

```
gh issue view {<number> | <url>} [flags]
```

**Key flags:** `--comments` `-c`, `--json`, `--jq`, `--template`, `--web`.

**JSON fields:** Same as `issue list`.

---

## issue edit

Edit one or more issues.

```
gh issue edit {<numbers> | <urls>} [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--title` | Set new title |
| `--body` / `--body-file` | Set new body |
| `--add-label` / `--remove-label` | Manage labels |
| `--add-assignee` / `--remove-assignee` | Manage assignees (`@me`, `@copilot`) |
| `--add-project` / `--remove-project` | Manage projects |
| `--milestone` / `--remove-milestone` | Manage milestone |

**Examples:**
```bash
gh issue edit 23 --title "Updated title" --body "New body"
gh issue edit 23 --add-label "bug,help wanted" --remove-label "core"
gh issue edit 23 34 --add-label "help wanted"  # Edit multiple
```

---

## issue close / reopen

```bash
gh issue close {<number>} [--comment "reason"] [--reason completed|"not planned"]
gh issue reopen {<number>} [--comment "reopening because..."]
```

---

## issue comment

```bash
gh issue comment {<number>} --body "comment text"
gh issue comment {<number>} --body-file comment.md
gh issue comment {<number>} --edit-last --body "updated"
gh issue comment {<number>} --delete-last [--yes]
```

---

## issue develop

Manage development branches linked to an issue.

```bash
# Create branch linked to issue and check it out
gh issue develop 123 --checkout

# Create branch from specific base
gh issue develop 123 --base my-feature

# Name the branch explicitly
gh issue develop 123 --name my-fix-branch

# List linked branches
gh issue develop --list 123

# Create branch in a different repo
gh issue develop 123 --repo cli/cli --branch-repo monalisa/cli
```

---

## issue delete

```bash
gh issue delete {<number>} [--yes]
```

---

## issue pin / unpin

```bash
gh issue pin {<number>}
gh issue unpin {<number>}
```

---

## issue transfer

```bash
gh issue transfer {<number>} <destination-repo>
```

---

## issue lock / unlock

```bash
gh issue lock {<number>} [--reason off_topic|resolved|spam|too_heated]
gh issue unlock {<number>}
```

---

## issue status

Show issues relevant to you (assigned, mentioned, recent activity).

```bash
gh issue status [--json] [--jq] [--template]
```
