# Other gh Commands

## Table of Contents
- [gist](#gist)
- [label](#label)
- [secret](#secret)
- [variable](#variable)
- [status](#status)
- [browse](#browse)
- [codespace](#codespace)
- [project](#project)
- [org](#org)
- [ruleset](#ruleset)
- [extension](#extension)
- [alias](#alias)
- [ssh-key / gpg-key](#ssh-key--gpg-key)

---

## gist

Manage GitHub gists.

```bash
# Create
gh gist create hello.py                          # Secret gist
gh gist create --public hello.py                 # Public gist
gh gist create hello.py -d "My gist"            # With description
gh gist create hello.py world.py                 # Multiple files
gh gist create *.md *.txt                        # Glob patterns
cat file.txt | gh gist create -                  # From stdin
gh gist create - --filename output.log           # Stdin with filename

# List
gh gist list                                     # Your gists (default 10)
gh gist list --public                            # Public only
gh gist list --secret                            # Secret only
gh gist list --filter octo --include-content     # Search content

# View
gh gist view <id>                                # View gist
gh gist view <id> --filename main.py             # Single file
gh gist view <id> --files                        # List filenames
gh gist view <id> --raw                          # Raw content
gh gist view <id> --web                          # Open in browser

# Edit
gh gist edit <id>                                # Edit in editor
gh gist edit <id> --add newfile.py               # Add file
gh gist edit <id> --remove oldfile.py            # Remove file
gh gist edit <id> --desc "New description"       # Update description

# Clone & Delete
gh gist clone <id> [<directory>]
gh gist delete <id> [--yes]
```

Gists specified by ID or URL.

---

## label

Manage repository labels.

```bash
# Create
gh label create "bug" --description "Something isn't working" --color E4E669

# List
gh label list [-R owner/repo]

# Edit
gh label edit "bug" --name "Bug" --description "Updated" --color FF0000

# Delete
gh label delete "bug" [--yes]

# Clone labels from another repo
gh label clone source-owner/source-repo [-R target-owner/target-repo]
```

---

## secret

Manage GitHub secrets (Actions, Dependabot, Codespaces).

```bash
# Set a secret
gh secret set SECRET_NAME                          # Interactive input
gh secret set SECRET_NAME --body "value"           # Direct value
gh secret set SECRET_NAME < secret.txt             # From file
gh secret set SECRET_NAME --env production         # Environment secret
gh secret set SECRET_NAME --org my-org             # Org secret
gh secret set SECRET_NAME --org my-org --visibility all  # Org-wide

# List secrets
gh secret list
gh secret list --env production
gh secret list --org my-org

# Delete
gh secret delete SECRET_NAME
gh secret delete SECRET_NAME --env production
```

---

## variable

Manage GitHub Actions / Dependabot variables.

```bash
# Set
gh variable set VAR_NAME --body "value"
gh variable set VAR_NAME --env production
gh variable set VAR_NAME --org my-org

# Get
gh variable get VAR_NAME
gh variable get VAR_NAME --env production

# List
gh variable list
gh variable list --env production

# Delete
gh variable delete VAR_NAME
```

---

## status

Show your cross-repository activity dashboard.

```bash
gh status                                    # All activity
gh status -e cli/cli -e cli/go-gh           # Exclude repos
gh status -o cli                            # Limit to org
```

Shows: assigned issues, assigned PRs, review requests, mentions, repo activity.

---

## browse

Open repository pages in browser.

```bash
gh browse                                    # Repo home
gh browse 217                                # Issue or PR #217
gh browse script/                            # Directory
gh browse main.go:312                        # File at line
gh browse main.go --branch bug-fix           # File on branch
gh browse main.go --commit=abc123            # File at commit
gh browse --settings                         # Repo settings
gh browse --releases                         # Repo releases
gh browse --projects                         # Repo projects
gh browse --wiki                             # Repo wiki
gh browse -n                                 # Print URL instead of opening
```

---

## codespace

Connect to and manage GitHub Codespaces. Aliases: `gh cs`.

```bash
# Create & manage
gh codespace create [--repo owner/repo] [--machine <type>] [--branch <branch>]
gh codespace list
gh codespace view
gh codespace delete [--all] [--days <N>]
gh codespace stop
gh codespace edit

# Connect
gh codespace ssh                             # SSH into codespace
gh codespace code                            # Open in VS Code

# Files
gh codespace cp local-file remote:path       # Upload
gh codespace cp remote:path local-file       # Download

# Other
gh codespace logs
gh codespace ports [list | forward | visibility]
gh codespace jupyter
gh codespace rebuild [--full]
```

---

## project

Work with GitHub Projects (v2). Requires `project` scope: `gh auth refresh -s project`.

```bash
# Create & manage
gh project create --owner monalisa --title "Roadmap"
gh project list --owner cli
gh project view 1 --owner cli [--web]
gh project edit 1 --owner cli --title "New Title"
gh project close 1
gh project delete 1
gh project copy 1 --source-owner cli --target-owner monalisa --title "Copy"

# Items
gh project item-list 1 --owner cli
gh project item-add 1 --owner cli --url <issue-or-pr-url>
gh project item-create 1 --owner cli --title "Draft issue"
gh project item-edit <item-id> --project-id <id> --field-id <id> --text "value"
gh project item-archive <item-id> --project-id <id>
gh project item-delete <item-id> --project-id <id>

# Fields
gh project field-list 1 --owner cli
gh project field-create 1 --owner cli --name "Priority" --data-type SINGLE_SELECT
gh project field-delete <field-id> --project-id <id>

# Links
gh project link 1 --repo owner/repo
gh project unlink 1 --repo owner/repo
gh project mark-template 1 --owner cli
```

---

## org

```bash
gh org list                                  # List your orgs
```

---

## ruleset

View repository rulesets. Aliases: `gh rs`.

```bash
gh ruleset list [-R owner/repo]
gh ruleset view [<id>] [-R owner/repo] [--web]
gh ruleset check <branch-name> [-R owner/repo]
```

---

## extension

Manage gh extensions — third-party commands.

```bash
# Discover & install
gh extension search <query>
gh extension browse                          # Interactive browser
gh extension install owner/gh-extension-name
gh extension install owner/gh-extension-name --pin v1.0

# Manage
gh extension list
gh extension upgrade <name>
gh extension upgrade --all
gh extension remove <name>

# Create your own
gh extension create <name>
gh extension create <name> --precompiled=go

# Run
gh <extension-name> [args]
gh extension exec <name> [args]              # If name conflicts
```

Extensions must be repos named `gh-<name>`. Browse at: https://github.com/topics/gh-extension

---

## alias

Create command shortcuts.

```bash
# Set aliases
gh alias set pv 'pr view'
gh alias set bugs 'issue list --label=bug'
gh alias set epicsearch 'issue list --label="$1" --json number,title'

# Use shell commands in aliases
gh alias set --shell igrep 'gh issue list --label="$1" | grep "$2"'

# List, import, delete
gh alias list
gh alias import aliases.yml
gh alias delete pv
```

---

## ssh-key / gpg-key

```bash
# SSH keys
gh ssh-key list
gh ssh-key add <key-file> [--title "name"] [--type authentication|signing]
gh ssh-key delete <key-id>

# GPG keys
gh gpg-key list
gh gpg-key add <key-file>
gh gpg-key delete <key-id>
```
