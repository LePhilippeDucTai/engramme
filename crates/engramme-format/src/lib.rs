//! Parser and validator for the engramme v1 vault format.
//!
//! [`parse_note`] and [`parse_proposal`] are pure: they take a path and the
//! file contents. [`read_note`] and [`read_proposal`] read one file from disk.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod block;
mod cloze;
mod datetime;
mod diagnostic;
mod fence;
mod frontmatter;
mod fs;
mod id;
mod model;
mod note;
mod proposal;
mod qa;
mod source;
mod target;

pub use diagnostic::{Diagnostic, DiagnosticKind, Parsed, Severity};
pub use fs::{read_note, read_proposal};
pub use id::CardId;
pub use model::{
    Card, CardKind, ClozeDeletion, Frontmatter, InboxProposal, Note, ProposalFrontmatter,
};
pub use note::parse_note;
pub use proposal::parse_proposal;
