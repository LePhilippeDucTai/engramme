use std::path::Path;

use crate::block::{self, Scan};
use crate::diagnostic::{Diagnostic, DiagnosticKind, Parsed};
use crate::frontmatter;
use crate::model::{InboxProposal, ProposalFrontmatter};
use crate::source;

/// Parses a proposal of `inbox/` from its contents. Pure: reads nothing from disk.
pub fn parse_proposal(path: &Path, source: &str) -> Parsed<InboxProposal> {
    let lines = source::lines(source);
    let split = frontmatter::split(path, &lines);
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
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
        cards,
        diagnostics: block_diagnostics,
        ..
    } = block::scan(path, split.body);
    diagnostics.extend(block_diagnostics);
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
