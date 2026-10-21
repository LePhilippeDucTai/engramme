use std::path::Path;

use crate::block::{self, Scan};
use crate::diagnostic::{Diagnostic, Parsed};
use crate::frontmatter;
use crate::model::{Frontmatter, Note};
use crate::source::{self, BLANKS, Line};

/// Parses a note from its contents. Pure: reads nothing from disk.
pub fn parse_note(path: &Path, source: &str) -> Parsed<Note> {
    let lines = source::lines(source);
    let split = frontmatter::split(path, &lines);
    let mut diagnostics: Vec<Diagnostic> = split.diagnostic.into_iter().collect();
    let frontmatter = match split.yaml {
        Some(yaml) => match frontmatter::parse_fields(path, &yaml, split.closing_line) {
            Ok(fields) => frontmatter::note_frontmatter(path, fields, &mut diagnostics),
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                Frontmatter::default()
            }
        },
        None => Frontmatter::default(),
    };
    let Scan {
        cards,
        prose,
        diagnostics: block_diagnostics,
        ..
    } = block::scan(path, split.body);
    diagnostics.extend(block_diagnostics);
    let title = frontmatter
        .title
        .as_deref()
        .map(|title| title.trim())
        .filter(|title| !title.is_empty())
        .or_else(|| heading_title(&prose))
        .map_or_else(|| file_stem(path), str::to_owned);
    diagnostics.sort_by_key(|d| d.line);
    Parsed {
        value: Note {
            path: path.to_path_buf(),
            title,
            frontmatter,
            cards,
        },
        diagnostics,
    }
}

/// A note without frontmatter nor cards, titled by its file stem.
pub(crate) fn empty_note(path: &Path) -> Note {
    Note {
        path: path.to_path_buf(),
        title: file_stem(path),
        frontmatter: Frontmatter::default(),
        cards: Vec::new(),
    }
}

/// File name without extension.
pub(crate) fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Text of the first level-1 heading among `prose` lines.
fn heading_title<'a>(prose: &[Line<'a>]) -> Option<&'a str> {
    prose.iter().find_map(|line| heading(line.text))
}

/// Text of a level-1 heading: up to 3 spaces, `#`, a space or a tab, then the
/// text without surrounding blanks nor a final run of `#` preceded by a space.
/// `None` for any other line and for an empty heading.
fn heading(text: &str) -> Option<&str> {
    let unindented = text.trim_start_matches(' ');
    if text.len() - unindented.len() > 3 {
        return None;
    }
    let rest = unindented.strip_prefix('#')?;
    if !rest.starts_with(BLANKS) {
        return None;
    }
    let rest = rest.trim_end_matches(BLANKS);
    let unhashed = rest.trim_end_matches('#');
    let rest = if unhashed.ends_with(' ') {
        unhashed
    } else {
        rest
    };
    Some(rest.trim_matches(BLANKS)).filter(|title| !title.is_empty())
}
