use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Parsed};
use crate::model::Note;
use crate::note::{empty_note, parse_note};

/// Reads and parses a note. Unreadable or non-UTF-8 files yield an empty note
/// and an `unreadable-file` error.
pub fn read_note(path: &Path) -> Parsed<Note> {
    let unreadable = |line: usize, message: String| Parsed {
        value: empty_note(path),
        diagnostics: vec![Diagnostic::new(
            path,
            line,
            DiagnosticKind::UnreadableFile,
            message,
        )],
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => return unreadable(1, format!("cannot read file: {err}")),
    };
    match std::str::from_utf8(&bytes) {
        Ok(source) => parse_note(path, source),
        Err(err) => {
            let line = 1 + bytes[..err.valid_up_to()]
                .iter()
                .filter(|&&b| b == b'\n')
                .count();
            unreadable(line, "file is not valid UTF-8".to_owned())
        }
    }
}
