use std::path::{Path, PathBuf};

use engramme_format::{CardKind, InboxProposal, Parsed, parse_proposal, read_proposal};

const PATH: &str = "inbox/2026-10-20T1432-claude-test.md";

const KEYS: [&str; 5] = [
    "proposed_by: claude",
    "date: 2026-10-20T14:32:00+02:00",
    "context: Reviewing the LGD model",
    "target: notes/credit-risk/lgd.md",
    "why: The downturn adjustment is easy to forget",
];

const QA: [&str; 5] = [":::qa", "Q", "---", "A", ":::"];

fn parse(lines: &[&str]) -> Parsed<InboxProposal> {
    parse_proposal(Path::new(PATH), &format!("{}\n", lines.join("\n")))
}

/// A proposal whose frontmatter holds the five keys, with `key` replaced by
/// `line` (or removed when `line` is empty), followed by a valid `qa`.
fn with_key(key: &str, line: &str) -> Parsed<InboxProposal> {
    let mut lines = vec!["---"];
    for k in KEYS {
        if k.starts_with(&format!("{key}:")) {
            if !line.is_empty() {
                lines.push(line);
            }
        } else {
            lines.push(k);
        }
    }
    lines.push("---");
    lines.extend_from_slice(&QA);
    parse(&lines)
}

fn codes(parsed: &Parsed<InboxProposal>) -> Vec<(usize, &'static str)> {
    parsed
        .diagnostics
        .iter()
        .map(|d| (d.line, d.kind.code()))
        .collect()
}

fn messages(parsed: &Parsed<InboxProposal>) -> Vec<String> {
    parsed.diagnostics.iter().map(|d| d.to_string()).collect()
}

#[test]
fn valid_proposal() {
    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS);
    lines.extend_from_slice(&[
        "---",
        "Some prose to add to the note.",
        "",
        ":::qa",
        "Why use a downturn LGD?",
        "---",
        "Capital must cover losses in a downturn.",
        ":::",
        "",
        ":::cloze",
        "$\\mathrm{EL} = {{c1::PD}} \\times \\mathrm{LGD}$",
        ":::",
    ]);
    let parsed = parse(&lines);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let fm = &parsed.value.frontmatter;
    assert_eq!(fm.proposed_by.as_deref(), Some("claude"));
    assert_eq!(fm.date.as_deref(), Some("2026-10-20T14:32:00+02:00"));
    assert_eq!(fm.context.as_deref(), Some("Reviewing the LGD model"));
    assert_eq!(fm.target.as_deref(), Some("notes/credit-risk/lgd.md"));
    assert_eq!(
        fm.why.as_deref(),
        Some("The downturn adjustment is easy to forget")
    );
    let cards = &parsed.value.cards;
    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].line, 10);
    assert_eq!(
        cards[0].kind,
        CardKind::Qa {
            front: "Why use a downturn LGD?".into(),
            back: "Capital must cover losses in a downturn.".into()
        }
    );
    assert_eq!(cards[1].line, 16);
}

#[test]
fn frontmatter_is_required() {
    let parsed = parse(&QA);
    assert_eq!(codes(&parsed), vec![(1, "missing-required-key")]);
    assert_eq!(parsed.value.cards.len(), 1);
}

#[test]
fn missing_required_keys_are_reported_in_order() {
    let mut lines = vec!["---", "---"];
    lines.extend_from_slice(&QA);
    let parsed = parse(&lines);
    assert_eq!(codes(&parsed), vec![(1, "missing-required-key"); 5]);
    let keys: Vec<String> = parsed
        .diagnostics
        .iter()
        .map(|d| d.message.split('`').nth(1).unwrap_or_default().to_owned())
        .collect();
    assert_eq!(keys, ["proposed_by", "date", "context", "target", "why"]);
}

#[test]
fn empty_required_values() {
    assert_eq!(
        codes(&with_key("why", "why:")),
        vec![(6, "missing-required-key")]
    );
    let context = with_key("context", "context: ''");
    assert_eq!(codes(&context), vec![(4, "missing-required-key")]);
    assert_eq!(context.value.frontmatter.context, None);
    let by = with_key("proposed_by", "proposed_by: '  '");
    assert_eq!(codes(&by), vec![(2, "missing-required-key")]);
}

