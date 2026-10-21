# engramme format v1

This is version 1 of the engramme vault format, validated by `engramme check`.
See the [README](../README.md) for setup and the [capture skill](../skill/SKILL.md)
for proposing cards from an agent.

## Vault layout

```text
<vault>/                              # ENGRAMME_DIR
  notes/                              # one note per topic; subdirectories allowed
    math/
      odds.md
  inbox/                              # proposals awaiting human triage
    2026-10-21T0930-codex-logit.md
  .app/
    log/2026-10.mac.jsonl              # versioned; append only
    log/2026-10.iphone.jsonl
    cache.sqlite                      # rebuildable; ignored by git
  .gitignore                          # contains at least .app/cache.sqlite
```

Set `ENGRAMME_DIR` to the vault directory. Journal files follow
`.app/log/<YYYY-MM>.<device>.jsonl`, one per UTC month and device.
Use UTF-8 files with LF line endings; CRLF is also accepted, and an initial
UTF-8 byte order mark (BOM) is ignored.

A vault check reads `notes/**/*.md` recursively and `inbox/*.md` directly in
`inbox/`. It ignores `inbox/archive/` and all other inbox subdirectories,
entries whose names start with `.`, files without the lowercase `.md`
extension, `.app/`, and files at the vault root. During traversal a symbolic
link counts only if it leads to a regular file; links to directories and
broken links are skipped. The top-level `notes/` and `inbox/` directories
themselves may be symbolic links to directories.

## Notes

Frontmatter is optional. It exists only when line 1 is `---` in column 0,
and ends at the **first** following `---` line in column 0; trailing spaces
and tabs on either delimiter are accepted. This split happens before YAML
parsing: even inside a block scalar, a column-zero `---` ends frontmatter.
An indented `---` can be scalar content. If no closing delimiter exists,
`invalid-frontmatter` points to line 1.

Empty, comment-only, or YAML `null` frontmatter is accepted as an empty
mapping. Otherwise the YAML must be a mapping; malformed YAML, duplicate
keys, or a non-mapping value produces `invalid-frontmatter`.

| Key | Type | Meaning |
| --- | --- | --- |
| `title` | String, optional; `null` means absent | Displayed title |
| `tags` | List of strings, optional | Classification and review filters |
| `sources` | List of strings, optional | References, paths, or URLs for the knowledge |

For `tags` and `sources`, `[]`, an empty YAML value, or `null` means an empty
list. An empty string is not a list. YAML 1.2 typing applies: `yes`, `no`, and
`on` remain strings, but numbers are not strings. Quote a numeric tag as
`"2024"`. A recognized key with the wrong type produces `invalid-frontmatter`
at its value and is ignored; for a block list or a `|` or `>` scalar, the
location is the first content line. Unknown keys are preserved and ignored.

The title is the non-empty, trimmed `title` value, otherwise the first
non-empty level-1 heading outside card blocks and fenced code, otherwise the
filename without `.md`. Such a heading starts with up to three spaces, then
`#` followed by a space or tab. A trailing run of `#` preceded by a space is
removed (`# Title ##` becomes `Title`, while `# C#` keeps `C#`).

The body is free Markdown (CommonMark and GFM), with LaTeX between `$…$` or
`$$…$$`. The parser preserves this content; rendering belongs to the app.
A note need not contain cards.

Complete file examples use `markdown file=<vault-relative-path>` fences.
The IDs below illustrate app-assigned IDs; omit IDs when writing new cards.
This note contains prose, two cloze numbers, a hint, LaTeX braces, and code:

````markdown file=notes/math/odds.md
---
title: Odds and log-odds
tags: [probability, "2026"]
sources:
  - Probability course notes, section 2
---

# Odds and log-odds

For a probability $0 < p < 1$, odds compare an event with its complement:

$$
o = \frac{p}{1 - p}.
$$

