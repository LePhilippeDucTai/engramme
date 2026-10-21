//! Docs rules → tests
//! - At least three format examples, including two notes and one proposal: `format_examples_pass_check`
//! - Complete examples pass check without diagnostics and each contains cards: `format_examples_pass_check`
//! - Every parser diagnostic code is documented: `format_documents_every_code`
//! - SKILL.md reads as a note without diagnostics; its frontmatter has only `name` and `description`, within Agent Skills limits: `skill_frontmatter_follows_agent_skills`
//! - The skill holds one example, a proposal by `claude` with 1 to 3 cards that passes check: `skill_template_passes_check`
//! - INSTALL.md snippets install the skill, add the instruction line once, and the Stop hook reminds without looping: `install_snippet_installs_the_skill`, `instruction_snippet_adds_the_line_once`, `stop_hook_reminds_without_looping`

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

#[test]
fn skill_frontmatter_follows_agent_skills() {
    let parsed = engramme_format::parse_note(Path::new("SKILL.md"), &read("../../skill/SKILL.md"));
    let diagnostics: Vec<String> = parsed.diagnostics.iter().map(ToString::to_string).collect();
    assert!(
        diagnostics.is_empty(),
        "SKILL.md as a note: {diagnostics:#?}"
    );
    let frontmatter = &parsed.value.frontmatter;
    assert_eq!(frontmatter.title, None);
    assert!(frontmatter.tags.is_empty() && frontmatter.sources.is_empty());
    let mut keys: Vec<&str> = frontmatter.extra.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["description", "name"]);
    assert_eq!(frontmatter.extra["name"].as_str(), Some("engramme-capture"));
    let description = frontmatter.extra["description"]
        .as_str()
        .expect("`description` must be a string");
    assert!(
        (1..=1024).contains(&description.chars().count()),
        "`description` must have 1 to 1024 characters"
    );
    assert_eq!(
        description,
        description.trim(),
        "`description` must not start or end with blanks"
    );
}

#[test]
fn skill_template_passes_check() {
    let files = examples(&read("../../skill/SKILL.md"));
    assert_eq!(
        files.len(),
        1,
        "SKILL.md must hold exactly one complete example"
    );
    assert!(
        files[0].0.starts_with("inbox/"),
        "the SKILL.md example must be a proposal under inbox/"
    );
    let root = vault("skill_template_passes_check", &files);
    assert_eq!(check(&root), (0, clean(1), String::new()));
    let parsed = engramme_format::read_vault(&root);
    let [proposal] = parsed.value.proposals.as_slice() else {
        panic!("the SKILL.md example must be read as one proposal");
    };
    assert!(
        (1..=3).contains(&proposal.cards.len()),
        "the SKILL.md example must have 1 to 3 cards"
    );
    assert_eq!(proposal.frontmatter.proposed_by.as_deref(), Some("claude"));
    std::fs::remove_dir_all(root).unwrap();
}

/// The content of the only `../../skill/INSTALL.md` fence whose info is `info`.
#[cfg(unix)]
fn snippet(info: &str) -> String {
    let blocks: Vec<String> = fenced(&read("../../skill/INSTALL.md"))
        .into_iter()
        .filter(|(found, _)| found == info)
        .map(|(_, content)| content)
        .collect();
    assert_eq!(blocks.len(), 1, "expected one `{info}` fence in INSTALL.md");
    blocks[0].clone()
}

/// Runs `script` with `sh -c` from the repository root, with `HOME=home`, and
/// returns whether it succeeded and its stdout.
#[cfg(unix)]
fn sh(script: &str, home: &Path, stdin: &str, engramme_dir: Option<&Path>) -> (bool, String) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(script)
        .current_dir("../..")
        .env("HOME", home)
        .env_remove("CODEX_HOME")
        .env_remove("COPILOT_HOME")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    match engramme_dir {
        Some(dir) => command.env("ENGRAMME_DIR", dir),
        None => command.env_remove("ENGRAMME_DIR"),
    };
    let mut child = command.spawn().unwrap();
    if let Some(mut input) = child.stdin.take() {
        let _ = input.write_all(stdin.as_bytes());
    }
    let output = child.wait_with_output().unwrap();
    (
        output.status.success(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[cfg(unix)]
#[test]
fn install_snippet_installs_the_skill() {
    let home = vault("install-skill", &[]);
    let (ok, _) = sh(&snippet("sh install-skill"), &home, "", None);
    assert!(ok);
    let skill = std::fs::read("../../skill/SKILL.md").unwrap();
    for dir in [".claude/skills", ".agents/skills"] {
        let installed = home.join(dir).join("engramme-capture/SKILL.md");
        assert_eq!(std::fs::read(&installed).unwrap(), skill, "{dir}");
    }
    std::fs::remove_dir_all(home).unwrap();
}

#[cfg(unix)]
#[test]
fn instruction_snippet_adds_the_line_once() {
    let home = vault("add-instruction", &[]);
    let script = snippet("sh add-instruction");
    for _ in 0..2 {
        assert!(sh(&script, &home, "", None).0);
    }
    let line = "At the end of a task, consider the engramme-capture skill.";
    for file in [
        ".claude/CLAUDE.md",
        ".codex/AGENTS.md",
        ".copilot/copilot-instructions.md",
    ] {
        let text = std::fs::read_to_string(home.join(file)).unwrap();
        assert_eq!(text.lines().filter(|l| *l == line).count(), 1, "{file}");
    }
    std::fs::remove_dir_all(home).unwrap();
}

#[cfg(unix)]
#[test]
fn stop_hook_reminds_without_looping() {
    let hook: serde_json::Value = serde_json::from_str(&snippet("json stop-hook")).unwrap();
    let entry = &hook["hooks"]["Stop"][0];
    assert!(entry.get("matcher").is_none());
    assert_eq!(entry["hooks"][0]["type"], "command");
    let command = entry["hooks"][0]["command"].as_str().unwrap();
    let home = vault("stop-hook", &[]);
    let dir = vault("stop-hook-vault", &[]);

    let (ok, out) = sh(
        command,
        &home,
        r#"{"hook_event_name":"Stop","stop_hook_active":false}"#,
        Some(&dir),
    );
    assert!(ok);
    let reminder: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(reminder["hookSpecificOutput"]["hookEventName"], "Stop");
    let context = reminder["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("engramme-capture"));

    let (ok, out) = sh(command, &home, r#"{"stop_hook_active": true}"#, Some(&dir));
    assert!(ok);
    assert_eq!(out, "");
    let inactive = r#"{"stop_hook_active":false}"#;
    assert_eq!(sh(command, &home, inactive, None).1, "");
    assert_eq!(
        sh(command, &home, inactive, Some(&dir.join("missing"))).1,
        ""
    );

    std::fs::remove_dir_all(home).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}
