//! Parser and validator for the engramme v1 vault format.
//!
//! [`parse_note`] is pure: it takes a path and the file contents. [`read_note`]
//! is the only function that reads from disk.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod block;
mod cloze;
mod diagnostic;
mod fence;
mod frontmatter;
mod fs;
mod id;
mod model;
mod note;
mod qa;
mod source;

pub use diagnostic::{Diagnostic, DiagnosticKind, Parsed, Severity};
pub use fs::read_note;
pub use id::CardId;
pub use model::{Card, CardKind, ClozeDeletion, Frontmatter, Note};
pub use note::parse_note;
