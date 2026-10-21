//! Docs rules → tests
//! - At least three format examples, including two notes and one proposal: `format_examples_pass_check`
//! - Complete examples pass check without diagnostics and each contains cards: `format_examples_pass_check`
//! - Every parser diagnostic code is documented: `format_documents_every_code`

use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn read(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {path}: {err}"))
}

/// Column-zero backtick fences, closed by exactly the opening run.
fn fenced(doc: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut lines = doc.lines();
    while let Some(line) = lines.next() {
        let ticks = line.bytes().take_while(|&byte| byte == b'`').count();
        if ticks < 3 {
            continue;
        }
        let (fence, info) = line.split_at(ticks);
        let mut content = String::new();
        let mut closed = false;
        for line in lines.by_ref() {
            if line == fence {
                closed = true;
                break;
            }
            content.push_str(line);
            content.push('\n');
        }
        assert!(closed, "unclosed documentation fence: {fence}{info}");
        blocks.push((info.trim().to_owned(), content));
    }
    blocks
}

fn examples(doc: &str) -> Vec<(String, String)> {
    fenced(doc)
        .into_iter()
        .filter_map(|(info, content)| {
            let path = info.strip_prefix("markdown file=")?;
            assert!(
                !path.chars().any(char::is_whitespace)
                    && (path.starts_with("notes/") || path.starts_with("inbox/"))
                    && path.ends_with(".md")
                    && path
                        .split('/')
                        .all(|part| !part.is_empty() && !part.starts_with('.')),
                "invalid documentation example path: {path:?}"
            );
            Some((path.to_owned(), content))
        })
        .collect()
}

fn vault(test: &str, files: &[(String, String)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("engramme-docs-{test}-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    let mut paths = HashSet::new();
    for (path, content) in files {
        assert!(
            paths.insert(path),
            "duplicate documentation example: {path}"
        );
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    root
}

fn check(root: &Path) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let args = [
        OsStr::new("engramme"),
        OsStr::new("check"),
        root.as_os_str(),
    ];
    let code = engramme_cli::run_with(args, None, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

fn clean(n: usize) -> String {
    if n == 1 {
        "checked 1 file: 0 errors, 0 warnings\n".to_owned()
    } else {
        format!("checked {n} files: 0 errors, 0 warnings\n")
    }
}

#[test]
fn format_examples_pass_check() {
    let files = examples(&read("../../docs/format.md"));
    assert!(
        files.len() >= 3,
        "document at least three complete examples"
    );
    assert!(
        files
            .iter()
            .filter(|(p, _)| p.starts_with("notes/"))
            .count()
            >= 2,
        "document at least two notes"
    );
    assert!(
        files.iter().any(|(p, _)| p.starts_with("inbox/")),
        "document at least one inbox proposal"
    );
    let root = vault("format_examples_pass_check", &files);
    assert_eq!(check(&root), (0, clean(files.len()), String::new()));
    let parsed = engramme_format::read_vault(&root);
    for note in &parsed.value.notes {
        assert!(
            !note.cards.is_empty(),
            "{} has no cards",
            note.path.display()
        );
    }
    for proposal in &parsed.value.proposals {
        assert!(
            !proposal.cards.is_empty(),
            "{} has no cards",
            proposal.path.display()
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn format_documents_every_code() {
    let source = read("../engramme-format/src/diagnostic.rs");
    let codes: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("DiagnosticKind::"))
        .filter_map(|line| {
            line.split_once("=> \"")?
                .1
                .split_once('"')
                .map(|(code, _)| code)
        })
        .collect();
    assert_eq!(
        codes.len(),
        21,
        "document the new code and update this count"
    );
    let doc = read("../../docs/format.md");
    for code in codes {
        assert!(
            doc.contains(&format!("`{code}`")),
            "undocumented code: {code}"
        );
    }
}
