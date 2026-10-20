use std::fmt;

use serde::Serialize;

/// A card identifier: `c-` followed by 6 to 8 characters in `[0-9a-z]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct CardId(String);

impl CardId {
    /// Parses an identifier, returning `None` when it is malformed.
    pub fn parse(s: &str) -> Option<CardId> {
        let rest = s.strip_prefix("c-")?;
        let valid = (6..=8).contains(&rest.len())
            && rest
                .bytes()
                .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase());
        valid.then(|| CardId(s.to_owned()))
    }

    /// The identifier as written, e.g. `c-91be0q`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The identifier of cloze sub-card `n`, e.g. `c-91be0q.2`.
    pub fn sub_card(&self, n: u32) -> String {
        format!("{}.{n}", self.0)
    }
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
