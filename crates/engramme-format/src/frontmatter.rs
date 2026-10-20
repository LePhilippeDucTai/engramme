use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use serde_saphyr::{MessageFormatter, Spanned, UserMessageFormatter};

use crate::datetime::is_date_time;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::model::{Frontmatter, ProposalFrontmatter};
use crate::source::{BLANKS, Line};
use crate::target::target_problem;

/// A frontmatter value and the file line it starts on.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Field {
    /// The value, as JSON.
    pub(crate) value: Value,
    /// 1-based file line of the value.
    pub(crate) line: usize,
}

/// A file split into frontmatter and body.
pub(crate) struct Split<'s, 'a> {
    /// YAML text between the `---` lines, if the frontmatter is closed.
    pub(crate) yaml: Option<String>,
    /// Line of the closing `---`.
    pub(crate) closing_line: usize,
    /// Lines after the frontmatter.
    pub(crate) body: &'s [Line<'a>],
    /// Error when the frontmatter is never closed.
    pub(crate) diagnostic: Option<Diagnostic>,
}

fn is_dashes(line: &Line) -> bool {
    line.text.trim_end_matches(BLANKS) == "---"
}

/// Splits `lines`: a frontmatter exists when line 1 is `---`, and ends at the
/// next `---` line.
pub(crate) fn split<'s, 'a>(path: &Path, lines: &'s [Line<'a>]) -> Split<'s, 'a> {
    let Some((first, rest)) = lines.split_first().filter(|(first, _)| is_dashes(first)) else {
        return Split {
            yaml: None,
            closing_line: 0,
            body: lines,
            diagnostic: None,
        };
    };
    match rest.iter().position(is_dashes) {
        Some(end) => Split {
            yaml: Some(
                rest[..end]
                    .iter()
                    .map(|line| line.text)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            closing_line: rest[end].number,
            body: &rest[end + 1..],
            diagnostic: None,
        },
        None => Split {
            yaml: None,
            closing_line: 0,
            body: rest,
            diagnostic: Some(Diagnostic::new(
                path,
                first.number,
                DiagnosticKind::InvalidFrontmatter,
                "frontmatter starting on line 1 is never closed: add a `---` line after it",
            )),
        },
    }
}

/// Parses the YAML of a frontmatter into its top-level keys. Line `k` of
/// `yaml` is line `k + 1` of the file, which closes on `closing_line`.
pub(crate) fn parse_fields(
    path: &Path,
    yaml: &str,
    closing_line: usize,
) -> Result<BTreeMap<String, Field>, Diagnostic> {
    let options = serde_saphyr::options! { strict_booleans: true, with_snippet: false };
    let parsed = serde_saphyr::from_str_with_options::<Option<BTreeMap<String, Spanned<Value>>>>(
        yaml, options,
    );
    match parsed {
        Ok(map) => Ok(map
            .unwrap_or_default()
            .into_iter()
            .map(|(key, spanned)| {
                let line = file_line(spanned.referenced.line(), usize::MAX);
                (
                    key,
                    Field {
                        value: spanned.value,
                        line,
                    },
                )
            })
            .collect()),
        Err(err) => {
            let line = file_line(err.location().map_or(0, |l| l.line()), closing_line);
            let message = UserMessageFormatter
                .format_message(err.without_snippet())
                .replace(['\r', '\n'], " ");
            Err(Diagnostic::new(
                path,
                line,
                DiagnosticKind::InvalidFrontmatter,
                format!("invalid YAML in frontmatter: {message}"),
            ))
        }
    }
}

/// File line of YAML line `yaml_line` (1-based, 0 when unknown), at most `max`.
fn file_line(yaml_line: u64, max: usize) -> usize {
    match usize::try_from(yaml_line) {
        Ok(0) | Err(_) => 1,
        Ok(n) => n.saturating_add(1).min(max.max(1)),
    }
}

/// Reads the keys of a note frontmatter. Keys of the wrong type are reported
/// and dropped; unknown keys go to `extra`.
pub(crate) fn note_frontmatter(
    path: &Path,
    fields: BTreeMap<String, Field>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Frontmatter {
    let mut frontmatter = Frontmatter::default();
    let mut wrong_type = |field: &Field, message: &str| {
        diagnostics.push(Diagnostic::new(
            path,
            field.line,
            DiagnosticKind::InvalidFrontmatter,
            message,
        ));
    };
    for (key, field) in fields {
        match key.as_str() {
            "title" => match &field.value {
                Value::String(s) => frontmatter.title = Some(s.clone()),
                Value::Null => {}
                _ => wrong_type(&field, "`title` must be a string"),
            },
            "tags" | "sources" => match string_list(&field.value) {
                Some(list) if key == "tags" => frontmatter.tags = list,
                Some(list) => frontmatter.sources = list,
                None => wrong_type(&field, &format!("`{key}` must be a list of strings")),
            },
            _ => {
                frontmatter.extra.insert(key, field.value);
            }
        }
    }
    frontmatter
}

/// A list of strings: `null` is an empty list, anything else but a list of
/// strings is `None`.
fn string_list(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::Null => Some(Vec::new()),
        Value::Array(items) => items
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect(),
        _ => None,
    }
}

/// Required keys of a proposal, in the order of the spec.
const REQUIRED: [&str; 5] = ["proposed_by", "date", "context", "target", "why"];

/// Reads the keys of a proposal frontmatter. Required keys that are missing,
/// empty or invalid are reported and left `None`.
pub(crate) fn proposal_frontmatter(
    path: &Path,
    mut fields: BTreeMap<String, Field>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ProposalFrontmatter {
    let mut frontmatter = ProposalFrontmatter::default();
    let mut report = |line: usize, kind: DiagnosticKind, message: String| {
        diagnostics.push(Diagnostic::new(path, line, kind, message));
    };
    for key in REQUIRED {
        let Some(field) = fields.remove(key) else {
            report(
                1,
                DiagnosticKind::MissingRequiredKey,
                format!("missing required key `{key}` in proposal frontmatter"),
            );
            continue;
        };
        let text = match &field.value {
            Value::Null => None,
            Value::String(s) if s.trim().is_empty() => None,
            Value::String(s) => Some(s.as_str()),
            _ if key == "date" => {
                report(
                    field.line,
                    DiagnosticKind::InvalidFrontmatter,
                    date_message(),
                );
                continue;
            }
            _ => {
                report(
                    field.line,
                    DiagnosticKind::InvalidFrontmatter,
                    format!("`{key}` must be a string"),
                );
                continue;
            }
        };
        let Some(text) = text else {
            report(
                field.line,
                DiagnosticKind::MissingRequiredKey,
                format!("required key `{key}` is empty"),
            );
            continue;
        };
        if key == "date" && !is_date_time(text) {
            report(
                field.line,
                DiagnosticKind::InvalidFrontmatter,
                date_message(),
            );
            continue;
        }
        if key == "target"
            && let Some(problem) = target_problem(text)
        {
            report(
                field.line,
                DiagnosticKind::InvalidTarget,
                format!("invalid target `{}`: {problem}", one_line(text)),
            );
            continue;
        }
        let value = Some(text.to_owned());
        match key {
            "proposed_by" => frontmatter.proposed_by = value,
            "date" => frontmatter.date = value,
            "context" => frontmatter.context = value,
            "target" => frontmatter.target = value,
            _ => frontmatter.why = value,
        }
    }
    for (key, field) in fields {
        match key.as_str() {
            "tags" | "sources" => match string_list(&field.value) {
                Some(list) if key == "tags" => frontmatter.tags = list,
                Some(list) => frontmatter.sources = list,
                None => report(
                    field.line,
                    DiagnosticKind::InvalidFrontmatter,
                    format!("`{key}` must be a list of strings"),
                ),
            },
            _ => {
                frontmatter.extra.insert(key, field.value);
            }
        }
    }
    frontmatter
}

fn date_message() -> String {
    "`date` must be an ISO 8601 date-time such as `2026-10-20T14:32:00+02:00`".to_owned()
}

/// `text` with line breaks replaced by spaces, for quoting in a message.
pub(crate) fn one_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}
