use crate::source::{Line, is_blank};

/// An open code fence: a run of at least 3 backticks or tildes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fence {
    /// Fence character, `` ` `` or `~`.
    ch: char,
    /// Length of the opening run.
    len: usize,
    /// Line of the opening fence.
    pub(crate) line: usize,
}

impl Fence {
    /// The fence opened by `line`: up to 3 spaces, then at least 3 backticks
    /// (with no other backtick on the line) or 3 tildes.
    pub(crate) fn open(line: &Line) -> Option<Fence> {
        let (ch, len, rest) = run(line.text)?;
        if len < 3 || (ch == '`' && rest.contains('`')) {
            return None;
        }
        Some(Fence {
            ch,
            len,
            line: line.number,
        })
    }

    /// Whether `text` closes this fence: up to 3 spaces, then at least as many
    /// of the same character, then only blanks.
    pub(crate) fn closes(&self, text: &str) -> bool {
        run(text).is_some_and(|(ch, len, rest)| ch == self.ch && len >= self.len && is_blank(rest))
    }
}

/// After up to 3 spaces, a run of backticks or tildes: its character, its
/// length and the rest of the line.
fn run(text: &str) -> Option<(char, usize, &str)> {
    let unindented = text.trim_start_matches(' ');
    if text.len() - unindented.len() > 3 {
        return None;
    }
    let ch = unindented
        .chars()
        .next()
        .filter(|&c| c == '`' || c == '~')?;
    let rest = unindented.trim_start_matches(ch);
    Some((ch, unindented.len() - rest.len(), rest))
}