#[test]
fn required_keys_must_be_strings() {
    for (key, line, at) in [
        ("proposed_by", "proposed_by: 123", 2),
        ("context", "context: [a]", 4),
        ("target", "target: {a: 1}", 5),
        ("why", "why: true", 6),
    ] {
        let parsed = with_key(key, line);
        assert_eq!(codes(&parsed), vec![(at, "invalid-frontmatter")], "{line}");
        assert_eq!(
            parsed.diagnostics[0].message,
            format!("`{key}` must be a string")
        );
    }
    let yes = with_key("proposed_by", "proposed_by: yes");
    assert!(yes.diagnostics.is_empty(), "{:?}", yes.diagnostics);
    assert_eq!(yes.value.frontmatter.proposed_by.as_deref(), Some("yes"));
}

#[test]
fn invalid_yaml_in_proposal() {
    let mut lines = vec!["---", "proposed_by: claude", "  bad: indent", "---"];
    lines.extend_from_slice(&QA);
    let parsed = parse(&lines);
    assert_eq!(codes(&parsed), vec![(3, "invalid-frontmatter")]);
    assert_eq!(parsed.value.cards.len(), 1);
}

#[test]
fn unclosed_frontmatter_in_proposal() {
    // A `---` line would close the frontmatter: the body is a cloze.
    let parsed = parse(&["---", "proposed_by: claude", ":::cloze", "{{c1::x}}", ":::"]);
    assert_eq!(codes(&parsed), vec![(1, "invalid-frontmatter")]);
    assert_eq!(parsed.value.cards.len(), 1);
}

#[test]
fn date_accepts_iso_8601_date_times() {
    for date in [
        "2026-10-20T14:32",
        "2026-10-20T14:32Z",
        "2026-10-20T14:32:05",
        "2026-10-20T14:32:05.123+02:00",
        "2026-10-20T14:32-05:30",
        "2024-02-29T00:00",
        "2000-02-29T23:59:59Z",
    ] {
        let parsed = with_key("date", &format!("date: '{date}'"));
        assert!(
            parsed.diagnostics.is_empty(),
            "{date}: {:?}",
            parsed.diagnostics
        );
        assert_eq!(parsed.value.frontmatter.date.as_deref(), Some(date));
        let unquoted = with_key("date", &format!("date: {date}"));
        assert!(
            unquoted.diagnostics.is_empty(),
            "{date}: {:?}",
            unquoted.diagnostics
        );
    }
}

#[test]
fn date_rejects_other_forms() {
    for date in [
        "'2026-10-20'",
        "'2026-10-20 14:32'",
        "'2026-10-20t14:32'",
        "'2026-10-20T14:32z'",
        "'2026-10-20T14:32.5'",
        "'2026-10-20T14:32:05.'",
        "'2026-10-20T24:00'",
        "'2026-10-20T14:60'",
        "'2026-10-20T14:32:60'",
        "'2026-13-01T00:00'",
        "'2026-00-10T00:00'",
        "'2026-04-31T00:00'",
        "'2026-02-29T00:00'",
        "'1900-02-29T00:00'",
        "'2026-10-20T14:32+0200'",
        "'2026-10-20T14:32+02'",
        "'2026-10-20T14:32+24:00'",
        "'20261020T1432'",
        "'2026-1-20T14:32'",
        "20261020",
    ] {
        let parsed = with_key("date", &format!("date: {date}"));
        assert_eq!(codes(&parsed), vec![(3, "invalid-frontmatter")], "{date}");
        assert_eq!(parsed.value.frontmatter.date, None);
    }
}

#[test]
fn target_accepts_paths_under_notes() {
    for target in [
        "notes/a.md",
        "notes/credit-risk/lgd downturn.md",
        "notes/café/é.md",
        "notes/a.b.md",
    ] {
        let parsed = with_key("target", &format!("target: '{target}'"));
        assert!(
            parsed.diagnostics.is_empty(),
            "{target}: {:?}",
            parsed.diagnostics
        );
        assert_eq!(parsed.value.frontmatter.target.as_deref(), Some(target));
    }
}

