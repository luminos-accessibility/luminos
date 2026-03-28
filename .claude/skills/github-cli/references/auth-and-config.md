# gh auth & gh config — Authentication and Configuration

## Table of Contents
- [auth login](#auth-login)
- [auth logout](#auth-logout)
- [auth status](#auth-status)
- [auth token](#auth-token)
- [auth refresh](#auth-refresh)
- [auth switch](#auth-switch)
- [auth setup-git](#auth-setup-git)
- [config set / get / list](#config-commands)
- [Environment Variables](#environment-variables)

---

## auth login

Authenticate with a GitHub host. Default: github.com with browser-based flow.

```
gh auth login [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--hostname` | `-h` | Target host (default github.com) |
| `--web` | `-w` | Force browser auth |
| `--with-token` | | Read PAT from stdin |
| `--git-protocol` | `-p` | Set git protocol: `ssh` or `https` |
| `--scopes` | `-s` | Additional OAuth scopes |
| `--skip-ssh-key` | | Skip SSH key setup |
| `--clipboard` | `-c` | Copy OAuth code to clipboard |
| `--insecure-storage` | | Store in plaintext (not credential store) |

**Examples:**
```bash
# Interactive browser auth
gh auth login

# With browser and clipboard
gh auth login --web --clipboard

# Token from file (non-interactive)
gh auth login --with-token < token.txt

# Token from environment
echo "$MY_TOKEN" | gh auth login --with-token

# Enterprise host
gh auth login --hostname enterprise.internal

# Set git protocol during login
gh auth login --git-protocol ssh
```

**Minimum token scopes:** `repo`, `read:org`, `gist`

**In GitHub Actions:**
```yaml
env:
  GH_TOKEN: ${{ github.token }}
```

---

## auth logout

Remove local auth configuration (does NOT revoke the token).

```bash
gh auth logout                                           # Interactive
gh auth logout --hostname enterprise.internal --user monalisa  # Specific
```

To revoke tokens: visit https://github.com/settings/applications → "GitHub CLI" → "Revoke Access"

---

## auth status

Check authentication state.

```bash
gh auth status                            # All accounts on all hosts
gh auth status --active                   # Active account only
gh auth status --hostname github.com      # Specific host
gh auth status --show-token               # Display tokens
gh auth status --json hosts               # JSON output
gh auth status --json hosts --show-token  # JSON with tokens
gh auth status --json hosts --jq '.hosts | add'  # Flat array
```

Exit code 1 if any account has auth issues.

---

## auth token

Print the auth token for a host/account.

```bash
gh auth token                              # Default host, active account
gh auth token --hostname enterprise.internal
gh auth token --user monalisa
```

---

## auth refresh

Update OAuth scopes for the active account.

```bash
# Add scopes
gh auth refresh --scopes write:org,read:public_key

# Remove scopes
gh auth refresh --remove-scopes delete_repo

# Reset to defaults
gh auth refresh --reset-scopes

# Common scope additions
gh auth refresh -s project          # For project management
gh auth refresh -s delete_repo      # For repo deletion
gh auth refresh -s admin:org        # For org admin
```

Note: To refresh an inactive account, first `gh auth switch` to it.

---

## auth switch

Switch active account for a host.

```bash
gh auth switch                                      # Interactive
gh auth switch --hostname enterprise.internal --user monalisa
```

---

## auth setup-git

Configure git to use gh as a credential helper.

```bash
gh auth setup-git                                   # All authenticated hosts
gh auth setup-git --hostname enterprise.internal    # Specific host
gh auth setup-git --hostname newhost.com --force    # Even if not authenticated
```

---

## config commands

### config set

```bash
gh config set git_protocol ssh              # ssh or https
gh config set editor vim                    # Text editor
gh config set prompt disabled               # Disable interactive prompts
gh config set prefer_editor_prompt enabled  # Prefer editor for prompts
gh config set pager less                    # Terminal pager
gh config set browser firefox              # Web browser
```

### config get

```bash
gh config get git_protocol
gh config get editor
```

### config list

```bash
gh config list
```

### config clear-cache

```bash
gh config clear-cache
```

### All configuration keys

| Key | Values | Default | Description |
|-----|--------|---------|-------------|
| `git_protocol` | `ssh`, `https` | `https` | Git clone/push protocol |
| `editor` | path/name | system default | Text editor |
| `prompt` | `enabled`, `disabled` | `enabled` | Interactive prompting |
| `prefer_editor_prompt` | `enabled`, `disabled` | `disabled` | Prefer editor over inline |
| `pager` | path/name | system default | Terminal pager |
| `browser` | path/name | system default | Web browser |
| `http_unix_socket` | path | | Unix socket for HTTP |
| `color_labels` | `enabled`, `disabled` | `disabled` | RGB label colors |
| `accessible_colors` | `enabled`, `disabled` | `disabled` | 4-bit accessible colors |
| `accessible_prompter` | `enabled`, `disabled` | `disabled` | Accessible prompts |
| `spinner` | `enabled`, `disabled` | `enabled` | Animated spinner |

---

## Environment Variables

### Authentication

| Variable | Description |
|----------|-------------|
| `GH_TOKEN` | Auth token for github.com (highest priority) |
| `GITHUB_TOKEN` | Auth token for github.com (fallback) |
| `GH_ENTERPRISE_TOKEN` | Auth token for Enterprise hosts |
| `GITHUB_ENTERPRISE_TOKEN` | Enterprise token (fallback) |

### Context

| Variable | Description |
|----------|-------------|
| `GH_HOST` | Target GitHub hostname |
| `GH_REPO` | Override repository (`[HOST/]OWNER/REPO`) |

### Editor & Browser

| Variable | Description |
|----------|-------------|
| `GH_EDITOR` | Editor (highest priority) |
| `GIT_EDITOR` | Editor (2nd priority) |
| `VISUAL` | Editor (3rd priority) |
| `EDITOR` | Editor (lowest priority) |
| `GH_BROWSER` | Browser (highest priority) |
| `BROWSER` | Browser (fallback) |

### Display

| Variable | Description |
|----------|-------------|
| `GH_PAGER` / `PAGER` | Terminal pager |
| `NO_COLOR` | Disable ANSI colors |
| `CLICOLOR=0` | Disable colors |
| `CLICOLOR_FORCE` | Force colors in pipes |
| `GH_FORCE_TTY` | Force TTY output (value = columns) |
| `GH_COLOR_LABELS` | Enable RGB label colors |
| `GLAMOUR_STYLE` | Markdown rendering style |
| `GH_MDWIDTH` | Max markdown wrap width |

### Behavior

| Variable | Description |
|----------|-------------|
| `GH_DEBUG` | `1` for verbose, `api` for HTTP logging |
| `GH_PROMPT_DISABLED` | Disable interactive prompts |
| `GH_NO_UPDATE_NOTIFIER` | Disable update notices |
| `GH_NO_EXTENSION_UPDATE_NOTIFIER` | Disable extension update notices |
| `GH_SPINNER_DISABLED` | Replace spinner with text |

### Paths

| Variable | Description |
|----------|-------------|
| `GH_CONFIG_DIR` | Config directory (default `~/.config/gh`) |
| `GH_PATH` | Path to gh executable |
