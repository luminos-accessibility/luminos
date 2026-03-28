# gh repo — Repository Commands

## Table of Contents
- [repo create](#repo-create)
- [repo clone](#repo-clone)
- [repo fork](#repo-fork)
- [repo list](#repo-list)
- [repo view](#repo-view)
- [repo edit](#repo-edit)
- [repo sync](#repo-sync)
- [repo rename](#repo-rename)
- [repo delete](#repo-delete)
- [repo archive / unarchive](#repo-archive--unarchive)
- [repo set-default](#repo-set-default)
- [repo deploy-key](#repo-deploy-key)

Repositories can be specified as `OWNER/REPO` or by URL.

---

## repo create

Create a new repository. Aliases: `gh repo new`.

```
gh repo create [<name>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--public` | | Public visibility |
| `--private` | | Private visibility |
| `--internal` | | Internal visibility (orgs) |
| `--clone` | `-c` | Clone after creating |
| `--source` | `-s` | Create from local directory |
| `--push` | | Push local commits to new repo |
| `--description` | `-d` | Repository description |
| `--homepage` | `-h` | Homepage URL |
| `--license` | `-l` | License keyword |
| `--gitignore` | `-g` | Gitignore template |
| `--template` | `-p` | Create from template repo |
| `--team` | `-t` | Grant team access (org repos) |
| `--add-readme` | | Add README |
| `--disable-issues` | | Disable issues |
| `--disable-wiki` | | Disable wiki |
| `--include-all-branches` | | Include all template branches |
| `--remote` | `-r` | Remote name for new repo |

**Examples:**
```bash
# Interactive
gh repo create

# Create and clone
gh repo create my-project --public --clone

# In a different org
gh repo create my-org/my-project --public

# From existing local repo
gh repo create my-project --private --source=. --remote=upstream --push

# From template
gh repo create my-project --template owner/template-repo --clone
```

---

## repo clone

Clone a repository locally.

```
gh repo clone <repository> [<directory>] [-- <gitflags>...]
```

**Flags:** `--upstream-remote-name` (`-u`, default "upstream")

**Behavior:**
- Omitting `OWNER/` defaults to authenticated user
- Forks get parent added as `upstream` remote
- Parent is set as default remote for forks
- Supports any URL format or `OWNER/REPO` shorthand

**Examples:**
```bash
gh repo clone cli/cli
gh repo clone myrepo
gh repo clone cli/cli workspace/cli
gh repo clone cli/cli -- --depth=1
```

---

## repo fork

Fork a repository.

```
gh repo fork [<repository>] [-- <gitflags>...] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--clone` | Clone the fork |
| `--remote` | Add git remote for fork |
| `--remote-name` | Name for new remote (default "origin") |
| `--org` | Fork to an organization |
| `--fork-name` | Custom name for the fork |
| `--default-branch-only` | Only include default branch |

---

## repo list

List repositories. Aliases: `gh repo ls`.

```
gh repo list [<owner>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--limit` | `-L` | Max repos (default 30) |
| `--language` | `-l` | Filter by language |
| `--topic` | | Filter by topic |
| `--visibility` | | `public`, `private`, `internal` |
| `--fork` | | Show only forks |
| `--source` | | Show only non-forks |
| `--archived` | | Show only archived |
| `--no-archived` | | Omit archived |
| `--json` | | JSON output |
| `--jq` / `--template` | | Output formatting |

**Examples:**
```bash
gh repo list my-org --limit 100 --language rust
gh repo list --json name,stargazerCount --jq 'sort_by(.stargazerCount) | reverse | .[:5]'
```

---

## repo view

Display repository info and README.

```
gh repo view [<repository>] [flags]
```

**Flags:** `--branch` (`-b`), `--json`, `--jq`, `--template`, `--web` (`-w`)

---

## repo edit

Edit repository settings.

```
gh repo edit [<repository>] [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--description` | Set description |
| `--homepage` | Set homepage URL |
| `--default-branch` | Set default branch |
| `--visibility` | Change visibility (requires `--accept-visibility-change-consequences`) |
| `--enable-issues` / `=false` | Toggle issues |
| `--enable-wiki` / `=false` | Toggle wiki |
| `--enable-discussions` / `=false` | Toggle discussions |
| `--enable-projects` / `=false` | Toggle projects |
| `--enable-auto-merge` / `=false` | Toggle auto-merge |
| `--enable-squash-merge` / `=false` | Toggle squash merge |
| `--enable-merge-commit` / `=false` | Toggle merge commits |
| `--enable-rebase-merge` / `=false` | Toggle rebase merge |
| `--delete-branch-on-merge` / `=false` | Toggle branch deletion |
| `--add-topic` / `--remove-topic` | Manage topics |
| `--template` / `=false` | Toggle template repo |
| `--enable-advanced-security` | Enable advanced security |
| `--enable-secret-scanning` | Enable secret scanning |

**Examples:**
```bash
gh repo edit --enable-issues --enable-wiki
gh repo edit --enable-projects=false
gh repo edit --add-topic rust --add-topic accessibility
gh repo edit --default-branch main
```

---

## repo sync

Sync a repository from its source (parent for forks, or specified source).

```
gh repo sync [<destination-repository>] [flags]
```

**Flags:** `--branch` (`-b`), `--source` (`-s`), `--force`

**Examples:**
```bash
gh repo sync                              # Sync local from remote parent
gh repo sync --branch v1                  # Sync specific branch
gh repo sync owner/cli-fork               # Sync remote fork
gh repo sync owner/repo --source owner2/repo2  # Custom source
```

---

## repo rename

```bash
gh repo rename <new-name> [-R owner/repo] [--yes]
```

---

## repo delete

```bash
gh repo delete [<repository>] [--yes]
```

Requires `delete_repo` scope: `gh auth refresh -s delete_repo`

---

## repo archive / unarchive

```bash
gh repo archive [<repository>] [--yes]
gh repo unarchive [<repository>] [--yes]
```

---

## repo set-default

Set the default remote repository for the current directory.

```bash
gh repo set-default [<repository>]
gh repo set-default --view    # View current default
gh repo set-default --unset   # Clear default
```

---

## repo deploy-key

```bash
gh repo deploy-key list [-R owner/repo]
gh repo deploy-key add <key-file> [--title "name"] [--allow-write]
gh repo deploy-key delete <key-id>
```
