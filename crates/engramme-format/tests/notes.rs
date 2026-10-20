use std::path::{Path, PathBuf};

use engramme_format::{
    Card, CardId, CardKind, ClozeDeletion, Diagnostic, DiagnosticKind, Severity, parse_note,
    read_note,
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
