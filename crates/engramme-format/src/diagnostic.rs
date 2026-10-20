use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The file is invalid.
    Error,
    /// The file is valid but something deserves attention.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        })
    }
}

/// What a diagnostic is about. Its [`code`](DiagnosticKind::code) is stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticKind {
    /// Frontmatter is not valid YAML, is never closed, or has a key of the wrong
    /// type or a malformed `date`.
    InvalidFrontmatter,
    /// A card block has no closing `:::` line.
    UnclosedBlock,
    /// The `:::` opening line is malformed.
    InvalidBlockHeader,
    /// The card type is not known.
    UnknownBlockType,
    /// The `{#<id>}` identifier is malformed.
    MalformedId,
    /// A `qa` card has no `---` separator.
    MissingSeparator,
    /// A `qa` card has an empty side.
    EmptyFace,
    /// A `cloze` card has no deletion.
    NoClozeDeletion,
    /// A cloze deletion is empty, badly numbered or never closed.
    InvalidClozeDeletion,
    /// The file cannot be read or is not UTF-8.
    UnreadableFile,
    /// A required key of a proposal is missing or empty.
    MissingRequiredKey,
    /// The `target` of a proposal is not a `.md` path under `notes/`.
    InvalidTarget,
    /// The card type is reserved for a future version.
    ReservedBlockType,
    /// Cloze numbers skip a value.
    NonConsecutiveClozeNumbers,
    /// A code fence is never closed.
    UnclosedCodeFence,
}

impl DiagnosticKind {
    /// Stable kebab-case code, e.g. `unclosed-block`.
    pub fn code(self) -> &'static str {
        match self {
            DiagnosticKind::InvalidFrontmatter => "invalid-frontmatter",
            DiagnosticKind::UnclosedBlock => "unclosed-block",
            DiagnosticKind::InvalidBlockHeader => "invalid-block-header",
            DiagnosticKind::UnknownBlockType => "unknown-block-type",
            DiagnosticKind::MalformedId => "malformed-id",
            DiagnosticKind::MissingSeparator => "missing-separator",
            DiagnosticKind::EmptyFace => "empty-face",
            DiagnosticKind::NoClozeDeletion => "no-cloze-deletion",
            DiagnosticKind::InvalidClozeDeletion => "invalid-cloze-deletion",
            DiagnosticKind::UnreadableFile => "unreadable-file",
            DiagnosticKind::MissingRequiredKey => "missing-required-key",
            DiagnosticKind::InvalidTarget => "invalid-target",
            DiagnosticKind::ReservedBlockType => "reserved-block-type",
            DiagnosticKind::NonConsecutiveClozeNumbers => "non-consecutive-cloze-numbers",
            DiagnosticKind::UnclosedCodeFence => "unclosed-code-fence",
        }
    }

    /// Level of diagnostics of this kind.
    pub fn severity(self) -> Severity {
        match self {
            DiagnosticKind::ReservedBlockType
            | DiagnosticKind::NonConsecutiveClozeNumbers
            | DiagnosticKind::UnclosedCodeFence => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

/// A message about a file, at a 1-based line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Diagnostic {
    /// File the diagnostic is about.
    pub file: PathBuf,
    /// 1-based line, frontmatter included.
    pub line: usize,
    /// Level, derived from `kind`.
    pub severity: Severity,
    /// What it is about.
    pub kind: DiagnosticKind,
    /// Human-readable message, in English.
    pub message: String,
}

impl Diagnostic {
    /// Builds a diagnostic whose severity follows `kind`.
    pub fn new(file: &Path, line: usize, kind: DiagnosticKind, message: impl Into<String>) -> Self {
        Diagnostic {
            file: file.to_path_buf(),
            line,
            severity: kind.severity(),
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {} [{}]",
            self.file.display(),
            self.line,
            self.severity,
            self.message,
            self.kind.code()
        )
    }
}

/// A parsed value with the diagnostics found on the way.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed<T> {
    /// The parsed value.
    pub value: T,
    /// Diagnostics, sorted by line.
    pub diagnostics: Vec<Diagnostic>,
}

impl<T> Parsed<T> {
    /// Whether any diagnostic is an error.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}
