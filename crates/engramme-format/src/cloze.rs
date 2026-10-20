use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::model::{CardKind, ClozeDeletion};
use crate::source::{Line, is_blank, normalize};

/// Parses the body of a `cloze` block opened on `opening`: deletions
/// `{{cN::text}}` or `{{cN::text::hint}}`, code and LaTeX included.
/// Returns `None` when a deletion is never closed.
pub(crate) fn parse(
    path: &Path,
    opening: &Line,
    body: &[Line],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<CardKind> {
    let text = normalize(body);
    let first_line = body
        .iter()
        .find(|line| !is_blank(line.text))
        .map_or(opening.number, |line| line.number);
    let line_of = |pos: usize| first_line + text[..pos].matches('\n').count();
    let mut report = |pos: usize, kind: DiagnosticKind, message: String| {
        diagnostics.push(Diagnostic::new(path, line_of(pos), kind, message));
    };

    let bytes = text.as_bytes();
    let mut deletions = Vec::new();
    let mut candidates = 0;
    let mut pos = 0;
    while let Some(found) = text[pos..].find("{{c") {
        let start = pos + found;
        let digits_start = start + 3;
        let digits_end = digits_start
            + bytes[digits_start..]
                .iter()
                .take_while(|b| b.is_ascii_digit())
                .count();
        let digits = &text[digits_start..digits_end];
        if digits.is_empty() || !text[digits_end..].starts_with("::") {
            pos = start + 1;
            continue;
        }
        candidates += 1;
        let content_start = digits_end + 2;
        let Some(content_end) = closing(&bytes[content_start..]).map(|n| content_start + n) else {
            report(
                start,
                DiagnosticKind::InvalidClozeDeletion,
                format!("cloze deletion `{{{{c{digits}::` is never closed with `}}}}`"),
            );
            return None;
        };
        pos = content_end + 2;
        let content = &text[content_start..content_end];
        let (hidden, hint) = match content.split_once("::") {
            Some((hidden, hint)) => (hidden, (!hint.trim().is_empty()).then(|| hint.to_owned())),
            None => (content, None),
        };
        let number = match digits.parse::<u32>() {
            Ok(n) if !digits.starts_with('0') => n,
            _ => {
                report(
                    start,
                    DiagnosticKind::InvalidClozeDeletion,
                    format!(
                        "invalid cloze number `c{digits}`: numbers start at 1, without leading zero"
                    ),
                );
                continue;
            }
        };
        if hidden.trim().is_empty() {
            report(
                start,
                DiagnosticKind::InvalidClozeDeletion,
                format!("cloze deletion `{{{{c{digits}::…}}}}` has empty text"),
            );
            continue;
        }
        deletions.push(ClozeDeletion {
            number,
            text: hidden.to_owned(),
            hint,
        });
    }

    if candidates == 0 {
        diagnostics.push(Diagnostic::new(
            path,
            opening.number,
            DiagnosticKind::NoClozeDeletion,
            "cloze card has no deletion such as `{{c1::text}}`",
        ));
    } else if let Some(message) = missing_numbers(&deletions) {
        diagnostics.push(Diagnostic::new(
            path,
            opening.number,
            DiagnosticKind::NonConsecutiveClozeNumbers,
            format!("cloze numbers are not consecutive: missing {message}"),
        ));
    }
    Some(CardKind::Cloze { text, deletions })
}

/// Most missing numbers listed in a message.
const MAX_LISTED: u64 = 5;

/// The numbers missing between 1 and the largest number, e.g. `c2, c4`, or
/// `None` when there is none. Long lists are cut: `c1, …, c5 and 9 more`.
fn missing_numbers(deletions: &[ClozeDeletion]) -> Option<String> {
    let mut numbers: Vec<u32> = deletions.iter().map(|d| d.number).collect();
    numbers.sort_unstable();
    numbers.dedup();
    let mut listed = Vec::new();
    let mut total: u64 = 0;
    let mut previous = 0u32;
    for n in numbers {
        let gap = u64::from(n - previous - 1);
        for missing in (previous + 1..n).take((MAX_LISTED.saturating_sub(total)) as usize) {
            listed.push(format!("c{missing}"));
        }
        total += gap;
        previous = n;
    }
    match total {
        0 => None,
        t if t > MAX_LISTED => Some(format!("{} and {} more", listed.join(", "), t - MAX_LISTED)),
        _ => Some(listed.join(", ")),
    }
}

/// Offset of the first `}}` at brace depth zero in `bytes`.
fn closing(bytes: &[u8]) -> Option<usize> {
    let mut depth = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' if depth > 0 => depth -= 1,
            b'}' if bytes.get(i + 1) == Some(&b'}') => return Some(i),
            _ => {}
        }
    }
    None
}