:::qa {#c-v7m2q9}
What do odds compare?
---
The probability of an event with the probability of its complement.
:::

:::cloze {#c-r8v4n2}
Odds are $o = {{c1::\frac{p}{1 - p}}}$.
Their natural logarithm is the {{c2::logit::name of the transformation}} of $p$.
:::

:::qa {#c-b6t9w3}
How do you compute odds from a probability $0 < p < 1$ in Python?
---
```python
odds = p / (1 - p)
```
:::
````

This note has no frontmatter, takes its title from `# Complement probability`,
and has a valid card without an ID:

```markdown file=notes/math/complement.md
# Complement probability

An event and its complement exhaust the possible outcomes.

:::qa
If an event has probability $p$, what is the probability of its complement?
---
$1 - p$.
:::
```

Diagnostic excerpts below are copied from real checks of the fragments as
standalone notes, using the filenames shown. For example, this numeric tag
is invalid:

```markdown
---
tags: [2024]
---
```

```text
numeric-tags.md:2: error: `tags` must be a list of strings [invalid-frontmatter]
```

## Card blocks

A question/answer fragment:

```markdown
:::qa
What do odds compare?
---
An event's probability with its complement's probability.
:::
```

Both delimiters start in column 0. The opening line is `:::` immediately
followed by the type: `::: qa` gives `invalid-block-header`. An optional
`{#<id>}` may follow the type, with or without intervening spaces or tabs;
no other suffix is allowed (`invalid-block-header`). The closing line is
`:::` alone. Trailing spaces and tabs on delimiter lines are tolerated.
Indented delimiters are plain text.

Types are lowercase: `qa` and `cloze` are implemented; `free` and `estimate`
are reserved, produce `reserved-block-type`, and yield no card. Any other
type, including `QA`, gives `unknown-block-type`.

```markdown
:::free
Explain why odds differ from probability.
:::
```

```text
reserved.md:1: warning: card type `free` is reserved for a future version: block ignored [reserved-block-type]
```

Blocks cannot nest. An unclosed block gives `unclosed-block` at its opening
line, including when another block opens before it closes; the new block is
then read separately. A stray `:::` outside a block is silently ignored.
Block parsing errors discard that block; other valid blocks can still be read.

Fenced code starts with at least three backticks or tildes, indented by at
most three spaces. It closes with at least as many of the same character,
also indented by at most three spaces, followed only by spaces or tabs.
A backtick opening fence cannot contain another backtick after its opening
run. Card delimiters and `---` separators inside fenced code are literal
content. An unclosed code fence outside a card gives the warning
`unclosed-code-fence`, and later card blocks are ignored. Inside a card,
the card instead receives `unclosed-block` at its opening line.

IDs have the form `c-` followed by 6 to 8 characters from `[0-9a-z]`;
other IDs give `malformed-id`. **Only the app assigns IDs** (subproject 2),
randomly rather than sequentially. Humans and agents must not assign them.
A card without an ID is valid. Reusing an ID among parsed note cards in a
vault gives `duplicate-id` on every occurrence after the first and cites the
first file and line. Checking a note alone sees only its own IDs.

History follows the ID: editing card text preserves its history. Deleting
the block removes the card from review; its history remains in the journal.

### qa

The front comes first, then the first `---` line outside fenced code, then
the back. This separator must be in column 0 and contain exactly three
dashes, with optional trailing spaces or tabs. Further `---` lines belong
to the back. The front cannot contain such a line outside code; use `***`
for a thematic break there. Both faces can span multiple Markdown lines;
blank lines at their edges are removed.

A missing separator gives `missing-separator` at the opening line. An empty
front gives `empty-face` at the opening line; an empty back gives it at the
separator line.

```markdown
:::qa
What are the odds of an event?
:::
```

```text
missing-separator.md:1: error: qa card has no `---` line between front and back [missing-separator]
```

### cloze

Write deletions as `{{cN::text}}` or `{{cN::text::hint}}`, where N is at least
1 with no leading zero (the parser accepts integers through 4294967295).
Deletions with the same N are hidden together; each distinct N produces a
sub-card `<id>.<N>` once the block has an ID.

The hint starts at the **first** `::` after the number separator, as in Anki:
`{{c1::std::mem::take}}` hides `std` with hint `mem::take`. Write
`std::mem::{{c1::take}}` to hide only `take`. An empty hint is treated as absent.

A deletion ends at the first `}}` with no unclosed inner braces. Balanced
LaTeX braces work: `{{c1::\frac{a}{b}}}` hides `\frac{a}{b}`. Deletions can
span lines and are recognized everywhere in the cloze body, including
LaTeX and code. A `{{c` not followed by digits and then `::` stays plain text.

`no-cloze-deletion` means no deletion was found. `invalid-cloze-deletion`
means empty or whitespace-only hidden text, `c0`, a leading zero such as
`c01`, a number too large for the parser, or a deletion never closed.

```markdown
:::cloze
The odds are {{c0::p / (1 - p)}}.
:::
```

```text
invalid-cloze.md:2: error: invalid cloze number `c0`: numbers start at 1, without leading zero [invalid-cloze-deletion]
```

Numbers should be consecutive from 1. Gaps give
`non-consecutive-cloze-numbers`, a warning that does not discard the card:

```markdown
:::cloze
The {{c1::odds}} are {{c3::p / (1 - p)}}.
:::
```

```text
cloze-gap.md:1: warning: cloze numbers are not consecutive: missing c2 [non-consecutive-cloze-numbers]
```

## Inbox proposals

Put one file per proposal directly in `inbox/`. Use
`<YYYY-MM-DD>T<HHMM>-<agent>-<topic>.md`: a real calendar date and time,
then at least two non-empty `[a-z0-9]+` words separated by `-`. If the name
exists, append `-2`, `-3`, and so on before `.md`. A nonconforming name gives
the warning `unexpected-file-name`.

Frontmatter is required, with the same delimiters and YAML rules as notes.
Without it, `missing-required-key` points to line 1. These five keys must
all be non-empty strings:

| Key | Meaning |
| --- | --- |
| `proposed_by` | Agent name, such as `claude`, `codex`, `copilot`, or `zcode` |
| `date` | Date and time of the proposal, in the form below |
| `context` | Current task, in one sentence |
| `target` | Vault-relative path of the note to extend or create |
| `why` | Why this knowledge is worth retaining |

A missing key gives `missing-required-key` at line 1; a null, empty, or
whitespace-only value gives it at that value's line (the first content line
for a block scalar). A value of the wrong type gives `invalid-frontmatter`.
Invalid required values are ignored.

`date` must be `YYYY-MM-DDTHH:MM`, optionally followed by `:SS` and a decimal
fraction of seconds, then optionally `Z` or `±HH:MM`. Dates must exist in
the Gregorian calendar; hours are 00–23, minutes and seconds 00–59, and
offset hours/minutes use those same ranges. For example,
`2026-10-21T07:30:00Z` and `2026-10-21T09:30:00.5+02:00` are accepted.
`2026-10-20 14:32`, `2026-10-20T14:32:00+0200`, and `2026-10-20` alone
give `invalid-frontmatter`. On BSD and GNU systems,
`date -u +%Y-%m-%dT%H:%M:%SZ` produces an accepted UTC value.

`target` must be relative, use `/` separators, start with `notes/`, and end
with `.md`. Empty, `.`, `..`, or hidden path components are forbidden, as
are absolute paths, drive prefixes, and backslashes (`invalid-target`).
The target note need not exist. Optional `tags` and `sources` are lists of
strings, as for notes, to be merged into the target note on acceptance.
Unknown keys are preserved.

The body is optional prose followed by 1 to 3 `qa` or `cloze` blocks, with
no IDs. No such block gives `no-card`; more than three gives `too-many-cards`
at the fourth opening line. This count includes `qa` and `cloze` openings
whose blocks have errors; reserved types do not count. An ID gives
`id-in-proposal`, and the card is omitted from the parsed proposal.

This complete proposal targets the first note above:

```markdown file=inbox/2026-10-21T0930-codex-logit.md
---
proposed_by: codex
date: 2026-10-21T07:30:00Z
context: Explaining how to convert between probability and log-odds
target: notes/math/odds.md
why: The inverse transformation helps interpret values on a log-odds scale
tags: [probability, transformations]
---

The inverse logit maps any finite real number to a probability between 0 and 1.

:::qa
What probability corresponds to log-odds of zero?
---
$p = 1/2$, because the odds are $e^0 = 1$.
:::

:::cloze
For log-odds $z$, the probability is $p = {{c1::\frac{1}{1 + e^{-z}}}}$.
:::
```

Triage is card by card: accept, edit then accept, or reject with a reason.
Before the app exists, copy the content you keep into the target note, then
delete the proposal file. Acceptance and metadata merging are manual at
this stage.

## Event log

`.app/log/<YYYY-MM>.<device>.jsonl` contains one JSON event per line,
timestamped in UTC. The UTC month of `ts` selects the file. Files are
append-only: never rewrite existing events.

`<device>` matches `[a-z0-9-]+`, for example `mac`, `iphone`, or `work-pc`.
Configure it locally on each device, **outside the vault**: the vault may
sync, but this setting does not. Each device writes only its own files, so
devices do not edit the same journal file during git or iCloud sync.

Every event has `ts` (ISO 8601 UTC timestamp) and `ev` (event type).

| `ev` | Additional fields | Producer |
| --- | --- | --- |
| `review` | `card` (ID or ID.N), `rating` (1 Again, 2 Hard, 3 Good, 4 Easy), `ms` (response duration in milliseconds) | App, subproject 2 |
| `suspend` / `unsuspend` | `card` | App |
| `triage` | `inbox_file`, `action` (`accept`, `edit`, `reject`), `reason` (on rejection), `proposed_by` | App |

Rejection reasons retain these exact values: `trivial` (too trivial), `faux`
(incorrect), `doublon` (duplicate), `trop-lie-au-code` (too tied to the code),
and `autre` (other).

Two illustrative lines in `.app/log/2026-10.mac.jsonl`:

```json
{"ts":"2026-10-21T09:40:00Z","ev":"review","card":"c-r8v4n2.1","rating":3,"ms":3800}
{"ts":"2026-10-21T09:41:00Z","ev":"triage","inbox_file":"2026-10-21T0930-codex-logit.md","action":"reject","reason":"doublon","proposed_by":"codex"}
```

Rebuild the SQLite cache by replaying all journal files across devices,
ordered by `ts`, then device name to break ties, then line order. Version 1
specifies this journal but currently produces no events; `engramme check`
does not read `.app/` or validate the journal.

## Checking a vault

Run `engramme check [<path>…]`. A directory is checked as a vault; a file is
checked as a proposal if its parent directory is named exactly `inbox`,
otherwise as a note. With no paths, `ENGRAMME_DIR` supplies the directory.
Explicit paths take precedence over that variable. Put `--` before a path
starting with `-`, as in `engramme check -- -draft.md`.

Diagnostics and the summary go to stdout; usage errors go to stderr.
Each diagnostic is one line:

```text
<file>:<line>: <error|warning>: <message> [<code>]
checked N files: E errors, W warnings
```

Line numbers start at 1 and include frontmatter. Paths are displayed as
given, with embedded line breaks replaced by spaces. Diagnostics are sorted
by file then line within each checked path; multiple paths are processed in
command-line order, not sorted together. Counts use the singular at 1:
`checked 1 file: 0 errors, 0 warnings`.

| Exit code | Meaning |
| --- | --- |
| 0 | No errors; warnings are allowed, including `not-a-vault` when no files were checked |
| 1 | At least one error, including an unreadable path |
| 2 | Checking did not start: invalid arguments, or no path and `ENGRAMME_DIR` is absent, empty, or not a directory |

From the repository root, this real run exits with 1:

```sh
cargo run -q -p engramme-cli -- check crates/engramme-cli/tests/fixtures/broken
```

```text
crates/engramme-cli/tests/fixtures/broken/inbox/2026-10-21T0905-claude-pd-horizon.md:8: error: card id `{#c-h4v9ke}` is not allowed in a proposal: remove it, only the app assigns ids [id-in-proposal]
crates/engramme-cli/tests/fixtures/broken/notes/pd.md:6: error: duplicate card id `c-7tq2lm`: first used at crates/engramme-cli/tests/fixtures/broken/notes/ead.md:3 [duplicate-id]
crates/engramme-cli/tests/fixtures/broken/notes/pd.md:12: error: qa card has no `---` line between front and back [missing-separator]
crates/engramme-cli/tests/fixtures/broken/notes/pd.md:17: warning: cloze numbers are not consecutive: missing c2 [non-consecutive-cloze-numbers]
checked 3 files: 3 errors, 1 warning
```

All 21 diagnostic codes:

| Code | Level | Cause |
| --- | --- | --- |
| `invalid-frontmatter` | Error | Unclosed frontmatter, invalid YAML, a non-mapping value other than null, a recognized key of the wrong type, or an invalid date-time |
| `unclosed-block` | Error | Card block never closed, including a new block opening before its closer or an unclosed code fence inside it |
| `invalid-block-header` | Error | Missing type immediately after `:::`, or an unsupported suffix after the type |
| `unknown-block-type` | Error | Type is neither implemented nor reserved |
| `malformed-id` | Error | ID is not `c-` followed by 6 to 8 lowercase ASCII letters or digits |
| `missing-separator` | Error | `qa` has no `---` separator outside fenced code |
| `empty-face` | Error | A `qa` front or back is empty after removing edge blank lines |
| `no-cloze-deletion` | Error | `cloze` contains no deletion candidate |
| `invalid-cloze-deletion` | Error | Deletion is empty, badly numbered, or unclosed |
| `unreadable-file` | Error | File or directory cannot be read, or file is not UTF-8; invalid UTF-8 is located at the line of the first invalid byte |
| `missing-required-key` | Error | Proposal lacks frontmatter, or a required key is absent, null, empty, or whitespace-only |
| `invalid-target` | Error | Proposal target is not an allowed relative `.md` path under `notes/` |
| `no-card` | Error | Proposal has no `qa` or `cloze` opening |
| `too-many-cards` | Error | Proposal has more than three `qa` or `cloze` openings; reported at the fourth |
| `id-in-proposal` | Error | A proposal block has an ID; only the app assigns IDs |
| `duplicate-id` | Error | A parsed note card repeats an earlier ID in the checked vault or lone note |
| `reserved-block-type` | Warning | `free` or `estimate` block is reserved and ignored |
| `non-consecutive-cloze-numbers` | Warning | Cloze numbers skip a value between 1 and the largest number |
| `unclosed-code-fence` | Warning | Code fence outside a card is never closed; later card blocks are ignored |
| `unexpected-file-name` | Warning | Proposal filename does not match the date, time, agent, and topic convention |
| `not-a-vault` | Warning | Directory has neither `notes/` nor `inbox/` |
