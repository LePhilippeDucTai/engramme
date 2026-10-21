use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticKind, Severity};
use crate::fence::Fence;
use crate::id::CardId;
use crate::model::Card;
use crate::source::{BLANKS, Line, is_blank};
use crate::{cloze, qa};

/// What [`scan`] found in a run of lines.
pub(crate) struct Scan<'a> {
    /// Cards of the blocks without errors, in file order.
    pub(crate) cards: Vec<Card>,
    /// Lines outside card blocks and code fences, stray `:::` lines excluded.
    pub(crate) prose: Vec<Line<'a>>,
    /// Diagnostics, in the order they were found.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// One header per opening line, in file order, unclosed blocks included.
    pub(crate) headers: Vec<Header<'a>>,
}

/// The parts of an opening line `:::<type> {#<id>}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Header<'a> {
    /// Line of the opening line.
    pub(crate) line: usize,
    /// Text right after `:::` up to a blank or `{`; empty when missing.
    pub(crate) card_type: &'a str,
    /// The rest after the type, without leading or trailing blanks.
    pub(crate) suite: &'a str,
}

impl<'a> Header<'a> {
    /// Splits an opening line.
    pub(crate) fn of(line: &Line<'a>) -> Header<'a> {
        let rest = header_text(line).get(3..).unwrap_or_default();
        let end = rest
            .find(|c: char| BLANKS.contains(&c) || c == '{')
            .unwrap_or(rest.len());
        let (card_type, suite) = rest.split_at(end);
        Header {
            line: line.number,
            card_type,
            suite: suite.trim_start_matches(BLANKS),
        }
    }

    /// Text between `{#` and the final `}`, even when malformed.
    pub(crate) fn id_text(&self) -> Option<&'a str> {
        self.suite.strip_prefix("{#")?.strip_suffix('}')
    }
}

/// Splits `lines` into card blocks and prose, and parses each closed block.
pub(crate) fn scan<'a>(path: &Path, lines: &[Line<'a>]) -> Scan<'a> {
    let mut scanner = Scanner {
        path,
        fence: None,
        block: None,
        scan: Scan {
            cards: Vec::new(),
            prose: Vec::new(),
            diagnostics: Vec::new(),
            headers: Vec::new(),
        },
    };
    for &line in lines {
        scanner.push(line);
    }
    scanner.finish()
}

/// A card block whose closing line has not been seen yet.
struct Block<'a> {
    /// The `:::<type>` line.
    opening: Line<'a>,
    /// Lines after the opening line, code fences included.
    body: Vec<Line<'a>>,
}

/// Type of a block whose body is parsed.
enum BodyType {
    Qa,
    Cloze,
}

/// State of [`scan`] between two lines.
struct Scanner<'p, 'a> {
    path: &'p Path,
    /// Code fence open before the current line.
    fence: Option<Fence>,
    /// Card block open before the current line.
    block: Option<Block<'a>>,
    scan: Scan<'a>,
}

