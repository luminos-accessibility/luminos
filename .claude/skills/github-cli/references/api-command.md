# gh api — GitHub API Access

The `gh api` command makes authenticated HTTP requests to the GitHub REST and GraphQL APIs. It is the most flexible command in the CLI — use it for any GitHub operation not covered by built-in commands.

## Table of Contents
- [Basic Usage](#basic-usage)
- [HTTP Methods](#http-methods)
- [Request Parameters](#request-parameters)
- [Request Body from File](#request-body-from-file)
- [Placeholders](#placeholders)
- [Output Formatting](#output-formatting)
- [Pagination](#pagination)
- [GraphQL](#graphql)
- [Headers and Previews](#headers-and-previews)
- [Debugging](#debugging)
- [Caching](#caching)
- [Environment Variables](#environment-variables)
- [Common API Endpoints](#common-api-endpoints)

---

## Basic Usage

```
gh api <endpoint> [flags]
```

The endpoint is a path under `https://api.github.com/`. No need to include the base URL.

```bash
# GET (default method)
gh api repos/{owner}/{repo}

# The response is printed as JSON
gh api repos/cli/cli
```

---

## HTTP Methods

Default is GET. If parameters are added, it switches to POST. Override with `--method` / `-X`:

```bash
gh api -X GET search/issues -f q='repo:cli/cli is:open'
gh api -X POST repos/{owner}/{repo}/issues -f title="Bug"
gh api -X PATCH repos/{owner}/{repo}/issues/123 -f state=closed
gh api -X DELETE repos/{owner}/{repo}/issues/123/labels/bug
gh api -X PUT repos/{owner}/{repo}/pulls/123/merge
```

---

## Request Parameters

### String parameters (`-f` / `--raw-field`)

Always sends the value as a string:

```bash
gh api repos/{owner}/{repo}/issues -f title="Bug report" -f body="Details here"
```

### Typed parameters (`-F` / `--field`)

Smart type conversion:
- `true`, `false`, `null` → JSON booleans/null
- Integer numbers → JSON numbers
- `{owner}`, `{repo}`, `{branch}` → values from current repo
- `@filename` → reads value from file (`@-` for stdin)

```bash
gh api repos/{owner}/{repo}/issues -F title="Bug" -F labels[]="bug" -F labels[]="urgent"
gh api repos/{owner}/{repo}/pulls -F draft=true    # boolean, not string
gh api gists -F 'files[myfile.txt][content]=@myfile.txt'  # file content
```

### Nested parameters

Use bracket syntax for nested objects and arrays:

```bash
# Nested object
gh api endpoint -F 'config[url]=https://example.com' -F 'config[content_type]=json'

# Array
gh api endpoint -F 'items[]=value1' -F 'items[]=value2'

# Empty array
gh api endpoint -F 'items[]'

# Deeply nested
gh api -X PATCH /orgs/{org}/properties/schema \
  -F 'properties[][property_name]=environment' \
  -F 'properties[][default_value]=production' \
  -F 'properties[][allowed_values][]=staging' \
  -F 'properties[][allowed_values][]=production'
```

---

## Request Body from File

For pre-constructed JSON payloads, use `--input`:

```bash
# From file
gh api repos/{owner}/{repo}/issues --input issue.json

# From stdin
echo '{"title":"Bug","body":"Details"}' | gh api repos/{owner}/{repo}/issues --input -

# From heredoc
gh api repos/{owner}/{repo}/issues --input - <<'EOF'
{
  "title": "Bug report",
  "body": "Something is broken",
  "labels": ["bug"]
}
EOF
```

When `--input` is used with `-f`/`-F` flags, the parameters go to the query string instead of body.

---

## Placeholders

These placeholders are auto-replaced from the current repo context:
- `{owner}` — repository owner
- `{repo}` — repository name
- `{branch}` — current branch

```bash
gh api repos/{owner}/{repo}/releases        # Current repo's releases
gh api repos/{owner}/{repo}/commits/{branch} # Current branch's latest commit
```

In shells that treat `{}` specially (e.g., PowerShell), quote the endpoint.

---

## Output Formatting

### jq filtering (`--jq` / `-q`)

Filter and transform JSON output using jq syntax (no `jq` binary required):

```bash
# Extract single field
gh api repos/{owner}/{repo}/releases --jq '.[0].tag_name'

# Extract array of values
gh api repos/{owner}/{repo}/issues --jq '.[].title'

# Complex transformation
gh api repos/{owner}/{repo}/issues --jq \
  '[.[] | {number, title, labels: [.labels[].name]}]'

# Conditional filtering
gh api repos/{owner}/{repo}/issues --jq \
  '[.[] | select(.labels | length > 0)]'

# Format as TSV
gh api repos/{owner}/{repo}/issues --jq '.[] | [.number, .title] | @tsv'
```

### Go templates (`--template` / `-t`)

```bash
gh api repos/{owner}/{repo}/issues --template \
  '{{range .}}{{.title}} ({{.labels | pluck "name" | join ", " | color "yellow"}}){{"\n"}}{{end}}'
```

Template functions: `autocolor`, `color`, `join`, `pluck`, `tablerow`, `tablerender`, `timeago`, `timefmt`, `truncate`, `hyperlink`.

### Silent mode (`--silent`)

Suppress response body (useful for POST/PATCH/DELETE when you only care about success):

```bash
gh api -X DELETE repos/{owner}/{repo}/issues/123/labels/bug --silent
```

### Include headers (`--include` / `-i`)

Print HTTP status line and headers:

```bash
gh api repos/{owner}/{repo} -i
# HTTP/2.0 200 OK
# Content-Type: application/json; charset=utf-8
# ...
```

---

## Pagination

Use `--paginate` to automatically fetch all pages:

```bash
# Get ALL stargazers (not just first page)
gh api repos/{owner}/{repo}/stargazers --paginate --jq '.[].login'

# Collect all pages into a single array with --slurp
gh api repos/{owner}/{repo}/issues --paginate --slurp --jq 'flatten | length'
```

**REST pagination:** `--paginate` follows `Link` headers automatically.

**GraphQL pagination:** Requires `$endCursor: String` variable and `pageInfo { hasNextPage, endCursor }` in the query. See [GraphQL](#graphql) below.

`--slurp` wraps all pages into an outer JSON array — useful when each page is a separate array.

---

## GraphQL

Use `graphql` as the endpoint:

```bash
# Simple query
gh api graphql -f query='{ viewer { login } }'

# With variables (use -F for typed, -f for strings)
gh api graphql -F owner='{owner}' -F name='{repo}' -f query='
  query($name: String!, $owner: String!) {
    repository(owner: $owner, name: $name) {
      releases(last: 3) {
        nodes { tagName }
      }
    }
  }
'

# Paginated GraphQL
gh api graphql --paginate -f query='
  query($endCursor: String) {
    viewer {
      repositories(first: 100, after: $endCursor) {
        nodes { nameWithOwner }
        pageInfo {
          hasNextPage
          endCursor
        }
      }
    }
  }
'

# Paginated + slurp + jq processing
gh api graphql --paginate --slurp -f query='
  query($endCursor: String) {
    viewer {
      repositories(first: 100, after: $endCursor) {
        nodes { isFork }
        pageInfo { hasNextPage endCursor }
      }
    }
  }
' | jq '[.[].data.viewer.repositories.nodes[]] | map(select(.isFork)) | length'
```

For GraphQL, all `-f`/`-F` fields other than `query` and `operationName` are passed as GraphQL variables.

---

## Headers and Previews

### Custom headers (`-H` / `--header`)

```bash
gh api repos/{owner}/{repo} -H 'Accept: application/vnd.github.v3.raw+json'
gh api repos/{owner}/{repo} -H 'X-GitHub-Api-Version: 2022-11-28'
```

### API previews (`-p` / `--preview`)

```bash
gh api repos/{owner}/{repo} --preview baptiste,nebula
```

---

## Debugging

### Verbose mode (`--verbose`)

Shows full HTTP request and response:

```bash
gh api repos/{owner}/{repo} --verbose
```

### Environment variable

```bash
GH_DEBUG=api gh api repos/{owner}/{repo}  # Log all HTTP traffic
```

---

## Caching

Cache responses for a duration:

```bash
gh api repos/{owner}/{repo}/releases --cache 1h
gh api repos/{owner}/{repo}/contributors --cache 3600s
```

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `GH_TOKEN` / `GITHUB_TOKEN` | Auth token for github.com |
| `GH_ENTERPRISE_TOKEN` | Auth token for Enterprise |
| `GH_HOST` | Target a different GitHub host |

---

## Common API Endpoints

### Repository
```bash
gh api repos/{owner}/{repo}                    # Repo info
gh api repos/{owner}/{repo}/contributors       # Contributors
gh api repos/{owner}/{repo}/topics             # Topics
gh api repos/{owner}/{repo}/languages          # Languages
gh api repos/{owner}/{repo}/tags               # Tags
gh api repos/{owner}/{repo}/branches           # Branches
```

### Issues & PRs
```bash
gh api repos/{owner}/{repo}/issues             # List issues
gh api repos/{owner}/{repo}/issues/123         # Single issue
gh api repos/{owner}/{repo}/issues/123/comments # Issue comments
gh api repos/{owner}/{repo}/pulls/123/comments  # PR review comments
gh api repos/{owner}/{repo}/pulls/123/reviews   # PR reviews
gh api repos/{owner}/{repo}/pulls/123/files     # PR changed files
gh api repos/{owner}/{repo}/pulls/123/commits   # PR commits
```

### Actions
```bash
gh api repos/{owner}/{repo}/actions/runs       # Workflow runs
gh api repos/{owner}/{repo}/actions/workflows  # Workflows
gh api repos/{owner}/{repo}/actions/artifacts  # Artifacts
gh api repos/{owner}/{repo}/actions/caches     # Caches
```

### Releases
```bash
gh api repos/{owner}/{repo}/releases           # List releases
gh api repos/{owner}/{repo}/releases/latest    # Latest release
gh api repos/{owner}/{repo}/releases/tags/v1.0 # By tag
```

### Users & Orgs
```bash
gh api user                                    # Authenticated user
gh api users/{username}                        # User info
gh api orgs/{org}                              # Org info
gh api orgs/{org}/repos                        # Org repos
gh api orgs/{org}/members                      # Org members
```

### Search
```bash
gh api search/repositories -f q='language:rust stars:>1000'
gh api search/issues -f q='repo:{owner}/{repo} is:open label:bug'
gh api search/code -f q='filename:Cargo.toml repo:{owner}/{repo}'
```

### Notifications
```bash
gh api notifications                           # List notifications
gh api -X PUT notifications -F read=true       # Mark all read
```
