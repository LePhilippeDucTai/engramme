//! Parser and validator for the engramme v1 vault format.
//!
//! [`parse_note`] is pure: it takes a path and the file contents. [`read_note`]
//! is the only function that reads from disk.
#![forbid(unsafe_code)]

mod diagnostic;
mod fs;
mod id;
mod model;
mod note;

pub use diagnostic::{Diagnostic, DiagnosticKind, Parsed, Severity};
pub use fs::read_note;
pub use id::CardId;
pub use model::{Card, CardKind, ClozeDeletion, Frontmatter, Note};
pub use note::parse_note;
