---
name: engramme-capture
description: Proposes flashcards to the user's engramme vault so that they retain, without AI help, the knowledge used in a task. Use at the end of a task that relied on non-trivial knowledge (a modelling choice and its rationale, an equation, an order of magnitude, a product specificity, the root cause of a bug, an implementation pitfall), not after every exchange. Writes 1 to 3 cards per session, only to $ENGRAMME_DIR/inbox/, for the user to triage.
---

# engramme-capture

Turn the knowledge a task relied on into flashcard proposals in the user's
engramme vault, so they keep it without AI help: you propose, the user triages.

## 1. When

Act at the end of a task that relied on non-trivial knowledge, not after every
exchange. Propose at most 3 cards per session, counting your earlier proposals
in the session. If nothing deserves to be remembered, propose nothing and say
nothing about it: that is a normal outcome.

## 2. What

Propose what the user should know without AI help: a modelling choice and its
rationale, an equation, an order of magnitude, a product specificity, the root
cause of a bug, an implementation pitfall. Never propose file paths, API
details that take seconds to look up, or ephemeral information.

## 3. Read before writing

Search `$ENGRAMME_DIR/notes/` (titles, `tags`, content, for example with
`grep -rli`) and the pending files of `$ENGRAMME_DIR/inbox/`. Do not propose
again what is already there. Set `target` to the existing note that fits,
otherwise to a new path `notes/<area>/<topic>.md`, in lowercase words joined
by `-`.

## 4. Writing cards

Put one idea per card. Ask a question that makes sense without the code at
hand, and give a short answer. Write math in LaTeX: `$…$` inline, `$$…$$` for
display. Prefer `qa`; use `cloze` for a formula or a key term.

## 5. Never write

Never write secrets, credentials (passwords, tokens, keys, connection strings),
personal data, client or counterparty names, or raw data values; round orders
of magnitude ("about 2%"). If the knowledge cannot be written without them,
propose nothing.

## 6. Where to write

Write only in `$ENGRAMME_DIR/inbox/`, and create it if it is missing. Never
create, edit or delete anything in `notes/`, `.app/` or anywhere else. Never
write a card ID `{#…}`. If `ENGRAMME_DIR` is not set or is not a directory
(`test -d "$ENGRAMME_DIR"`; PowerShell, untested:
`Test-Path -PathType Container $env:ENGRAMME_DIR`), write nothing and say in
one line: `engramme-capture: ENGRAMME_DIR is not set or not a directory, nothing proposed.`
Write one file per target note, and never overwrite a file.

## 7. After writing

If the `engramme` command exists, run `engramme check <file>`, fix the file and
run the check again until it reports `0 errors, 0 warnings`. Then announce the
proposal in one line, for example:
`engramme-capture: proposed 2 cards in inbox/2026-10-21T0930-claude-logit-clipping.md`

## Proposal file

This section is enough to write a valid file; the full format is at
https://github.com/LePhilippeDucTai/engramme/blob/main/docs/format.md.

Name the file `<YYYY-MM-DD>T<HHMM>-<agent>-<topic>.md`, with the prefix from
`date +%Y-%m-%dT%H%M` in local time (PowerShell, untested:
`Get-Date -Format "yyyy-MM-dd'T'HHmm"`). `<agent>` is your name in lowercase
(`claude`, `codex`, `copilot`, `zcode`), the same as `proposed_by`; `<topic>`
is 1 to 4 `[a-z0-9]` words joined by `-`. If the file exists, add `-2`, `-3`,
and so on before `.md`.

Start with a frontmatter between `---` lines. Required keys, as non-empty strings:

- `proposed_by`: your agent name.
- `date`: the current UTC time, from `date -u +%Y-%m-%dT%H:%M:%SZ` (PowerShell,
  untested: `(Get-Date).ToUniversalTime().ToString("yyyy-MM-dd'T'HH:mm:ss'Z'")`);
  an offset without a colon, such as `+0200`, is rejected.
- `context`: the task, in one sentence.
- `target`: the note to extend or create (rule 3).
- `why`: why this knowledge is worth remembering.

`tags` and `sources` are optional lists of strings, such as `[pd, "2024"]`.
Double-quote a value or list item that contains `: ` or ` #` (or any of `,[]{}`
inside `[…]`), ends with `:`, starts with one of `-?:,[]{}#&*!|>'"%@` or a
backtick, or reads as a number, `true`, `false` or `null`. Inside double
quotes, write `\\` for `\` and `\"` for `"`.

After the frontmatter, write optional short prose, then 1 to 3 card blocks
whose `:::` lines start in column 0, with nothing after the type:

- `qa`: the front, a `---` line, then the back; no `---` line in the front.
- `cloze`: `{{c1::hidden}}` or `{{c1::hidden::hint}}`, numbered from 1 without gaps;
  deletions sharing a number are hidden together; hidden text cannot contain `::`.

Example:

```markdown file=inbox/2026-10-21T0930-claude-logit-clipping.md
---
proposed_by: claude
date: 2026-10-21T07:30:00Z
context: Calibrating a logistic PD model on a portfolio with very few defaults
target: notes/credit-risk/probability-of-default.md
why: Predicted PDs of exactly 0 or 1 make the log-odds infinite and break the calibration
tags: [pd, logistic-regression]
---

A logistic PD model works on the log-odds of the PD, which are infinite at 0 and 1.

:::cloze
The log-odds of a PD $p$ are $\operatorname{logit}(p) = {{c1::\log \frac{p}{1 - p}}}$.
:::

:::qa
Why clip PDs to $[\varepsilon, 1 - \varepsilon]$ before taking their logit?
---
Because $\operatorname{logit}(0) = -\infty$ and $\operatorname{logit}(1) = +\infty$.
:::
```
