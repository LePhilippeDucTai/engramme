use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Parsed};
use crate::model::{InboxProposal, Note};
use crate::note::{empty_note, parse_note};
use crate::proposal::{empty_proposal, parse_proposal};

/// Reads a file as UTF-8, or returns the `unreadable-file` error.
fn read_source(path: &Path) -> Result<String, Diagnostic> {
    let unreadable = |line: usize, message: String| {
        Diagnostic::new(path, line, DiagnosticKind::UnreadableFile, message)
    };
    let bytes =
        std::fs::read(path).map_err(|err| unreadable(1, format!("cannot read file: {err}")))?;
    String::from_utf8(bytes).map_err(|err| {
        let bytes = err.as_bytes();
        let valid = err.utf8_error().valid_up_to();
        let line = 1 + bytes[..valid].iter().filter(|&&b| b == b'\n').count();
        unreadable(line, "file is not valid UTF-8".to_owned())
    })
}

/// Reads and parses a note. Unreadable or non-UTF-8 files yield an empty note
/// and an `unreadable-file` error.
pub fn read_note(path: &Path) -> Parsed<Note> {
    match read_source(path) {
        Ok(source) => parse_note(path, &source),
        Err(diagnostic) => Parsed {
            value: empty_note(path),
            diagnostics: vec![diagnostic],
        },
    }
}

/// Reads and parses a proposal of `inbox/`. Unreadable or non-UTF-8 files
/// yield an empty proposal and an `unreadable-file` error.
pub fn read_proposal(path: &Path) -> Parsed<InboxProposal> {
    match read_source(path) {
        Ok(source) => parse_proposal(path, &source),
        Err(diagnostic) => Parsed {
            value: empty_proposal(path),
            diagnostics: vec![diagnostic],
        },
    }
}
