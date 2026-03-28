# gh release — Release Commands

## Table of Contents
- [release create](#release-create)
- [release list](#release-list)
- [release view](#release-view)
- [release edit](#release-edit)
- [release download](#release-download)
- [release upload](#release-upload)
- [release delete](#release-delete)
- [release delete-asset](#release-delete-asset)

All commands accept `-R owner/repo`.

---

## release create

Create a new release. Aliases: `gh release new`.

```
gh release create [<tag>] [<files>... | <pattern>...] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--title` | `-t` | Release title |
| `--notes` | `-n` | Release notes |
| `--notes-file` | `-F` | Read notes from file (`-` for stdin) |
| `--notes-from-tag` | | Use annotated tag message as notes |
| `--generate-notes` | | Auto-generate title and notes via GitHub API |
| `--notes-start-tag` | | Starting tag for generated notes |
| `--draft` | `-d` | Save as draft |
| `--prerelease` | `-p` | Mark as prerelease |
| `--latest` | | Mark as latest (`--latest=false` to skip) |
| `--target` | | Target branch/commit for auto-tag |
| `--verify-tag` | | Abort if tag doesn't exist |
| `--discussion-category` | | Start a discussion |
| `--fail-on-no-commits` | | Fail if no new commits since last release |

**Asset labels:** Append `#label text` after filename:
```bash
gh release create v1.0 'dist/app.zip#Application Bundle'
```

**Examples:**
```bash
# Interactive
gh release create

# With auto-generated notes
gh release create v1.0.0 --generate-notes

# With custom notes
gh release create v1.0.0 --notes "## Changes\n- Fixed bug"

# With notes from file
gh release create v1.0.0 -F CHANGELOG.md

# With assets
gh release create v1.0.0 ./dist/*.tar.gz ./dist/*.deb

# Draft prerelease
gh release create v2.0.0-beta.1 --draft --prerelease

# From annotated tag
gh release create v1.0.0 --notes-from-tag

# Fail if no new commits
gh release create v1.0.1 --fail-on-no-commits --generate-notes
```

**Behavior:**
- If tag doesn't exist, it's created from the default branch (or `--target`).
- `--generate-notes` uses GitHub Release Notes API.
- When both `--notes` and `--generate-notes` are used, custom notes are prepended.

---

## release list

List releases. Aliases: `gh release ls`.

```
gh release list [flags]
```

**Flags:**
| Flag | Description |
|------|-------------|
| `--limit` (`-L`) | Max results (default 30) |
| `--exclude-drafts` | Hide drafts |
| `--exclude-pre-releases` | Hide prereleases |
| `--order` (`-O`) | `asc` or `desc` (default `desc`) |
| `--json`, `--jq`, `--template` | Output formatting |

**JSON fields:** createdAt, isDraft, isLatest, isPrerelease, name, publishedAt, tagName

---

## release view

View release details. Without argument, shows latest.

```
gh release view [<tag>] [--json] [--jq] [--template] [--web]
```

**JSON fields:** apiUrl, assets, author, body, createdAt, databaseId, id, isDraft, isImmutable, isPrerelease, name, publishedAt, tagName, tarballUrl, targetCommitish, uploadUrl, url, zipballUrl

---

## release edit

```
gh release edit <tag> [flags]
```

**Key flags:**
| Flag | Description |
|------|-------------|
| `--title` (`-t`) | New title |
| `--notes` (`-n`) / `--notes-file` (`-F`) | New notes |
| `--tag` | Change tag name |
| `--target` | Change target branch/commit |
| `--draft` / `--draft=false` | Toggle draft status |
| `--prerelease` / `--prerelease=false` | Toggle prerelease |
| `--latest` / `--latest=false` | Toggle latest flag |
| `--discussion-category` | Start discussion on publish |
| `--verify-tag` | Abort if tag doesn't exist |

**Examples:**
```bash
gh release edit v1.0 --draft=false          # Publish a draft
gh release edit v1.0 --notes-file notes.md  # Update notes
gh release edit v1.0 --latest=false         # Remove latest flag
```

---

## release download

Download release assets.

```
gh release download [<tag>] [flags]
```

**Key flags:**
| Flag | Short | Description |
|------|-------|-------------|
| `--pattern` | `-p` | Glob pattern (repeatable) |
| `--archive` | `-A` | Download source archive: `zip` or `tar.gz` |
| `--dir` | `-D` | Target directory (default `.`) |
| `--output` | `-O` | Write single asset to file (`-` for stdout) |
| `--clobber` | | Overwrite existing files |
| `--skip-existing` | | Skip if file exists |

Without `<tag>`, downloads from latest release (requires `--pattern` or `--archive`).

**Examples:**
```bash
gh release download v1.0.0                         # All assets
gh release download v1.0.0 --pattern '*.deb'       # Debs only
gh release download -p '*.deb' -p '*.rpm'          # Multiple patterns
gh release download v1.0.0 --archive=zip           # Source archive
gh release download v1.0.0 -O app.zip -D ./output  # To specific file
```

---

## release upload

Upload assets to an existing release.

```
gh release upload <tag> <files>... [--clobber]
```

**Asset labels:** `'path/to/file.zip#Display Name'`

---

## release delete

```
gh release delete <tag> [--yes] [--cleanup-tag]
```

`--cleanup-tag` also deletes the git tag.

---

## release delete-asset

```
gh release delete-asset <tag> <asset-name> [--yes]
```
