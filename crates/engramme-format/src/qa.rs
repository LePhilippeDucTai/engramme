use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::fence::Fence;
use crate::model::CardKind;
use crate::source::{BLANKS, Line, normalize};

/// Parses the body of a `qa` block opened on `opening`: front, a `---` line,
/// back. Returns `None` after reporting an error.
pub(crate) fn parse(
    path: &Path,
    opening: &Line,
    body: &[Line],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<CardKind> {
    let Some(index) = separator(body) else {
        diagnostics.push(Diagnostic::new(
            path,
            opening.number,
            DiagnosticKind::MissingSeparator,
            "qa card has no `---` line between front and back",
        ));
        return None;
    };
    let (front, rest) = body.split_at(index);
    let (separator, back) = rest.split_first()?;
    let front = normalize(front);
    let back = normalize(back);
    if front.is_empty() {
        diagnostics.push(Diagnostic::new(
            path,
            opening.number,
            DiagnosticKind::EmptyFace,
            "qa card has an empty front",
        ));
    }
    if back.is_empty() {
        diagnostics.push(Diagnostic::new(
            path,
            separator.number,
            DiagnosticKind::EmptyFace,
            "qa card has an empty back",
        ));
    }
    (!front.is_empty() && !back.is_empty()).then_some(CardKind::Qa { front, back })
}

/// Index of the first `---` line of `body` outside code fences.
fn separator(body: &[Line]) -> Option<usize> {
    let mut fence: Option<Fence> = None;
    body.iter().position(|line| match fence {
        Some(open) => {
            if open.closes(line.text) {
                fence = None;
            }
            false
        }
        None => {
            fence = Fence::open(line);
            fence.is_none() && line.text.trim_end_matches(BLANKS) == "---"
        }
    })
}
