//! Vault checks: walk of `notes/` and `inbox/`, duplicate ids, role of a
//! lone file, and the map of every §5 diagnostic to the tests that trigger it.
//!
//! Vault rules → tests
//! - walk `notes/` recursively and `inbox/` flat, skip hidden entries, non-`.md` files, `.app/` and other root files: `walk_ignores_hidden_non_md_app_and_inbox_subdirectories`, `vault_fixtures_match_snapshots`
//! - deterministic order, by path components: `walk_order_follows_path_components`, `vault_diagnostics_are_sorted_by_file_then_line`
//! - symbolic links followed only to regular files, except `notes/` and `inbox/` themselves: `symlinks_are_followed_only_to_files`, `notes_directory_may_be_a_symlink`
//! - missing or unreadable directories: `missing_directories`, `unreadable_vault_paths`
//! - duplicate ids among notes: `duplicate_ids_across_notes`, `duplicate_ids_within_a_note`, `every_later_occurrence_cites_the_first`, `qa_and_cloze_share_ids`, `cards_without_id_are_not_duplicates`, `proposal_ids_are_not_duplicates`, `duplicate_messages`
//! - role of a lone file: `check_path_by_role`, `check_path_finds_duplicates_in_a_lone_note`
//! - files counted by `check_path`: `check_path_counts_files`
//!
//! §5 diagnostic → code → tests
//! - unclosed block → `unclosed-block` → `unclosed_block_at_eof` (notes.rs)
//! - unknown block type → `unknown-block-type` → `unknown_type_is_an_error` (notes.rs)
//! - empty face in a `qa` → `empty-face` → `qa_empty_front`, `qa_empty_back` (notes.rs)
//! - missing `---` in a `qa` → `missing-separator` → `qa_missing_separator` (notes.rs)
//! - cloze without deletion, or empty deletion → `no-cloze-deletion`, `invalid-cloze-deletion` → `cloze_without_deletion`, `cloze_empty_deletion` (notes.rs)
//! - malformed id → `malformed-id` → `malformed_id_is_an_error` (notes.rs)
//! - duplicate id in the vault → `duplicate-id` → `duplicate_ids_across_notes`, `duplicate_ids_within_a_note`
//! - invalid YAML frontmatter → `invalid-frontmatter` → `invalid_yaml_reports_file_line` (notes.rs), `invalid_yaml_in_proposal` (inbox.rs)
//! - missing required key in a proposal → `missing-required-key` → `missing_required_keys_are_reported_in_order` (inbox.rs)
//! - `target` outside `notes/` → `invalid-target` → `target_rejections` (inbox.rs)
//! - more than 3 cards in a proposal → `too-many-cards` → `more_than_three_cards_is_an_error` (inbox.rs)
//! - id in a proposal → `id-in-proposal` → `id_in_proposal_is_an_error` (inbox.rs)
//! - reserved type (warning) → `reserved-block-type` → `reserved_types_warn_and_yield_no_card` (notes.rs)
//! - non-consecutive cloze numbers (warning) → `non-consecutive-cloze-numbers` → `cloze_non_consecutive_numbers_warn` (notes.rs)
//! - added: `invalid-block-header` → `invalid_block_header` (notes.rs); `unreadable-file` → `read_note_reports_unreadable_files` (notes.rs), `read_proposal_reports_unreadable_files` (inbox.rs), `unreadable_vault_paths`; `unclosed-code-fence` → `unclosed_code_fence_warns` (notes.rs); `no-card` → `no_card_is_an_error` (inbox.rs); `unexpected-file-name` → `file_name_convention` (inbox.rs); `not-a-vault` → `missing_directories`
//! - every kind triggered, codes and severities stable: `every_diagnostic_kind_is_triggered`, `codes_and_severities_are_stable`

use std::path::{Path, PathBuf};

use engramme_format::{
    Diagnostic, DiagnosticKind, Note, Parsed, Severity, Vault, check_path, duplicate_ids,
    parse_note, parse_proposal, read_note, read_vault,
};

