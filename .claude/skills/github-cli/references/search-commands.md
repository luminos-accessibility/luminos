# gh search — Search Commands

## Table of Contents
- [search repos](#search-repos)
- [search issues](#search-issues)
- [search prs](#search-prs)
- [search code](#search-code)
- [search commits](#search-commits)
- [Excluding Results](#excluding-results)

---

## search repos

Search for repositories.

```
gh search repos [<query>] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--owner` | Filter by owner |
| `--language` | Filter by language |
| `--topic` | Filter by topic (repeatable) |
| `--license` | Filter by license |
| `--visibility` | `public`, `private`, `internal` |
| `--stars` | Filter by stars (e.g., `>1000`, `100..500`) |
| `--forks` | Filter by fork count |
| `--created` / `--updated` | Filter by date |
| `--size` | Filter by size in KB |
| `--archived` | Filter by archived state |
| `--include-forks` | `false`, `true`, `only` |
| `--good-first-issues` | Filter by good-first-issue count |
| `--help-wanted-issues` | Filter by help-wanted count |
| `--sort` | `forks`, `help-wanted-issues`, `stars`, `updated` |
| `--order` | `asc`, `desc` |
| `--limit` (`-L`) | Max results (default 30) |
| `--match` | Restrict to `name`, `description`, `readme` |
| `--json`, `--jq`, `--template` | Output formatting |
| `--web` | Open in browser |

**JSON fields:** createdAt, defaultBranch, description, forksCount, fullName, hasDownloads, hasIssues, hasPages, hasProjects, hasWiki, homepage, id, isArchived, isDisabled, isFork, isPrivate, language, license, name, openIssuesCount, owner, pushedAt, size, stargazersCount, updatedAt, url, visibility, watchersCount

**Examples:**
```bash
gh search repos "vim plugin"
gh search repos --owner=microsoft --visibility=public
gh search repos --topic=rust,accessibility
gh search repos --language=go --good-first-issues=">=10"
gh search repos --stars=">5000" --sort=stars
gh search repos -- -topic:linux        # Exclude topic
```

---

## search issues

Search for issues across GitHub.

```
gh search issues [<query>] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--repo` (`-R`) | Filter by repo (repeatable) |
| `--owner` | Filter by owner |
| `--author` | Filter by author |
| `--assignee` | Filter by assignee (`@me`) |
| `--label` | Filter by label (repeatable) |
| `--milestone` | Filter by milestone |
| `--state` | `open`, `closed` |
| `--created` / `--closed` / `--updated` | Date filters |
| `--comments` | Filter by comment count (e.g., `>100`) |
| `--reactions` | Filter by reaction count |
| `--interactions` | Filter by reactions + comments |
| `--language` | Filter by repo language |
| `--involves` | Filter by user involvement |
| `--mentions` / `--commenter` | Filter by mentions/commenters |
| `--no-assignee` / `--no-label` / `--no-milestone` / `--no-project` | Filter for missing |
| `--locked` | Filter locked issues |
| `--archived` | Filter by repo archived state |
| `--visibility` | Filter by repo visibility |
| `--include-prs` | Include pull requests in results |
| `--match` | Restrict to `title`, `body`, `comments` |
| `--sort` | `comments`, `created`, `interactions`, `reactions`, `updated`, etc. |
| `--limit` (`-L`) | Max results (default 30) |
| `--json`, `--jq`, `--template`, `--web` | Output |

**JSON fields:** assignees, author, authorAssociation, body, closedAt, commentsCount, createdAt, id, isLocked, isPullRequest, labels, number, repository, state, title, updatedAt, url

**Examples:**
```bash
gh search issues "broken feature"
gh search issues --owner=cli --include-prs
gh search issues --assignee=@me --state=open
gh search issues --comments=">100"
gh search issues --label bug --repo cli/cli
gh search issues -- -label:bug              # Exclude label
```

---

## search prs

Search for pull requests across GitHub.

```
gh search prs [<query>] [flags]
```

Supports all `search issues` flags PLUS:

| Flag | Description |
|------|-------------|
| `--base` (`-B`) | Filter by base branch |
| `--head` (`-H`) | Filter by head branch |
| `--draft` | Filter drafts |
| `--merged` | Filter merged PRs |
| `--merged-at` | Filter by merge date |
| `--checks` | Filter by check status: `pending`, `success`, `failure` |
| `--review` | `none`, `required`, `approved`, `changes_requested` |
| `--review-requested` | Filter by review request |
| `--reviewed-by` | Filter by reviewer |

**Examples:**
```bash
gh search prs --repo=cli/cli --draft
gh search prs --review-requested=@me --state=open
gh search prs --assignee=@me --merged
gh search prs --checks=failure --state=open
gh search prs -- -label:bug
```

---

## search code

Search within code in repositories.

```
gh search code <query> [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--repo` (`-R`) | Filter by repo (repeatable) |
| `--owner` | Filter by owner |
| `--language` | Filter by language |
| `--filename` | Filter by filename |
| `--extension` | Filter by file extension |
| `--size` | Filter by size in KB |
| `--match` | Restrict to `file` or `path` |
| `--limit` (`-L`) | Max results (default 30) |
| `--json`, `--jq`, `--template`, `--web` | Output |

**JSON fields:** path, repository, sha, textMatches, url

**Examples:**
```bash
gh search code "error handling"
gh search code deque --language=python
gh search code cli --owner=microsoft
gh search code panic --repo cli/cli
gh search code lint --filename package.json
gh search code "use crate" --extension rs
```

Note: Uses the legacy GitHub code search engine. Results may differ from github.com.

---

## search commits

Search for commits across GitHub.

```
gh search commits [<query>] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--repo` (`-R`) | Filter by repo |
| `--owner` | Filter by owner |
| `--author` | Filter by author |
| `--committer` | Filter by committer |
| `--author-name` / `--committer-name` | Filter by name |
| `--author-email` / `--committer-email` | Filter by email |
| `--author-date` / `--committer-date` | Filter by date |
| `--hash` | Filter by commit SHA |
| `--parent` / `--tree` | Filter by parent/tree hash |
| `--merge` | Filter merge commits |
| `--visibility` | Filter by repo visibility |
| `--sort` | `author-date`, `committer-date` |
| `--limit` (`-L`) | Max results (default 30) |
| `--json`, `--jq`, `--template`, `--web` | Output |

**JSON fields:** author, commit, committer, id, parents, repository, sha, url

**Examples:**
```bash
gh search commits "bug fix"
gh search commits --committer=monalisa
gh search commits --author-name="Jane Doe"
gh search commits --hash=8dd03144ffdc6c
gh search commits --author-date="<2022-02-01"
```

---

## Excluding Results

GitHub search supports excluding with `-qualifier:value`. In the shell, this requires special handling to avoid the `-` being interpreted as a flag:

**Unix/Linux/macOS:**
```bash
gh search issues -- "my query -label:bug"
```

**PowerShell:**
```powershell
gh --% search issues -- "my query -label:bug"
```

The `--` tells the shell that everything after it is an argument, not a flag.
