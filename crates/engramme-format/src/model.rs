use std::path::PathBuf;

use serde::Serialize;

use crate::id::CardId;

/// A parsed note: a Markdown file under `notes/`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Note {
    /// Path of the file, as given to the parser.
    pub path: PathBuf,
    /// Displayed title: frontmatter `title`, else the first `#` heading, else the file stem.
    pub title: String,
    /// Frontmatter keys.
    pub frontmatter: Frontmatter,
    /// Cards without errors, in file order.
    pub cards: Vec<Card>,
}

/// Frontmatter of a note. Every key is optional.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Frontmatter {
    /// `title`, when it is a string.
    pub title: Option<String>,
    /// `tags`.
    pub tags: Vec<String>,
    /// `sources`.
    pub sources: Vec<String>,
    /// Unknown keys, kept as they are.
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A card block.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Card {
    /// Identifier from `{#<id>}`, if any.
    pub id: Option<CardId>,
    /// Line of the opening `:::<type>` line (1-based).
    pub line: usize,
    /// Card type and content.
    #[serde(flatten)]
    pub kind: CardKind,
}

/// Content of a card, by type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CardKind {
    /// Question and answer.
    Qa {
        /// Front side (Markdown).
        front: String,
        /// Back side (Markdown).
        back: String,
    },
    /// Text with deletions `{{cN::text}}`.
    Cloze {
        /// Full body (Markdown), deletions included.
        text: String,
        /// Deletions in text order; one number may appear several times.
        deletions: Vec<ClozeDeletion>,
    },
}

/// One deletion `{{cN::text}}` or `{{cN::text::hint}}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ClozeDeletion {
    /// `N`, at least 1.
    pub number: u32,
    /// Hidden text.
    pub text: String,
    /// Optional hint.
    pub hint: Option<String>,
}

impl Card {
    /// Identifiers reviewed for this card: `[id]` for `qa`, `id.N` for each
    /// distinct `N` of a `cloze` (ascending). Empty without an identifier.
    pub fn review_ids(&self) -> Vec<String> {
        let Some(id) = &self.id else {
            return Vec::new();
        };
        match &self.kind {
            CardKind::Qa { .. } => vec![id.as_str().to_owned()],
            CardKind::Cloze { deletions, .. } => {
                let mut numbers: Vec<u32> = deletions.iter().map(|d| d.number).collect();
                numbers.sort_unstable();
                numbers.dedup();
                numbers.into_iter().map(|n| id.sub_card(n)).collect()
            }
        }
    }
}

/// A card proposal written by an agent in `inbox/`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InboxProposal {
    /// Path of the file, as given to the parser.
    pub path: PathBuf,
    /// Frontmatter keys.
    pub frontmatter: ProposalFrontmatter,
    /// Cards without errors, in file order.
    pub cards: Vec<Card>,
}

/// Frontmatter of a proposal. Required keys are `None` when missing or invalid.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct ProposalFrontmatter {
    /// Agent that wrote the proposal, e.g. `claude`.
    pub proposed_by: Option<String>,
    /// ISO 8601 date-time of the proposal, as written.
    pub date: Option<String>,
    /// Task in progress, in one sentence.
    pub context: Option<String>,
    /// Note to complete or create, relative to the vault, under `notes/`.
    pub target: Option<String>,
    /// Why this knowledge is worth keeping.
    pub why: Option<String>,
    /// Tags to merge into the target note.
    pub tags: Vec<String>,
    /// Sources to merge into the target note.
    pub sources: Vec<String>,
    /// Unknown keys, kept as they are.
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A whole vault: the notes of `notes/` and the proposals of `inbox/`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Vault {
    /// Root directory, as given to [`read_vault`](crate::read_vault).
    pub root: PathBuf,
    /// Notes under `notes/`, subdirectories included, in walk order.
    pub notes: Vec<Note>,
    /// Proposals directly in `inbox/`, in walk order.
    pub proposals: Vec<InboxProposal>,
}