fn note(path: &str, lines: &[&str]) -> Note {
    parse_note(Path::new(path), &format!("{}\n", lines.join("\n"))).value
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<(usize, &'static str)> {
    diagnostics
        .iter()
        .map(|d| (d.line, d.kind.code()))
        .collect()
}

fn located(diagnostics: &[Diagnostic]) -> Vec<(&Path, usize, &'static str)> {
    diagnostics
        .iter()
        .map(|d| (d.file.as_path(), d.line, d.kind.code()))
        .collect()
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics.iter().map(|d| d.to_string()).collect()
}

#[test]
fn duplicate_ids_across_notes() {
    let a = note("notes/a.md", &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"]);
    let b = note(
        "notes/b.md",
        &["# B", "", ":::qa {#c-91be0q}", "Q", "---", "A", ":::"],
    );
    assert_eq!(
        located(&duplicate_ids(&[a.clone(), b.clone()])),
        [(Path::new("notes/b.md"), 3, "duplicate-id")]
    );
    assert_eq!(
        located(&duplicate_ids(&[b, a])),
        [(Path::new("notes/a.md"), 1, "duplicate-id")]
    );
}

#[test]
fn duplicate_ids_within_a_note() {
    let a = note(
        "notes/a.md",
        &[
            ":::qa {#c-91be0q}",
            "Q",
            "---",
            "A",
            ":::",
            "",
            ":::qa {#c-91be0q}",
            "Another Q",
            "---",
            "Another A",
            ":::",
        ],
    );
    assert_eq!(
        located(&duplicate_ids(&[a])),
        [(Path::new("notes/a.md"), 7, "duplicate-id")]
    );
}

#[test]
fn every_later_occurrence_cites_the_first() {
    let a = note(
        "notes/a.md",
        &["# A", "", ":::qa {#c-91be0q}", "Q", "---", "A", ":::"],
    );
    let b = note(
        "notes/b.md",
        &[
            ":::qa {#c-91be0q}",
            "Q",
            "---",
            "A",
            ":::",
            ":::qa {#c-91be0q}",
            "Q",
            "---",
            "A",
            ":::",
        ],
    );
    let diagnostics = duplicate_ids(&[a, b]);
    assert_eq!(
        located(&diagnostics),
        [
            (Path::new("notes/b.md"), 1, "duplicate-id"),
            (Path::new("notes/b.md"), 6, "duplicate-id")
        ]
    );
    for d in &diagnostics {
        assert!(
            d.message.ends_with("first used at notes/a.md:3"),
            "{}",
            d.message
        );
    }
}

#[test]
fn qa_and_cloze_share_ids() {
    let a = note(
        "notes/a.md",
        &[
            ":::qa {#c-91be0q}",
            "Q",
            "---",
            "A",
            ":::",
            ":::cloze {#c-91be0q}",
            "{{c1::x}}",
            ":::",
        ],
    );
    assert_eq!(
        located(&duplicate_ids(&[a])),
        [(Path::new("notes/a.md"), 6, "duplicate-id")]
    );
}

#[test]
fn cards_without_id_are_not_duplicates() {
    let qa = [":::qa", "Q", "---", "A", ":::"];
    let a = note("notes/a.md", &[qa, qa].concat());
    let b = note(
        "notes/b.md",
        &[&qa[..], &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"]].concat(),
    );
    assert_eq!(a.cards.len() + b.cards.len(), 4);
    assert!(duplicate_ids(&[a, b]).is_empty());
}

#[test]
fn duplicate_messages() {
    let a = note("notes/a.md", &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"]);
    let b = note(
        "notes/b.md",
        &["# B", "", ":::cloze {#c-91be0q}", "{{c1::x}}", ":::"],
    );
    assert_eq!(
        messages(&duplicate_ids(&[a, b.clone()])),
        [
            "notes/b.md:3: error: duplicate card id `c-91be0q`: first used at notes/a.md:1 [duplicate-id]"
        ]
    );
    let odd = note(
        "notes/a\nb.md",
        &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"],
    );
    assert_eq!(
        messages(&duplicate_ids(&[odd, b])),
        [
            "notes/b.md:3: error: duplicate card id `c-91be0q`: first used at notes/a b.md:1 [duplicate-id]"
        ]
    );
}

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/vault");

/// `path` relative to `root`, components joined by `/`.
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap()
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// `file:line: code` of each diagnostic, `file` relative to `root`.
fn summary(root: &Path, diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|d| format!("{}:{}: {}", relative(root, &d.file), d.line, d.kind.code()))
        .collect()
}

/// Paths of the notes and of the proposals of `vault`, relative to its root.
fn walked(vault: &Vault) -> (Vec<String>, Vec<String>) {
    (
        vault
            .notes
            .iter()
            .map(|n| relative(&vault.root, &n.path))
            .collect(),
        vault
            .proposals
            .iter()
            .map(|p| relative(&vault.root, &p.path))
            .collect(),
    )
}

/// `item` as JSON, with its `path` relative to `root`.
fn with_relative_path(root: &Path, path: &Path, item: impl serde::Serialize) -> serde_json::Value {
    let mut value = serde_json::to_value(item).unwrap();
    value["path"] = relative(root, path).into();
    value
}

fn snapshot(parsed: &Parsed<Vault>) -> serde_json::Value {
    let root = &parsed.value.root;
    let notes: Vec<serde_json::Value> = parsed
        .value
        .notes
        .iter()
        .map(|n| with_relative_path(root, &n.path, n))
        .collect();
    let proposals: Vec<serde_json::Value> = parsed
        .value
        .proposals
        .iter()
        .map(|p| with_relative_path(root, &p.path, p))
        .collect();
    let diagnostics: Vec<serde_json::Value> = parsed
        .diagnostics
        .iter()
        .map(|d| {
            serde_json::json!({
                "file": relative(root, &d.file),
                "line": d.line,
                "severity": d.severity,
                "code": d.kind.code(),
            })
        })
        .collect();
    serde_json::json!({
        "notes": notes,
        "proposals": proposals,
        "diagnostics": diagnostics,
    })
}

#[test]
fn vault_fixtures_match_snapshots() {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    roots.sort();
    assert!(!roots.is_empty());
    let update = std::env::var_os("UPDATE_FIXTURES").is_some();
    let mut failures = Vec::new();
    for root in &roots {
        let parsed = read_vault(root);
        let name = root.file_name().unwrap().to_string_lossy();
        let in_valid = name == "valid";
        if in_valid && parsed.has_errors() {
            failures.push(format!("{name}: errors in a valid vault"));
        }
        if !in_valid && !parsed.has_errors() {
            failures.push(format!("{name}: no error in an invalid vault"));
        }
        let actual = snapshot(&parsed);
        let json = root.with_extension("json");
        if update {
            let text = serde_json::to_string_pretty(&actual).unwrap() + "\n";
            std::fs::write(&json, text).unwrap();
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&json) else {
            failures.push(format!("{}: missing snapshot", json.display()));
            continue;
        };
        let expected: serde_json::Value = serde_json::from_str(&text).unwrap();
        if expected != actual {
            failures.push(format!(
                "{name}:\nexpected: {}\nactual:   {}",
                serde_json::to_string_pretty(&expected).unwrap(),
                serde_json::to_string_pretty(&actual).unwrap()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn walk_ignores_hidden_non_md_app_and_inbox_subdirectories() {
    let root = Path::new(FIXTURES).join("valid");
    for ignored in [
        "notes/attachments/diagram.txt",
        "notes/.drafts/broken.md",
        "notes/.broken.md",
        "inbox/archive/2026-10-19T0900-claude-old.md",
        "inbox/notes.txt",
        "README.md",
        ".app/log/2026-10.mac.jsonl",
    ] {
        assert!(root.join(ignored).is_file(), "{ignored} is missing");
    }
    let parsed = read_vault(&root);
    let (notes, proposals) = walked(&parsed.value);
    assert_eq!(
        notes,
        ["notes/credit-risk/lgd-downturn.md", "notes/ifrs9.md"]
    );
    assert_eq!(proposals, ["inbox/2026-10-20T1432-claude-ifrs9-staging.md"]);
    assert_eq!(
        summary(&root, &parsed.diagnostics),
        ["notes/ifrs9.md:11: reserved-block-type"]
    );
}

#[test]
fn walk_order_follows_path_components() {
    let root = Path::new(FIXTURES).join("invalid");
    let parsed = read_vault(&root);
    let (notes, _) = walked(&parsed.value);
    assert_eq!(notes, ["notes/a/b.md", "notes/a-b.md", "notes/a.md"]);
    // The walk order decides which occurrence of `c-91be0q` comes first.
    let duplicate = parsed
        .diagnostics
        .iter()
        .find(|d| d.kind == DiagnosticKind::DuplicateId && d.message.contains("`c-91be0q`"))
        .unwrap();
    assert_eq!(relative(&root, &duplicate.file), "notes/a-b.md");
    assert!(
        duplicate.message.ends_with(&format!(
            "first used at {}:1",
            root.join("notes/a/b.md").display()
        )),
        "{}",
        duplicate.message
    );
}

/// A new directory with a unique name under the system temp dir, removed
/// with its contents when dropped.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> TempDir {
        let path = std::env::temp_dir().join(format!(
            "engramme-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        TempDir(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Writes each `(path, contents)` under `root`, creating parent directories.
fn write_files(root: &Path, files: &[(&str, &str)]) {
    for (path, contents) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}

#[test]
fn vault_diagnostics_are_sorted_by_file_then_line() {
    let root = Path::new(FIXTURES).join("invalid");
    assert_eq!(
        summary(&root, &read_vault(&root).diagnostics),
        [
            "inbox/2026-10-20T1432-claude-dup-id.md:1: missing-required-key",
            "inbox/2026-10-20T1432-claude-dup-id.md:7: id-in-proposal",
            "inbox/misnamed.md:1: unexpected-file-name",
            "notes/a-b.md:3: duplicate-id",
            "notes/a.md:7: duplicate-id",
            "notes/a.md:11: unclosed-block",
        ]
    );
    // Files follow the order of `Path`, not of their text: `a/b.md` first.
    let temp = TempDir::new("sorted");
    write_files(
        temp.path(),
        &[
            ("notes/a.md", ":::qa\n"),
            ("notes/a-b.md", ":::qa\n"),
            ("notes/a/b.md", ":::qa\n"),
        ],
    );
    assert_eq!(
        summary(temp.path(), &read_vault(temp.path()).diagnostics),
        [
            "notes/a/b.md:1: unclosed-block",
            "notes/a-b.md:1: unclosed-block",
            "notes/a.md:1: unclosed-block",
        ]
    );
}

#[test]
fn proposal_ids_are_not_duplicates() {
    let root = Path::new(FIXTURES).join("invalid");
    let parsed = read_vault(&root);
    let id_diagnostics: Vec<String> = summary(&root, &parsed.diagnostics)
        .into_iter()
        .filter(|s| s.ends_with(": id-in-proposal") || s.ends_with(": duplicate-id"))
        .collect();
    assert_eq!(
        id_diagnostics,
        [
            "inbox/2026-10-20T1432-claude-dup-id.md:7: id-in-proposal",
            "notes/a-b.md:3: duplicate-id",
            "notes/a.md:7: duplicate-id",
        ]
    );
    assert!(parsed.value.proposals[0].cards.is_empty());
}

#[test]
fn missing_directories() {
    let empty = TempDir::new("empty");
    let parsed = read_vault(empty.path());
    assert_eq!(
        messages(&parsed.diagnostics),
        [format!(
            "{}:1: warning: no `notes/` or `inbox/` directory: nothing to check [not-a-vault]",
            empty.path().display()
        )]
    );
    assert!(!parsed.has_errors());

    let notes_only = TempDir::new("notes-only");
    write_files(notes_only.path(), &[("notes/a.md", "# A\n")]);
    let parsed = read_vault(notes_only.path());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.value.notes.len(), 1);

    let inbox_only = TempDir::new("inbox-only");
    std::fs::create_dir(inbox_only.path().join("inbox")).unwrap();
    let parsed = read_vault(inbox_only.path());
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
}

#[test]
fn unreadable_vault_paths() {
    let temp = TempDir::new("unreadable");
    write_files(temp.path(), &[("file", ""), ("vault/notes", "")]);
    let missing = temp.path().join("missing");
    let file = temp.path().join("file");
    let vault = temp.path().join("vault");
    let notes_file = vault.join("notes");
    for (root, unreadable) in [(&missing, &missing), (&file, &file), (&vault, &notes_file)] {
        let parsed = read_vault(root);
        // A single diagnostic: no `not-a-vault` when `notes` is a file.
        assert_eq!(
            located(&parsed.diagnostics),
            [(unreadable.as_path(), 1, "unreadable-file")]
        );
        let message = &parsed.diagnostics[0].message;
        assert!(message.starts_with("cannot read directory: "), "{message}");
        assert!(parsed.value.notes.is_empty() && parsed.value.proposals.is_empty());
    }
}

#[cfg(unix)]
#[test]
fn symlinks_are_followed_only_to_files() {
    use std::os::unix::fs::symlink;
    let temp = TempDir::new("symlinks");
    write_files(
        temp.path(),
        &[
            ("outside/card.md", ":::qa\nQ\n---\nA\n:::\n"),
            ("outside/dir/bad.md", ":::qa\nnever closed\n"),
            ("vault/notes/plain.md", "# Plain\n"),
        ],
    );
    let outside = temp.path().join("outside");
    let root = temp.path().join("vault");
    let notes = root.join("notes");
    symlink(outside.join("card.md"), notes.join("link.md")).unwrap();
    symlink(&notes, notes.join("cycle")).unwrap();
    // Named `.md` too, so that it is neither walked nor read as a file.
    symlink(outside.join("dir"), notes.join("dir.md")).unwrap();
    symlink(outside.join("missing.md"), notes.join("broken.md")).unwrap();
    let parsed = read_vault(&root);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let (walked_notes, _) = walked(&parsed.value);
    assert_eq!(walked_notes, ["notes/link.md", "notes/plain.md"]);
    assert_eq!(parsed.value.notes[0].cards.len(), 1);
}

#[test]
fn check_path_by_role() {
    let temp = TempDir::new("roles");
    let qa = ":::qa\nQ\n---\nA\n:::\n";
    write_files(
        temp.path(),
        &[
            ("vault/inbox/2026-10-20T1432-claude-x.md", qa),
            ("vault/notes/x.md", qa),
            ("x.md", qa),
            ("Inbox/2026-10-20T1432-claude-x.md", qa),
        ],
    );
    let proposal =
        check_path(&temp.path().join("vault/inbox/2026-10-20T1432-claude-x.md")).diagnostics;
    assert_eq!(codes(&proposal), [(1, "missing-required-key")]);
    assert!(
        proposal[0]
            .message
            .starts_with("proposal has no frontmatter: "),
        "{}",
        proposal[0].message
    );
    for note in [
        "vault/notes/x.md",
        "x.md",
        "Inbox/2026-10-20T1432-claude-x.md",
    ] {
        let diagnostics = check_path(&temp.path().join(note)).diagnostics;
        assert!(diagnostics.is_empty(), "{note}: {diagnostics:?}");
    }
    let vault = temp.path().join("vault");
    let diagnostics = check_path(&vault).diagnostics;
    assert_eq!(diagnostics, read_vault(&vault).diagnostics);
    assert_eq!(codes(&diagnostics), [(1, "missing-required-key")]);
    let missing = check_path(&temp.path().join("missing.md")).diagnostics;
    assert_eq!(codes(&missing), [(1, "unreadable-file")]);
    assert!(
        missing[0].message.starts_with("cannot read file: "),
        "{}",
        missing[0].message
    );
}

#[test]
fn check_path_finds_duplicates_in_a_lone_note() {
    let root = Path::new(FIXTURES).join("invalid");
    let lone = root.join("notes/a.md");
    let diagnostics = check_path(&lone).diagnostics;
    assert_eq!(
        codes(&diagnostics),
        [(7, "duplicate-id"), (11, "unclosed-block")]
    );
    assert_eq!(
        diagnostics[0].message,
        format!(
            "duplicate card id `c-3x9k2p`: first used at {}:1",
            lone.display()
        )
    );
    // Alone, `notes/a-b.md` does not see the `c-91be0q` of `notes/a/b.md`.
    assert!(
        check_path(&root.join("notes/a-b.md"))
            .diagnostics
            .is_empty()
    );
}

const ALL_KINDS: [(DiagnosticKind, &str, Severity); 21] = [
    (
        DiagnosticKind::InvalidFrontmatter,
        "invalid-frontmatter",
        Severity::Error,
    ),
    (
        DiagnosticKind::UnclosedBlock,
        "unclosed-block",
        Severity::Error,
    ),
    (
        DiagnosticKind::InvalidBlockHeader,
        "invalid-block-header",
        Severity::Error,
    ),
    (
        DiagnosticKind::UnknownBlockType,
        "unknown-block-type",
        Severity::Error,
    ),
    (DiagnosticKind::MalformedId, "malformed-id", Severity::Error),
    (
        DiagnosticKind::MissingSeparator,
        "missing-separator",
        Severity::Error,
    ),
    (DiagnosticKind::EmptyFace, "empty-face", Severity::Error),
    (
        DiagnosticKind::NoClozeDeletion,
        "no-cloze-deletion",
        Severity::Error,
    ),
    (
        DiagnosticKind::InvalidClozeDeletion,
        "invalid-cloze-deletion",
        Severity::Error,
    ),
    (
        DiagnosticKind::UnreadableFile,
        "unreadable-file",
        Severity::Error,
    ),
    (
        DiagnosticKind::MissingRequiredKey,
        "missing-required-key",
        Severity::Error,
    ),
    (
        DiagnosticKind::InvalidTarget,
        "invalid-target",
        Severity::Error,
    ),
    (DiagnosticKind::NoCard, "no-card", Severity::Error),
    (
        DiagnosticKind::TooManyCards,
        "too-many-cards",
        Severity::Error,
    ),
    (
        DiagnosticKind::IdInProposal,
        "id-in-proposal",
        Severity::Error,
    ),
    (DiagnosticKind::DuplicateId, "duplicate-id", Severity::Error),
    (
        DiagnosticKind::ReservedBlockType,
        "reserved-block-type",
        Severity::Warning,
    ),
    (
        DiagnosticKind::NonConsecutiveClozeNumbers,
        "non-consecutive-cloze-numbers",
        Severity::Warning,
    ),
    (
        DiagnosticKind::UnclosedCodeFence,
        "unclosed-code-fence",
        Severity::Warning,
    ),
    (
        DiagnosticKind::UnexpectedFileName,
        "unexpected-file-name",
        Severity::Warning,
    ),
    (DiagnosticKind::NotAVault, "not-a-vault", Severity::Warning),
];

const PROPOSAL_KEYS: &str = "---\nproposed_by: claude\ndate: 2026-10-20T14:32\ncontext: c\ntarget: notes/a.md\nwhy: w\n---\n";

fn note_diagnostics(source: &str) -> Vec<Diagnostic> {
    parse_note(Path::new("notes/a.md"), source).diagnostics
}

fn proposal_diagnostics(name: &str, source: &str) -> Vec<Diagnostic> {
    parse_proposal(Path::new(&format!("inbox/{name}")), source).diagnostics
}

/// Diagnostics of an input that triggers `kind`, through the public API only.
/// No `_` arm: a new kind without a trigger does not compile.
fn trigger(kind: DiagnosticKind) -> Vec<Diagnostic> {
    const NAME: &str = "2026-10-20T1432-claude-x.md";
    let qa = ":::qa\nQ\n---\nA\n:::\n";
    match kind {
        DiagnosticKind::InvalidFrontmatter => note_diagnostics("---\n- a\n---\n"),
        DiagnosticKind::UnclosedBlock => note_diagnostics(":::qa\nQ\n"),
        DiagnosticKind::InvalidBlockHeader => note_diagnostics("::: qa\n:::\n"),
        DiagnosticKind::UnknownBlockType => note_diagnostics(":::note\n:::\n"),
        DiagnosticKind::MalformedId => note_diagnostics(":::qa {#c-1}\nQ\n---\nA\n:::\n"),
        DiagnosticKind::MissingSeparator => note_diagnostics(":::qa\nQ\n:::\n"),
        DiagnosticKind::EmptyFace => note_diagnostics(":::qa\n---\nA\n:::\n"),
        DiagnosticKind::NoClozeDeletion => note_diagnostics(":::cloze\nx\n:::\n"),
        DiagnosticKind::InvalidClozeDeletion => note_diagnostics(":::cloze\n{{c1::}}\n:::\n"),
        DiagnosticKind::UnreadableFile => {
            read_note(Path::new("/nonexistent/engramme/notes/a.md")).diagnostics
        }
        DiagnosticKind::MissingRequiredKey => proposal_diagnostics(NAME, qa),
        DiagnosticKind::InvalidTarget => proposal_diagnostics(
            NAME,
            &format!("{}{qa}", PROPOSAL_KEYS.replace("notes/a.md", "../a.md")),
        ),
        DiagnosticKind::NoCard => proposal_diagnostics(NAME, &format!("{PROPOSAL_KEYS}prose\n")),
        DiagnosticKind::TooManyCards => {
            proposal_diagnostics(NAME, &format!("{PROPOSAL_KEYS}{}", qa.repeat(4)))
        }
        DiagnosticKind::IdInProposal => proposal_diagnostics(
            NAME,
            &format!("{PROPOSAL_KEYS}:::qa {{#c-91be0q}}\nQ\n---\nA\n:::\n"),
        ),
        DiagnosticKind::DuplicateId => {
            let a = note("notes/a.md", &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"]);
            let b = note("notes/b.md", &[":::qa {#c-91be0q}", "Q", "---", "A", ":::"]);
            duplicate_ids(&[a, b])
        }
        DiagnosticKind::ReservedBlockType => note_diagnostics(":::free\n:::\n"),
        DiagnosticKind::NonConsecutiveClozeNumbers => {
            note_diagnostics(":::cloze\n{{c2::x}}\n:::\n")
        }
        DiagnosticKind::UnclosedCodeFence => note_diagnostics("```\n:::qa\n"),
        DiagnosticKind::UnexpectedFileName => {
            proposal_diagnostics("misnamed.md", &format!("{PROPOSAL_KEYS}{qa}"))
        }
        DiagnosticKind::NotAVault => {
            let dir = TempDir::new("empty-vault");
            read_vault(dir.path()).diagnostics
        }
    }
}

#[test]
fn every_diagnostic_kind_is_triggered() {
    for (kind, _, _) in ALL_KINDS {
        let diagnostics = trigger(kind);
        assert!(
            diagnostics.iter().any(|d| d.kind == kind),
            "{kind:?} not triggered: {diagnostics:?}"
        );
    }
}

#[test]
fn codes_and_severities_are_stable() {
    for (kind, code, severity) in ALL_KINDS {
        assert_eq!(kind.code(), code);
        assert_eq!(kind.severity(), severity, "{code}");
        assert_eq!(serde_json::to_value(kind).unwrap(), serde_json::json!(code));
    }
}

/// `notes/` and `inbox/` themselves may be links to directories (e.g. a synced
/// folder); only links met during the walk are restricted to regular files.
#[test]
#[cfg(unix)]
fn notes_directory_may_be_a_symlink() {
    let temp = TempDir::new("notes-symlink");
    write_files(
        temp.path(),
        &[
            ("synced/a.md", "# A\n"),
            ("synced/sub/b.md", "# B\n"),
            ("vault/inbox/.keep", ""),
        ],
    );
    let root = temp.path().join("vault");
    std::os::unix::fs::symlink(temp.path().join("synced"), root.join("notes")).unwrap();
    std::os::unix::fs::symlink(temp.path().join("synced"), temp.path().join("synced/loop"))
        .unwrap();
    let parsed = read_vault(&root);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let titles: Vec<&str> = parsed
        .value
        .notes
        .iter()
        .map(|n| n.title.as_str())
        .collect();
    assert_eq!(titles, ["A", "B"]);
}

#[test]
fn check_path_counts_files() {
    let fixtures = Path::new(FIXTURES);
    let valid = check_path(&fixtures.join("valid"));
    assert_eq!(valid.files, 3);
    assert_eq!(
        valid.diagnostics,
        read_vault(&fixtures.join("valid")).diagnostics
    );
    assert_eq!(check_path(&fixtures.join("invalid")).files, 5);
    assert_eq!(check_path(&fixtures.join("invalid/notes/a.md")).files, 1);
    let temp = TempDir::new("count");
    let missing = check_path(&temp.path().join("missing"));
    assert_eq!(missing.files, 0);
    assert_eq!(codes(&missing.diagnostics), [(1, "unreadable-file")]);
    let empty = check_path(temp.path());
    assert_eq!(empty.files, 0);
    assert_eq!(codes(&empty.diagnostics), [(1, "not-a-vault")]);
}
