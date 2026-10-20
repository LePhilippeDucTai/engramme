use std::path::{Path, PathBuf};

use engramme_format::{
    Card, CardId, CardKind, ClozeDeletion, Diagnostic, DiagnosticKind, Note, Parsed, Severity,
    parse_note, read_note,
};

fn id(s: &str) -> CardId {
    CardId::parse(s).unwrap()
}

fn deletion(number: u32) -> ClozeDeletion {
    ClozeDeletion {
        number,
        text: "x".into(),
        hint: None,
    }
}

#[test]
fn card_id_accepts_valid_forms() {
    for s in ["c-abc123", "c-0000000", "c-zz99zz99"] {
        assert_eq!(
            CardId::parse(s).map(|i| i.as_str().to_owned()),
            Some(s.into())
        );
    }
}

#[test]
fn card_id_rejects_malformed_forms() {
    for s in [
        "c-abc12",
        "c-abc123456",
        "c-ABC123",
        "C-abc123",
        "c_abc123",
        "c-abcé12",
        "",
        "c-",
    ] {
        assert_eq!(CardId::parse(s), None, "{s:?}");
    }
}

#[test]
fn card_id_sub_card() {
    let i = id("c-91be0q");
    assert_eq!(i.sub_card(2), "c-91be0q.2");
    assert_eq!(i.to_string(), "c-91be0q");
}

#[test]
fn review_ids_of_qa_and_cloze() {
    let qa = Card {
        id: Some(id("c-91be0q")),
        line: 1,
        kind: CardKind::Qa {
            front: "f".into(),
            back: "b".into(),
        },
    };
    assert_eq!(qa.review_ids(), vec!["c-91be0q"]);
    let cloze = Card {
        id: Some(id("c-91be0q")),
        line: 1,
        kind: CardKind::Cloze {
            text: "t".into(),
            deletions: vec![deletion(2), deletion(1), deletion(2)],
        },
    };
    assert_eq!(cloze.review_ids(), vec!["c-91be0q.1", "c-91be0q.2"]);
    let anonymous = Card { id: None, ..cloze };
    assert!(anonymous.review_ids().is_empty());
}

#[test]
fn diagnostic_display_format() {
    let d = Diagnostic::new(
        Path::new("notes/a.md"),
        7,
        DiagnosticKind::UnclosedBlock,
        "card block `:::qa` is never closed: add a `:::` line",
    );
    assert_eq!(
        d.to_string(),
        "notes/a.md:7: error: card block `:::qa` is never closed: add a `:::` line [unclosed-block]"
    );
}

#[test]
fn diagnostic_severity_follows_kind() {
    let p = Path::new("a.md");
    assert_eq!(
        Diagnostic::new(p, 1, DiagnosticKind::MalformedId, "m").severity,
        Severity::Error
    );
    for kind in [
        DiagnosticKind::ReservedBlockType,
        DiagnosticKind::NonConsecutiveClozeNumbers,
        DiagnosticKind::UnclosedCodeFence,
    ] {
        assert_eq!(Diagnostic::new(p, 1, kind, "m").severity, Severity::Warning);
    }
}

#[test]
fn plain_note_has_no_cards() {
    let parsed = parse_note(
        Path::new("notes/risque-credit/lgd-downturn.md"),
        "Some prose.\n\nMore prose.\n",
    );
    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.value.cards.is_empty());
    assert_eq!(parsed.value.title, "lgd-downturn");
}

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "engramme-{}-{}-{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn read_note_reports_unreadable_files() {
    let missing = read_note(Path::new("/nonexistent/engramme/missing.md"));
    assert_eq!(missing.diagnostics.len(), 1);
    assert_eq!(missing.diagnostics[0].kind, DiagnosticKind::UnreadableFile);
    assert_eq!(missing.diagnostics[0].line, 1);
    assert!(
        missing.diagnostics[0]
            .message
            .starts_with("cannot read file: ")
    );

    let path = temp_file("bad.md", b"ok\n\xff\n");
    let bad = read_note(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(bad.diagnostics.len(), 1);
    assert_eq!(bad.diagnostics[0].kind, DiagnosticKind::UnreadableFile);
    assert_eq!(bad.diagnostics[0].line, 2);
    assert_eq!(bad.diagnostics[0].message, "file is not valid UTF-8");
    assert!(bad.value.cards.is_empty());
}

