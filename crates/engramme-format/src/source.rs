/// Characters that count as blank: spaces and tabs.
pub(crate) const BLANKS: [char; 2] = [' ', '\t'];

/// A line of a source file, without its line ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Line<'a> {
    /// 1-based line number.
    pub(crate) number: usize,
    /// Text of the line, without `\n` or `\r\n`.
    pub(crate) text: &'a str,
}

/// Splits `source` into numbered lines, after removing a leading UTF-8 BOM.
/// Lines end with `\n` or `\r\n`.
pub(crate) fn lines(source: &str) -> Vec<Line<'_>> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    source
        .lines()
        .zip(1..)
        .map(|(text, number)| Line { number, text })
        .collect()
}

/// Whether `text` is empty or made of blanks only.
pub(crate) fn is_blank(text: &str) -> bool {
    text.trim_start_matches(BLANKS).is_empty()
}

/// Text of a card body or face: blank lines removed at both ends, the other
/// lines joined by `\n`.
pub(crate) fn normalize(mut lines: &[Line]) -> String {
    while let [first, rest @ ..] = lines
        && is_blank(first.text)
    {
        lines = rest;
    }
    while let [rest @ .., last] = lines
        && is_blank(last.text)
    {
        lines = rest;
    }
    lines
        .iter()
        .map(|line| line.text)
        .collect::<Vec<_>>()
        .join("\n")
}