impl<'a> Scanner<'_, 'a> {
    /// Reads one line: an open fence can only be closed; otherwise a fence
    /// line opens a fence, and `:::` lines in column 0 close or open blocks.
    fn push(&mut self, line: Line<'a>) {
        if let Some(fence) = self.fence {
            if fence.closes(line.text) {
                self.fence = None;
            }
            self.code(line);
        } else if let Some(fence) = Fence::open(&line) {
            self.fence = Some(fence);
            self.code(line);
        } else if let Some(rest) = line.text.strip_prefix(":::") {
            if is_blank(rest) {
                if let Some(block) = self.block.take() {
                    self.close(block);
                }
            } else {
                if let Some(outer) = self.block.take() {
                    self.report(
                        outer.opening.number,
                        DiagnosticKind::UnclosedBlock,
                        format!(
                            "card block `{}` is not closed before the block on line {}: blocks cannot be nested",
                            header_text(&outer.opening),
                            line.number
                        ),
                    );
                }
                self.scan.headers.push(Header::of(&line));
                self.block = Some(Block {
                    opening: line,
                    body: Vec::new(),
                });
            }
        } else if let Some(block) = &mut self.block {
            block.body.push(line);
        } else {
            self.scan.prose.push(line);
        }
    }

    /// Records a fence line or a line inside a fence: it belongs to the open
    /// block, if any, and is never prose.
    fn code(&mut self, line: Line<'a>) {
        if let Some(block) = &mut self.block {
            block.body.push(line);
        }
    }

    /// Parses a block whose closing line was just seen.
    fn close(&mut self, block: Block<'a>) {
        let first = self.scan.diagnostics.len();
        let Some((body_type, id)) = self.parse_header(&Header::of(&block.opening)) else {
            return;
        };
        let kind = match body_type {
            BodyType::Qa => qa::parse(
                self.path,
                &block.opening,
                &block.body,
                &mut self.scan.diagnostics,
            ),
            BodyType::Cloze => cloze::parse(
                self.path,
                &block.opening,
                &block.body,
                &mut self.scan.diagnostics,
            ),
        };
        let has_errors = self.scan.diagnostics[first..]
            .iter()
            .any(|d| d.severity == Severity::Error);
        if let Some(kind) = kind
            && !has_errors
        {
            self.scan.cards.push(Card {
                id,
                line: block.opening.number,
                kind,
            });
        }
    }

    /// Parses an opening line `:::<type> {#<id>}`. Returns the type and the
    /// identifier when the body must be parsed.
    fn parse_header(&mut self, header: &Header) -> Option<(BodyType, Option<CardId>)> {
        let line = header.line;
        let (card_type, suite) = (header.card_type, header.suite);
        if card_type.is_empty() {
            self.report(
                line,
                DiagnosticKind::InvalidBlockHeader,
                "expected a card type right after `:::`, e.g. `:::qa`",
            );
            return None;
        }
        let mut id = None;
        let mut valid_header = true;
        if let Some(content) = header.id_text() {
            id = CardId::parse(content);
            if id.is_none() {
                self.report(
                    line,
                    DiagnosticKind::MalformedId,
                    format!(
                        "malformed card id `{content}`: expected `c-` followed by 6 to 8 characters in [0-9a-z]"
                    ),
                );
            }
        } else if !suite.is_empty() {
            self.report(
                line,
                DiagnosticKind::InvalidBlockHeader,
                format!(
                    "unexpected `{suite}` after `:::{card_type}`: only `{{#<id>}}` may follow the type"
                ),
            );
            valid_header = false;
        }
        let body_type = match card_type {
            "qa" => BodyType::Qa,
            "cloze" => BodyType::Cloze,
            "free" | "estimate" => {
                self.report(
                    line,
                    DiagnosticKind::ReservedBlockType,
                    format!(
                        "card type `{card_type}` is reserved for a future version: block ignored"
                    ),
                );
                return None;
            }
            _ => {
                self.report(
                    line,
                    DiagnosticKind::UnknownBlockType,
                    format!("unknown card type `{card_type}`: expected `qa` or `cloze`"),
                );
                return None;
            }
        };
        valid_header.then_some((body_type, id))
    }

    /// Reports a block still open at the end of the lines, else a code fence
    /// still open outside any block.
    fn finish(mut self) -> Scan<'a> {
        if let Some(block) = self.block.take() {
            // A fence still open was opened inside the block: no block opens
            // while a fence is open.
            let open_fence = self
                .fence
                .map(|fence| {
                    format!(
                        " (the code fence opened on line {} is still open)",
                        fence.line
                    )
                })
                .unwrap_or_default();
            self.report(
                block.opening.number,
                DiagnosticKind::UnclosedBlock,
                format!(
                    "card block `{}` is never closed: add a `:::` line{open_fence}",
                    header_text(&block.opening)
                ),
            );
        } else if let Some(fence) = self.fence {
            self.report(
                fence.line,
                DiagnosticKind::UnclosedCodeFence,
                "code fence opened here is never closed: card blocks after it are ignored",
            );
        }
        self.scan
    }

    /// Adds a diagnostic about `line`.
    fn report(&mut self, line: usize, kind: DiagnosticKind, message: impl Into<String>) {
        self.scan
            .diagnostics
            .push(Diagnostic::new(self.path, line, kind, message));
    }
}

/// An opening line without its trailing blanks, as quoted in messages.
fn header_text<'a>(opening: &Line<'a>) -> &'a str {
    opening.text.trim_end_matches(BLANKS)
}
