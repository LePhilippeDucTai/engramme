use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;
use serde_saphyr::{MessageFormatter, Spanned, UserMessageFormatter};

use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::model::Frontmatter;
use crate::source::{BLANKS, Line};

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
            "tags" | "sources" => {
                let list = match &field.value {
                    Value::Null => Some(Vec::new()),
                    Value::Array(items) => items
                        .iter()
                        .map(|item| item.as_str().map(str::to_owned))
                        .collect(),
                    _ => None,
                };
                match list {
                    Some(list) if key == "tags" => frontmatter.tags = list,
                    Some(list) => frontmatter.sources = list,
                    None => wrong_type(&field, &format!("`{key}` must be a list of strings")),
                }
            }
            _ => {
                frontmatter.extra.insert(key, field.value);
            }
        }
    }
    frontmatter
}
