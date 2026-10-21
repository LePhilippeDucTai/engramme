use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Parsed};
use crate::frontmatter::one_line;
use crate::fs::{read_note, read_proposal};
use crate::id::CardId;
use crate::model::{Note, Vault};

/// Reports the card ids used more than once among `notes`. Pure. Notes are
/// taken in the given order, their cards in file order, and cards without an
/// id are ignored. Every occurrence after the first is a `duplicate-id` error
/// that cites the first one.
pub fn duplicate_ids(notes: &[Note]) -> Vec<Diagnostic> {
    let mut first: HashMap<&CardId, (&Path, usize)> = HashMap::new();
    let mut diagnostics = Vec::new();
    for note in notes {
        for card in &note.cards {
            let Some(id) = &card.id else {
                continue;
            };
            match first.entry(id) {
                Entry::Vacant(entry) => {
                    entry.insert((note.path.as_path(), card.line));
                }
                Entry::Occupied(entry) => {
                    let (file, line) = *entry.get();
                    diagnostics.push(Diagnostic::new(
                        &note.path,
                        card.line,
                        DiagnosticKind::DuplicateId,
                        format!(
                            "duplicate card id `{id}`: first used at {}:{line}",
                            one_line(&file.display().to_string())
                        ),
                    ));
                }
            }
        }
    }
    diagnostics
}

/// Reads and checks a whole vault: the `.md` files under `notes/` as notes,
/// those directly in `inbox/` as proposals, then the duplicate card ids among
/// the notes. Hidden entries, subdirectories of `inbox/`, other files and
/// symbolic links to anything but a regular file are skipped. Diagnostics
/// are sorted by file, then line.
pub fn read_vault(root: &Path) -> Parsed<Vault> {
    let mut vault = Vault {
        root: root.to_path_buf(),
        notes: Vec::new(),
        proposals: Vec::new(),
    };
    if let Err(err) = std::fs::read_dir(root) {
        return Parsed {
            value: vault,
            diagnostics: vec![unreadable_directory(root, &err)],
        };
    }
    let mut walk_diagnostics = Vec::new();
    let mut note_files = Vec::new();
    let mut proposal_files = Vec::new();
    let has_notes = walk(
        &root.join("notes"),
        true,
        &mut note_files,
        &mut walk_diagnostics,
    );
    let has_inbox = walk(
        &root.join("inbox"),
        false,
        &mut proposal_files,
        &mut walk_diagnostics,
    );
    if !has_notes && !has_inbox {
        walk_diagnostics.push(Diagnostic::new(
            root,
            1,
            DiagnosticKind::NotAVault,
            "no `notes/` or `inbox/` directory: nothing to check",
        ));
    }
    let mut diagnostics = Vec::new();
    for path in &note_files {
        let parsed = read_note(path);
        diagnostics.extend(parsed.diagnostics);
        vault.notes.push(parsed.value);
    }
    for path in &proposal_files {
        let parsed = read_proposal(path);
        diagnostics.extend(parsed.diagnostics);
        vault.proposals.push(parsed.value);
    }
    diagnostics.append(&mut walk_diagnostics);
    diagnostics.extend(duplicate_ids(&vault.notes));
    diagnostics.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Parsed {
        value: vault,
        diagnostics,
    }
}

/// What [`check_path`] found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Checked {
    /// Files checked: the `.md` files read in a vault, 1 for a file, 0 when
    /// the path cannot be read.
    pub files: usize,
    /// Diagnostics, sorted as described for [`check_path`].
    pub diagnostics: Vec<Diagnostic>,
}

/// Checks a path given to `engramme check`. A directory is read as a vault by
/// [`read_vault`]. A file is a proposal when its parent directory is named
/// exactly `inbox`, else a note, which only sees its own duplicate ids. A
/// path that cannot be read yields a single `unreadable-file` error. It also
/// counts the files checked.
pub fn check_path(path: &Path) -> Checked {
    let metadata = match std::fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Checked {
                files: 0,
                diagnostics: vec![Diagnostic::new(
                    path,
                    1,
                    DiagnosticKind::UnreadableFile,
                    format!("cannot read file: {err}"),
                )],
            };
        }
    };
    if metadata.is_dir() {
        let parsed = read_vault(path);
        return Checked {
            files: parsed.value.notes.len() + parsed.value.proposals.len(),
            diagnostics: parsed.diagnostics,
        };
    }
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    if absolute.parent().and_then(Path::file_name) == Some(OsStr::new("inbox")) {
        return Checked {
            files: 1,
            diagnostics: read_proposal(path).diagnostics,
        };
    }
    let Parsed {
        value: note,
        mut diagnostics,
    } = read_note(path);
    diagnostics.extend(duplicate_ids(std::slice::from_ref(&note)));
    diagnostics.sort_by_key(|d| d.line);
    Checked {
        files: 1,
        diagnostics,
    }
}

/// Adds to `files` the `.md` files of `dir`, by name, descending into
/// subdirectories when `recursive`. Returns `false` when `dir` does not exist.
/// Entries whose name starts with `.` are skipped; a symbolic link counts only
/// when it leads to a regular file.
fn walk(
    dir: &Path,
    recursive: bool,
    files: &mut Vec<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return false,
        Err(err) => {
            diagnostics.push(unreadable_directory(dir, &err));
            return true;
        }
    };
    let mut entries = Vec::new();
    for entry in read_dir {
        match entry {
            Ok(entry) => entries.push((entry.file_name(), entry)),
            Err(err) => diagnostics.push(unreadable_directory(dir, &err)),
        }
    }
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (name, entry) in entries {
        if name.as_encoded_bytes().first() == Some(&b'.') {
            continue;
        }
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                diagnostics.push(unreadable_directory(dir, &err));
                continue;
            }
        };
        let path = entry.path();
        if file_type.is_dir() {
            if recursive {
                walk(&path, recursive, files, diagnostics);
            }
        } else if path.extension() == Some(OsStr::new("md"))
            && (file_type.is_file()
                || (file_type.is_symlink()
                    && std::fs::metadata(&path).is_ok_and(|metadata| metadata.is_file())))
        {
            files.push(path);
        }
    }
    true
}

/// The `unreadable-file` error of a directory that cannot be listed.
fn unreadable_directory(dir: &Path, err: &io::Error) -> Diagnostic {
    Diagnostic::new(
        dir,
        1,
        DiagnosticKind::UnreadableFile,
        format!("cannot read directory: {err}"),
    )
}
