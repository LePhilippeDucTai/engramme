use std::path::Path;

use crate::diagnostic::Parsed;
use crate::model::{Frontmatter, Note};

/// Parses a note from its contents. Pure: reads nothing from disk.
pub fn parse_note(path: &Path, _source: &str) -> Parsed<Note> {
    Parsed {
        value: empty_note(path),
        diagnostics: Vec::new(),
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
