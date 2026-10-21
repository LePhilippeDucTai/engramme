use std::path::Path;

use crate::block::{self, Scan};
use crate::datetime::is_calendar_date;
use crate::diagnostic::{Diagnostic, DiagnosticKind, Parsed};
use crate::frontmatter::{self, one_line};
use crate::model::{InboxProposal, ProposalFrontmatter};
use crate::source;

/// Parses a proposal of `inbox/` from its contents. Pure: reads nothing from disk.
pub fn parse_proposal(path: &Path, source: &str) -> Parsed<InboxProposal> {
    let lines = source::lines(source);
    let split = frontmatter::split(path, &lines);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !is_proposal_file_name(&name) {
        diagnostics.push(Diagnostic::new(
            path,
            1,
            DiagnosticKind::UnexpectedFileName,
            format!(
                "proposal file name `{}` does not match `<YYYY-MM-DD>T<HHMM>-<agent>-<topic>.md`",
                one_line(&name)
            ),
        ));
    }
    let frontmatter = match (split.yaml, split.diagnostic) {
        (None, None) => {
            diagnostics.push(Diagnostic::new(
                path,
                1,
                DiagnosticKind::MissingRequiredKey,
                "proposal has no frontmatter: add `proposed_by`, `date`, `context`, `target` and `why` between two `---` lines",
            ));
            ProposalFrontmatter::default()
        }
        (_, Some(diagnostic)) => {
            diagnostics.push(diagnostic);
            ProposalFrontmatter::default()
        }
        (Some(yaml), None) => match frontmatter::parse_fields(path, &yaml, split.closing_line) {
            Ok(fields) => frontmatter::proposal_frontmatter(path, fields, &mut diagnostics),
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                ProposalFrontmatter::default()
            }
        },
    };
    let Scan {
        mut cards,
        diagnostics: block_diagnostics,
        headers,
        ..
    } = block::scan(path, split.body);
    diagnostics.extend(block_diagnostics);
    for header in &headers {
        if let Some(content) = header.id_text() {
            diagnostics.push(Diagnostic::new(
                path,
                header.line,
                DiagnosticKind::IdInProposal,
                format!(
                    "card id `{{#{}}}` is not allowed in a proposal: remove it, only the app assigns ids",
                    content
                ),
            ));
        }
    }
    let counted: Vec<_> = headers
        .iter()
        .filter(|header| matches!(header.card_type, "qa" | "cloze"))
        .collect();
    if counted.is_empty() {
        diagnostics.push(Diagnostic::new(
            path,
            1,
            DiagnosticKind::NoCard,
            "proposal has no `qa` or `cloze` block: add 1 to 3 cards",
        ));
    } else if let Some(fourth) = counted.get(MAX_CARDS) {
        diagnostics.push(Diagnostic::new(
            path,
            fourth.line,
            DiagnosticKind::TooManyCards,
            format!(
                "proposal has {} card blocks: at most {MAX_CARDS} are allowed",
                counted.len()
            ),
        ));
    }
    cards.retain(|card| card.id.is_none());
    diagnostics.sort_by_key(|d| d.line);
    Parsed {
        value: InboxProposal {
            path: path.to_path_buf(),
            frontmatter,
            cards,
        },
        diagnostics,
    }
}

/// A proposal without frontmatter nor cards.
pub(crate) fn empty_proposal(path: &Path) -> InboxProposal {
    InboxProposal {
        path: path.to_path_buf(),
        frontmatter: ProposalFrontmatter::default(),
        cards: Vec::new(),
    }
}

/// Most card blocks in a proposal.
const MAX_CARDS: usize = 3;

/// Whether `name` is `<YYYY-MM-DD>T<HHMM>-<agent>-<topic>.md`: a valid date
/// and time, then at least two non-empty `[a-z0-9]` words separated by `-`.
fn is_proposal_file_name(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let b = stem.as_bytes();
    if b.len() < 16 {
        return false;
    }
    let digits = |range: std::ops::Range<usize>| {
        b[range.clone()].iter().all(u8::is_ascii_digit).then(|| {
            b[range]
                .iter()
                .fold(0u32, |acc, &d| acc * 10 + u32::from(d - b'0'))
        })
    };
    let shape = b[4] == b'-' && b[7] == b'-' && b[10] == b'T' && b[15] == b'-';
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute)) = (
        digits(0..4),
        digits(5..7),
        digits(8..10),
        digits(11..13),
        digits(13..15),
    ) else {
        return false;
    };
    if !shape || !is_calendar_date(year, month, day) || hour > 23 || minute > 59 {
        return false;
    }
    let words: Vec<&[u8]> = b[16..].split(|&c| c == b'-').collect();
    words.len() >= 2
        && words.iter().all(|word| {
            !word.is_empty()
                && word
                    .iter()
                    .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
        })
}
