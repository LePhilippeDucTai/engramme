use std::io::Write;
use std::path::PathBuf;

use engramme_format::{Severity, check_path};

/// Checks each path in order, writes one line per diagnostic then a summary,
/// and returns 1 when there is an error, 0 otherwise.
pub(crate) fn check(paths: &[PathBuf], out: &mut dyn Write) -> i32 {
    let (mut files, mut errors, mut warnings) = (0, 0, 0);
    for path in paths {
        let checked = check_path(path);
        files += checked.files;
        for diagnostic in &checked.diagnostics {
            let _ = writeln!(out, "{diagnostic}");
            match diagnostic.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
            }
        }
    }
    let _ = writeln!(
        out,
        "checked {}: {}, {}",
        plural(files, "file"),
        plural(errors, "error"),
        plural(warnings, "warning")
    );
    i32::from(errors > 0)
}

/// `1 file`, `0 files`, `2 files`.
fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}