fn parse(lines: &[&str]) -> Parsed<Note> {
    parse_note(
        Path::new("notes/test.md"),
        &format!("{}\n", lines.join("\n")),
    )
}

fn codes(parsed: &Parsed<Note>) -> Vec<(usize, &'static str)> {
    parsed
        .diagnostics
        .iter()
        .map(|d| (d.line, d.kind.code()))
        .collect()
}

fn qa(front: &str, back: &str) -> CardKind {
    CardKind::Qa {
        front: front.into(),
        back: back.into(),
    }
}

fn qa_card(line: usize, front: &str, back: &str) -> Card {
    Card {
        id: None,
        line,
        kind: qa(front, back),
    }
}

fn cloze_text(card: &Card) -> &str {
    match &card.kind {
        CardKind::Cloze { text, .. } => text,
        CardKind::Qa { .. } => panic!("expected a cloze card: {card:?}"),
    }
}

#[test]
fn qa_card_with_id() {
    let parsed = parse(&[
        "Some prose.",
        "",
        ":::qa {#c-91be0q}",
        "What does LGD stand for?",
        "---",
        "Loss given default.",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [Card {
            id: Some(id("c-91be0q")),
            line: 3,
            kind: qa("What does LGD stand for?", "Loss given default."),
        }]
    );
}

#[test]
fn card_without_id_is_valid() {
    let parsed = parse(&[
        ":::qa",
        "Q",
        "---",
        "A",
        ":::",
        ":::cloze",
        "The {{c1::PD}} is a probability.",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    let cards = &parsed.value.cards;
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0], qa_card(1, "Q", "A"));
    assert_eq!(cards[1].id, None);
    assert_eq!(cards[1].line, 6);
    assert_eq!(cloze_text(&cards[1]), "The {{c1::PD}} is a probability.");
}

#[test]
fn qa_first_separator_outside_code_wins() {
    let parsed = parse(&[
        ":::qa",
        "Front",
        "```",
        "---",
        "```",
        "---",
        "Back",
        "---",
        "More back",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [qa_card(1, "Front\n```\n---\n```", "Back\n---\nMore back")]
    );
}

#[test]
fn qa_faces_are_multiline_markdown() {
    let equation = r"\mathrm{EL} = \mathrm{PD} \times \mathrm{LGD} \times \mathrm{EAD}";
    let parsed = parse(&[
        ":::qa",
        "  ",
        "State the expected loss.",
        "",
        "$$",
        equation,
        "$$",
        "",
        "---",
        "",
        "In code:",
        "",
        "```python",
        "el = pd * lgd * ead",
        "```",
        "  ",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [qa_card(
            1,
            &format!("State the expected loss.\n\n$$\n{equation}\n$$"),
            "In code:\n\n```python\nel = pd * lgd * ead\n```",
        )]
    );
}

#[test]
fn qa_missing_separator() {
    let parsed = parse(&[":::qa", "Front only", ":::"]);
    assert_eq!(codes(&parsed), [(1, "missing-separator")]);
    assert!(parsed.value.cards.is_empty());
    // Without a separator, faces are not checked.
    assert_eq!(codes(&parse(&[":::qa", ":::"])), [(1, "missing-separator")]);
}

#[test]
fn qa_separator_inside_code_does_not_count() {
    let parsed = parse(&[":::qa", "Front", "~~~", "---", "~~~", "Back", ":::"]);
    assert_eq!(codes(&parsed), [(1, "missing-separator")]);
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn qa_empty_front() {
    let parsed = parse(&[":::qa", "", "   ", "---", "Back", ":::"]);
    assert_eq!(codes(&parsed), [(1, "empty-face")]);
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn qa_empty_back() {
    let parsed = parse(&["Prose.", ":::qa", "Front", "---", "", ":::"]);
    assert_eq!(codes(&parsed), [(4, "empty-face")]);
    assert!(parsed.value.cards.is_empty());
    let both = parse(&[":::qa", "---", ":::"]);
    assert_eq!(codes(&both), [(1, "empty-face"), (2, "empty-face")]);
}

#[test]
fn closing_line_inside_backtick_fence_is_not_a_closer() {
    let parsed = parse(&[
        ":::qa",
        "Front",
        "---",
        "```markdown",
        "~~~",
        ":::",
        "```",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [qa_card(1, "Front", "```markdown\n~~~\n:::\n```")]
    );
}

#[test]
fn closing_line_inside_tilde_fence_is_not_a_closer() {
    let parsed = parse(&[
        ":::cloze",
        "The {{c1::PD}} is a probability.",
        "~~~",
        "```",
        ":::",
        "~~~",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.value.cards.len(), 1);
    assert_eq!(
        cloze_text(&parsed.value.cards[0]),
        "The {{c1::PD}} is a probability.\n~~~\n```\n:::\n~~~"
    );
}

#[test]
fn shorter_fence_does_not_close_longer_one() {
    let parsed = parse(&[":::qa", "Front", "---", "````", "```", ":::", "````", ":::"]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [qa_card(1, "Front", "````\n```\n:::\n````")]
    );
}

#[test]
fn opening_line_inside_code_fence_is_ignored() {
    let parsed = parse(&[
        "Prose.",
        "   ```markdown",
        ":::qa",
        "Q",
        "---",
        "A",
        ":::",
        "  ```",
        "~~~",
        ":::cloze {#c-abc123}",
        "~~~",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn indented_delimiters_are_plain_text() {
    let outside = parse(&["  :::qa", "Q", "---", "A", "  :::"]);
    assert!(outside.diagnostics.is_empty());
    assert!(outside.value.cards.is_empty());
    let inside = parse(&[
        ":::qa", "Q", "---", "A", "  :::", " :::qa", "    ```", ":::",
    ]);
    assert!(inside.diagnostics.is_empty());
    assert_eq!(
        inside.value.cards,
        [qa_card(1, "Q", "A\n  :::\n :::qa\n    ```")]
    );
}

#[test]
fn trailing_whitespace_on_delimiters_is_tolerated() {
    let parsed = parse(&[
        ":::qa {#c-abc123} \t",
        "Q",
        "--- \t",
        "A",
        "```",
        "code",
        "``` \t",
        "::: \t",
        ":::cloze\t",
        "The {{c1::PD}} is a probability.",
        ":::  ",
    ]);
    assert!(parsed.diagnostics.is_empty());
    let cards = &parsed.value.cards;
    assert_eq!(cards.len(), 2);
    assert_eq!(
        cards[0],
        Card {
            id: Some(id("c-abc123")),
            line: 1,
            kind: qa("Q", "A\n```\ncode\n``` \t"),
        }
    );
    assert_eq!(cloze_text(&cards[1]), "The {{c1::PD}} is a probability.");
}

#[test]
fn unclosed_block_at_eof() {
    let parsed = parse(&[
        ":::qa",
        "Q",
        "---",
        "A",
        ":::",
        "",
        ":::qa {#c-abc123}",
        "Q",
        "---",
        "A",
    ]);
    assert_eq!(codes(&parsed), [(7, "unclosed-block")]);
    assert_eq!(parsed.value.cards, [qa_card(1, "Q", "A")]);
}

#[test]
fn unclosed_block_mentions_open_fence() {
    let parsed = parse(&[":::qa", "Q", "---", "```", "code", ":::"]);
    assert_eq!(codes(&parsed), [(1, "unclosed-block")]);
    assert!(
        parsed.diagnostics[0]
            .message
            .ends_with("(the code fence opened on line 4 is still open)")
    );
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn nested_block_reports_unclosed_outer() {
    let parsed = parse(&[
        ":::qa",
        "Outer front",
        ":::cloze",
        "The {{c1::PD}} is a probability.",
        ":::",
        ":::",
    ]);
    assert_eq!(codes(&parsed), [(1, "unclosed-block")]);
    let cards = &parsed.value.cards;
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].line, 3);
    assert_eq!(cloze_text(&cards[0]), "The {{c1::PD}} is a probability.");
}

#[test]
fn unknown_type_is_an_error() {
    let parsed = parse(&[
        ":::note",
        "# Inside an unknown block",
        ":::",
        ":::QA",
        "Q",
        "---",
        "A",
        ":::",
        ":::qa",
        "Q",
        "---",
        "A",
        ":::",
    ]);
    assert_eq!(
        codes(&parsed),
        [(1, "unknown-block-type"), (4, "unknown-block-type")]
    );
    assert_eq!(parsed.value.cards, [qa_card(9, "Q", "A")]);
    assert_eq!(parsed.value.title, "test");
}

#[test]
fn reserved_types_warn_and_yield_no_card() {
    let parsed = parse(&[
        ":::free",
        "Explain IFRS 9 staging.",
        ":::",
        ":::estimate {#c-abc123}",
        "How many defaults per year?",
        ":::",
    ]);
    assert_eq!(
        codes(&parsed),
        [(1, "reserved-block-type"), (4, "reserved-block-type")]
    );
    assert!(
        parsed
            .diagnostics
            .iter()
            .all(|d| d.severity == Severity::Warning)
    );
    assert!(!parsed.has_errors());
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn invalid_block_header() {
    for header in ["::: qa", ":::qa junk", ":::qa {.x}"] {
        // The body has no `---`: it is not parsed.
        let parsed = parse(&[header, "Front only", ":::"]);
        assert_eq!(codes(&parsed), [(1, "invalid-block-header")], "{header}");
        assert!(parsed.value.cards.is_empty());
    }
}

#[test]
fn id_may_follow_type_without_space() {
    let parsed = parse(&[":::qa{#c-abc123}", "Q", "---", "A", ":::"]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(
        parsed.value.cards,
        [Card {
            id: Some(id("c-abc123")),
            line: 1,
            kind: qa("Q", "A"),
        }]
    );
}

#[test]
fn malformed_id_is_an_error() {
    for suite in ["{#c-12}", "{#C-ABC123}", "{#}"] {
        let header = format!(":::qa {suite}");
        let parsed = parse(&[header.as_str(), "Q", "---", "A", ":::"]);
        assert_eq!(codes(&parsed), [(1, "malformed-id")], "{suite}");
        assert!(parsed.value.cards.is_empty());
    }
    let parsed = parse(&[":::qa {#c-12}", "Front only", ":::"]);
    assert_eq!(
        codes(&parsed),
        [(1, "malformed-id"), (1, "missing-separator")]
    );
}

#[test]
fn stray_closing_line_is_ignored() {
    let parsed = parse(&[
        ":::", "Prose.", "::: ", ":::qa", "Q", "---", "A", ":::", ":::",
    ]);
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.value.cards, [qa_card(4, "Q", "A")]);
}

#[test]
fn unclosed_code_fence_warns() {
    let parsed = parse(&[
        ":::qa", "Q", "---", "A", ":::", "Prose.", "```", ":::qa", "Q", "---", "A", ":::",
    ]);
    assert_eq!(codes(&parsed), [(7, "unclosed-code-fence")]);
    assert!(!parsed.has_errors());
    assert_eq!(parsed.value.cards, [qa_card(1, "Q", "A")]);
}

#[test]
fn title_from_first_heading() {
    let parsed = parse(&[
        "```",
        "# Not the title: code",
        "```",
        ":::qa",
        "# Not the title: card",
        "---",
        "A",
        ":::",
        "#tag",
        "## Section",
        "    # Not the title: indented",
        "# ",
        "# Real title #",
        "# Second title",
    ]);
    assert_eq!(parsed.value.title, "Real title");
    for (line, title) in [
        ("   #\tTitle", "Title"),
        ("# C#", "C#"),
        ("# Title ##  ", "Title"),
    ] {
        assert_eq!(parse(&[line]).value.title, title, "{line:?}");
    }
}

#[test]
fn title_from_file_stem() {
    let parsed = parse_note(
        Path::new("notes/risque-credit/ifrs9-staging.md"),
        "#tag\n## Section\n#\n# \n# #\nText # not a heading\n",
    );
    assert_eq!(parsed.value.title, "ifrs9-staging");
}

#[test]
fn latex_and_markdown_do_not_trigger_diagnostics() {
    let parsed = parse(&[
        "# Expected loss",
        "",
        r"Inline $\mathrm{EL} = \mathrm{PD} \times \mathrm{LGD}$, {braces} and {{c1::text}}.",
        "",
        "$$",
        r"\frac{a}{b} = \sum_{i=1}^{n} x_{i}",
        "$$",
        "",
        "| Stage | Horizon |",
        "|---|---|",
        "| 1 | 12 months |",
        "",
        "---",
        "",
        "- `:::qa` and ```:::``` in code spans",
        "```js``` is inline code, not a fence",
        "> A quote with #tag and ~~strike~~.",
        "<!-- ::: -->",
        "",
        ":::qa {#c-7k2m9x}",
        r"What is $\mathrm{EAD}$ for a {revolving} facility?",
        "---",
        r"$\mathrm{EAD} = \mathrm{drawn} + \mathrm{CCF} \times \mathrm{undrawn}$",
        "",
        "| CCF | Value |",
        "|---|---|",
        ":::",
    ]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.value.title, "Expected loss");
    assert_eq!(
        parsed.value.cards,
        [Card {
            id: Some(id("c-7k2m9x")),
            line: 20,
            kind: qa(
                r"What is $\mathrm{EAD}$ for a {revolving} facility?",
                concat!(
                    r"$\mathrm{EAD} = \mathrm{drawn} + \mathrm{CCF} \times \mathrm{undrawn}$",
                    "\n\n| CCF | Value |\n|---|---|"
                ),
            ),
        }]
    );
}

#[test]
fn diagnostics_are_sorted_by_line() {
    let parsed = parse(&[
        ":::QA",
        ":::",
        ":::qa {#bad}",
        "Front only",
        ":::",
        ":::qa",
        "Front",
        "---",
        ":::",
        ":::free",
        ":::",
        ":::qa",
        ":::qa",
    ]);
    assert!(parsed.diagnostics.is_sorted_by_key(|d| d.line));
    assert_eq!(
        codes(&parsed),
        [
            (1, "unknown-block-type"),
            (3, "malformed-id"),
            (3, "missing-separator"),
            (8, "empty-face"),
            (10, "reserved-block-type"),
            (12, "unclosed-block"),
            (13, "unclosed-block"),
        ]
    );
}

#[test]
fn structure_messages() {
    let cases: [(&[&str], &str); 12] = [
        (
            &[":::qa {#c-abc123} ", "Q"],
            "notes/test.md:1: error: card block `:::qa {#c-abc123}` is never closed: add a `:::` line [unclosed-block]",
        ),
        (
            &[":::qa", "Q", "---", "```"],
            "notes/test.md:1: error: card block `:::qa` is never closed: add a `:::` line (the code fence opened on line 4 is still open) [unclosed-block]",
        ),
        (
            &[":::qa", "Q", ":::qa", "Q", "---", "A", ":::"],
            "notes/test.md:1: error: card block `:::qa` is not closed before the block on line 3: blocks cannot be nested [unclosed-block]",
        ),
        (
            &["::: qa", ":::"],
            "notes/test.md:1: error: expected a card type right after `:::`, e.g. `:::qa` [invalid-block-header]",
        ),
        (
            &[":::qa junk", ":::"],
            "notes/test.md:1: error: unexpected `junk` after `:::qa`: only `{#<id>}` may follow the type [invalid-block-header]",
        ),
        (
            &[":::qa {#c-12}", "Q", "---", "A", ":::"],
            "notes/test.md:1: error: malformed card id `c-12`: expected `c-` followed by 6 to 8 characters in [0-9a-z] [malformed-id]",
        ),
        (
            &[":::note", ":::"],
            "notes/test.md:1: error: unknown card type `note`: expected `qa` or `cloze` [unknown-block-type]",
        ),
        (
            &[":::free", ":::"],
            "notes/test.md:1: warning: card type `free` is reserved for a future version: block ignored [reserved-block-type]",
        ),
        (
            &[":::qa", "Q", ":::"],
            "notes/test.md:1: error: qa card has no `---` line between front and back [missing-separator]",
        ),
        (
            &[":::qa", "---", "A", ":::"],
            "notes/test.md:1: error: qa card has an empty front [empty-face]",
        ),
        (
            &[":::qa", "Q", "---", ":::"],
            "notes/test.md:3: error: qa card has an empty back [empty-face]",
        ),
        (
            &["Prose.", "```"],
            "notes/test.md:2: warning: code fence opened here is never closed: card blocks after it are ignored [unclosed-code-fence]",
        ),
    ];
    for (lines, expected) in cases {
        let messages: Vec<String> = parse(lines)
            .diagnostics
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(messages, [expected], "{lines:?}");
    }
}

fn deletions(card: &Card) -> Vec<(u32, &str, Option<&str>)> {
    match &card.kind {
        CardKind::Cloze { deletions, .. } => deletions
            .iter()
            .map(|d| (d.number, d.text.as_str(), d.hint.as_deref()))
            .collect(),
        CardKind::Qa { .. } => panic!("expected a cloze card: {card:?}"),
    }
}

fn cloze_note(body: &[&str]) -> Parsed<Note> {
    let mut lines = vec![":::cloze {#c-91be0q}"];
    lines.extend_from_slice(body);
    lines.push(":::");
    parse(&lines)
}

#[test]
fn cloze_deletion_with_and_without_hint() {
    let parsed = cloze_note(&["{{c1::PD}} and {{c2::LGD::loss given default}}"]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        deletions(&parsed.value.cards[0]),
        vec![(1, "PD", None), (2, "LGD", Some("loss given default"))]
    );
    let blank_hint = cloze_note(&["{{c1::PD::  }}"]);
    assert_eq!(deletions(&blank_hint.value.cards[0]), vec![(1, "PD", None)]);
}

#[test]
fn cloze_hint_splits_at_first_double_colon() {
    let parsed = cloze_note(&["Use {{c1::std::mem::take}}."]);
    assert_eq!(
        deletions(&parsed.value.cards[0]),
        vec![(1, "std", Some("mem::take"))]
    );
}

#[test]
fn cloze_same_number_is_one_sub_card() {
    let parsed = cloze_note(&["{{c1::a}} {{c2::b}} {{c1::c}}"]);
    let card = &parsed.value.cards[0];
    assert_eq!(
        deletions(card),
        vec![(1, "a", None), (2, "b", None), (1, "c", None)]
    );
    assert_eq!(card.review_ids(), vec!["c-91be0q.1", "c-91be0q.2"]);
}

#[test]
fn cloze_inside_latex_with_braces() {
    let parsed = cloze_note(&[
        r"$\mathrm{EL} = {{c1::PD}} \times \mathrm{LGD}$",
        r"$x = {{c2::\frac{a}{b}}}$ and $y = {{c3::\hat{x}}}}$",
    ]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(
        deletions(&parsed.value.cards[0]),
        vec![
            (1, "PD", None),
            (2, r"\frac{a}{b}", None),
            (3, r"\hat{x}", None)
        ]
    );
    assert!(cloze_text(&parsed.value.cards[0]).ends_with(r"{{c3::\hat{x}}}}$"));
}

#[test]
fn cloze_inside_code_counts() {
    let parsed = cloze_note(&["```rust", "let x = {{c1::std::mem::take}}(&mut v);", "```"]);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(deletions(&parsed.value.cards[0]).len(), 1);
}

#[test]
fn cloze_multiline_deletion() {
    let parsed = cloze_note(&["{{c1::first", "second}}"]);
    assert_eq!(
        deletions(&parsed.value.cards[0]),
        vec![(1, "first\nsecond", None)]
    );
}

#[test]
fn cloze_without_deletion() {
    for body in ["Plain text", "{{c::x}} {{C1::x}} {{c1:x}}"] {
        let parsed = cloze_note(&[body]);
        assert_eq!(codes(&parsed), vec![(1, "no-cloze-deletion")], "{body}");
        assert!(parsed.value.cards.is_empty());
    }
}

#[test]
fn cloze_empty_deletion() {
    for body in ["{{c1::}}", "{{c1::  }}", "{{c1::::hint}}"] {
        let parsed = cloze_note(&[body]);
        assert_eq!(
            codes(&parsed),
            vec![(2, "invalid-cloze-deletion")],
            "{body}"
        );
        assert!(parsed.value.cards.is_empty());
    }
}

#[test]
fn cloze_number_must_be_positive_without_leading_zero() {
    for body in ["{{c0::x}}", "{{c01::x}}", "{{c99999999999::x}}"] {
        let parsed = cloze_note(&[body]);
        assert_eq!(
            codes(&parsed),
            vec![(2, "invalid-cloze-deletion")],
            "{body}"
        );
        assert!(parsed.value.cards.is_empty());
    }
}

#[test]
fn cloze_unterminated_deletion() {
    let parsed = cloze_note(&["{{c1::open", "{{c2::also open"]);
    assert_eq!(codes(&parsed), vec![(2, "invalid-cloze-deletion")]);
    assert!(parsed.value.cards.is_empty());
}

#[test]
fn cloze_non_consecutive_numbers_warn() {
    let parsed = cloze_note(&["{{c1::a}} {{c3::b}}"]);
    assert_eq!(codes(&parsed), vec![(1, "non-consecutive-cloze-numbers")]);
    assert_eq!(
        parsed.diagnostics[0].message,
        "cloze numbers are not consecutive: missing c2"
    );
    assert_eq!(parsed.value.cards.len(), 1);
    let only_two = cloze_note(&["{{c2::a}}"]);
    assert_eq!(
        only_two.diagnostics[0].message,
        "cloze numbers are not consecutive: missing c1"
    );
    assert_eq!(only_two.value.cards.len(), 1);
}

#[test]
fn cloze_diagnostic_lines() {
    let parsed = cloze_note(&["", "  ", "{{c1::a}}", "text", "and {{c2::}}"]);
    assert_eq!(codes(&parsed), vec![(6, "invalid-cloze-deletion")]);
}

#[test]
fn cloze_messages() {
    let cases: [(&str, &str); 4] = [
        (
            "Plain text",
            "notes/test.md:1: error: cloze card has no deletion such as `{{c1::text}}` [no-cloze-deletion]",
        ),
        (
            "{{c1::open",
            "notes/test.md:2: error: cloze deletion `{{c1::` is never closed with `}}` [invalid-cloze-deletion]",
        ),
        (
            "{{c01::x}}",
            "notes/test.md:2: error: invalid cloze number `c01`: numbers start at 1, without leading zero [invalid-cloze-deletion]",
        ),
        (
            "{{c1::  }}",
            "notes/test.md:2: error: cloze deletion `{{c1::…}}` has empty text [invalid-cloze-deletion]",
        ),
    ];
    for (body, expected) in cases {
        let parsed = cloze_note(&[body]);
        assert_eq!(parsed.diagnostics.len(), 1, "{body}");
        assert_eq!(parsed.diagnostics[0].to_string(), expected);
    }
}