#[test]
fn target_rejections() {
    let absolute = "expected a path relative to the vault, such as `notes/credit-risk/lgd.md`";
    let separator = "use `/` as path separator";
    let component = "empty, `.`, `..` and hidden path components are not allowed";
    let under = "expected a path under `notes/`, such as `notes/credit-risk/lgd.md`";
    let md = "expected a `.md` file";
    let cases = [
        ("/notes/a.md", absolute),
        ("C:/notes/a.md", absolute),
        ("c:\\notes\\a.md", absolute),
        ("\\\\srv\\notes\\a.md", absolute),
        ("notes\\a.md", separator),
        ("notes//a.md", component),
        ("notes/./a.md", component),
        ("notes/../a.md", component),
        ("../notes/a.md", component),
        ("notes/.hidden/a.md", component),
        ("notes/", component),
        ("notes", under),
        ("Notes/a.md", under),
        ("inbox/a.md", under),
        ("a.md", under),
        ("notes/a.txt", md),
        ("notes/a.MD", md),
    ];
    for (target, reason) in cases {
        let yaml = format!("target: '{}'", target.replace('\'', "''"));
        let parsed = with_key("target", &yaml);
        assert_eq!(codes(&parsed), vec![(5, "invalid-target")], "{target}");
        assert_eq!(
            parsed.diagnostics[0].message,
            format!("invalid target `{target}`: {reason}")
        );
        assert_eq!(parsed.value.frontmatter.target, None);
    }
    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS[..3]);
    lines.extend_from_slice(&["target: |", "  notes/a", "  b.txt", KEYS[4], "---"]);
    lines.extend_from_slice(&QA);
    let multiline = parse(&lines);
    // A block scalar is located on its first content line.
    assert_eq!(codes(&multiline), vec![(6, "invalid-target")]);
    assert_eq!(
        multiline.diagnostics[0].message,
        format!("invalid target `notes/a b.txt `: {md}")
    );
}

#[test]
fn optional_tags_and_sources() {
    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS);
    lines.extend_from_slice(&["tags: [lgd, basel]", "sources:", "  - CRE36", "---"]);
    lines.extend_from_slice(&QA);
    let parsed = parse(&lines);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.value.frontmatter.tags, vec!["lgd", "basel"]);
    assert_eq!(parsed.value.frontmatter.sources, vec!["CRE36"]);

    let absent = with_key("why", KEYS[4]);
    assert!(absent.value.frontmatter.tags.is_empty());

    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS);
    lines.extend_from_slice(&["tags:", "sources: ~", "---"]);
    lines.extend_from_slice(&QA);
    assert!(parse(&lines).diagnostics.is_empty());

    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS);
    lines.extend_from_slice(&["tags: lgd", "sources: [1]", "---"]);
    lines.extend_from_slice(&QA);
    let bad = parse(&lines);
    assert_eq!(
        messages(&bad),
        [
            format!("{PATH}:7: error: `tags` must be a list of strings [invalid-frontmatter]"),
            format!("{PATH}:8: error: `sources` must be a list of strings [invalid-frontmatter]"),
        ]
    );
}

#[test]
fn unknown_keys_are_kept_in_extra() {
    let mut lines = vec!["---"];
    lines.extend_from_slice(&KEYS);
    lines.extend_from_slice(&["title: Ignored", "meta:", "  n: 2", "---"]);
    lines.extend_from_slice(&QA);
    let parsed = parse(&lines);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let extra = &parsed.value.frontmatter.extra;
    assert_eq!(extra["title"], serde_json::json!("Ignored"));
    assert_eq!(extra["meta"], serde_json::json!({"n": 2}));
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
fn read_proposal_reports_unreadable_files() {
    let missing = read_proposal(Path::new("/nonexistent/engramme/inbox/x.md"));
    assert_eq!(codes(&missing), vec![(1, "unreadable-file")]);
    assert!(
        missing.diagnostics[0]
            .message
            .starts_with("cannot read file: ")
    );

    let path = temp_file("bad.md", b"---\nproposed_by: \xff\n");
    let bad = read_proposal(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(codes(&bad), vec![(2, "unreadable-file")]);
    assert_eq!(bad.diagnostics[0].message, "file is not valid UTF-8");
}

#[test]
fn proposal_frontmatter_messages() {
    let mut no_keys = vec!["---", "---"];
    no_keys.extend_from_slice(&QA);
    let cases = [
        (
            parse(&QA),
            "1: error: proposal has no frontmatter: add `proposed_by`, `date`, `context`, `target` and `why` between two `---` lines [missing-required-key]",
        ),
        (
            with_key("why", ""),
            "1: error: missing required key `why` in proposal frontmatter [missing-required-key]",
        ),
        (
            with_key("why", "why:"),
            "6: error: required key `why` is empty [missing-required-key]",
        ),
        (
            with_key("date", "date: '2026-10-20'"),
            "3: error: `date` must be an ISO 8601 date-time such as `2026-10-20T14:32:00+02:00` [invalid-frontmatter]",
        ),
        (
            with_key("context", "context: 1"),
            "4: error: `context` must be a string [invalid-frontmatter]",
        ),
        (
            with_key("target", "target: notes/a.txt"),
            "5: error: invalid target `notes/a.txt`: expected a `.md` file [invalid-target]",
        ),
    ];
    for (parsed, expected) in cases {
        assert_eq!(messages(&parsed), [format!("{PATH}:{expected}")]);
    }
}
