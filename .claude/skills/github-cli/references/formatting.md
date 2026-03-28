# Output Formatting — `--json`, `--jq`, `--template`

Many `gh` commands support structured output via three flags that work together.

## `--json`

Converts output to JSON. Requires a comma-separated list of field names:

```bash
gh pr list --json number,title,author
gh issue view 123 --json title,body,labels
```

To see available fields for a command, pass `--json` without a value:
```bash
gh pr list --json   # Prints available field names
```

## `--jq`

Filter and transform JSON using jq syntax. Requires `--json` to be set first. The `jq` binary does NOT need to be installed — gh has a built-in jq evaluator.

**Selecting fields:**
```bash
gh pr list --json author --jq '.[].author.login'
gh pr view 123 --json title --jq '.title'
```

**Filtering arrays:**
```bash
# Issues with labels
gh issue list --json number,title,labels --jq \
  '[.[] | select(.labels | length > 0)]'

# PRs by specific author
gh pr list --json number,title,author --jq \
  '[.[] | select(.author.login == "monalisa")]'
```

**Transforming output:**
```bash
# Custom format
gh pr list --json number,title --jq '.[] | "#\(.number) \(.title)"'

# TSV output
gh pr list --json number,title --jq '.[] | [.number, .title] | @tsv'

# CSV output
gh pr list --json number,title --jq '.[] | [.number, .title] | @csv'
```

**Aggregation:**
```bash
# Count
gh pr list --json number --jq 'length'

# Sort
gh repo list --json name,stargazerCount --jq 'sort_by(.stargazerCount) | reverse'

# Group by
gh issue list --json state --jq 'group_by(.state) | map({state: .[0].state, count: length})'
```

**Extracting nested data:**
```bash
# Label names from issues
gh issue list --json number,title,labels --jq \
  'map(.labels = (.labels | map(.name)))'

# PR review decisions
gh pr list --json number,reviewDecision --jq \
  '.[] | select(.reviewDecision == "APPROVED") | .number'
```

## `--template`

Format output using Go templates. Requires `--json` to be set first.

### Basic syntax

```bash
# Simple iteration
gh pr list --json number,title --template \
  '{{range .}}#{{.number}} {{.title}}{{"\n"}}{{end}}'
```

### Built-in template functions

| Function | Description | Example |
|----------|-------------|---------|
| `color <style> <input>` | Colorize text | `{{.title \| color "green"}}` |
| `autocolor <style> <input>` | Color only in terminals | `{{.title \| autocolor "green"}}` |
| `join <sep> <list>` | Join list items | `{{.labels \| join ", "}}` |
| `pluck <field> <list>` | Extract field from list items | `{{.labels \| pluck "name"}}` |
| `tablerow <fields>...` | Align as table columns | `{{tablerow .number .title}}` |
| `tablerender` | Render accumulated table rows | `{{tablerender}}` |
| `timeago <time>` | Relative timestamp | `{{timeago .updatedAt}}` |
| `timefmt <format> <time>` | Format timestamp | `{{timefmt "2006-01-02" .createdAt}}` |
| `truncate <len> <input>` | Truncate to length | `{{truncate 50 .title}}` |
| `hyperlink <url> <text>` | Terminal hyperlink | `{{hyperlink .url .title}}` |

### Sprig functions

Also available: `contains`, `hasPrefix`, `hasSuffix`, `regexMatch` from the Sprig library.

### Table formatting

```bash
# Simple table
gh pr list --json number,title,headRefName,updatedAt --template \
  '{{range .}}{{tablerow (printf "#%v" .number | autocolor "green") .title .headRefName (timeago .updatedAt)}}{{end}}'

# Table with headers
gh pr view 123 --json assignees,reviews --template \
  '{{tablerow "ASSIGNEE" "NAME"}}{{range .assignees}}{{tablerow .login .name}}{{end}}{{tablerender}}
{{tablerow "REVIEWER" "STATE"}}{{range .reviews}}{{tablerow .author.login .state}}{{end}}'
```

### Complex example

```bash
gh pr view 123 --json number,title,body,reviews,assignees --template '
{{printf "#%v" .number}} {{.title}}

{{.body}}

{{tablerow "ASSIGNEE" "NAME"}}
{{- range .assignees}}{{tablerow .login .name}}{{end}}
{{- tablerender}}

{{tablerow "REVIEWER" "STATE" "COMMENT"}}
{{- range .reviews}}{{tablerow .author.login .state .body}}{{end}}'
```

### Hyperlinks

```bash
gh issue list --json title,url --template \
  '{{range .}}{{hyperlink .url .title}}{{"\n"}}{{end}}'
```

## Color styles

For the `color` and `autocolor` functions, styles use [mgutz/ansi](https://github.com/mgutz/ansi) syntax:

- Basic: `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `black`
- Bold: `red+b`, `green+b`
- Underline: `red+u`
- Background: `red:white` (red text on white background)

## Piping to jq

When output is piped (not a TTY), JSON is compact. For pretty-printing:

```bash
gh pr view 123 --json title,body | jq .
```

When connected to a terminal, `--jq` output is automatically pretty-printed.
