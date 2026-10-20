/// Why `target` is not a valid proposal target, as the text that follows
/// "invalid target `…`: " in the message, or `None` when it is valid.
pub(crate) fn target_problem(target: &str) -> Option<&'static str> {
    let bytes = target.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if target.starts_with(['/', '\\']) || drive {
        return Some("expected a path relative to the vault, such as `notes/credit-risk/lgd.md`");
    }
    if target.contains('\\') {
        return Some("use `/` as path separator");
    }
    let components: Vec<&str> = target.split('/').collect();
    if components
        .iter()
        .any(|c| c.is_empty() || c.starts_with('.'))
    {
        return Some("empty, `.`, `..` and hidden path components are not allowed");
    }
    if components.len() < 2 || components[0] != "notes" {
        return Some("expected a path under `notes/`, such as `notes/credit-risk/lgd.md`");
    }
    if !components.last().is_some_and(|name| name.ends_with(".md")) {
        return Some("expected a `.md` file");
    }
    None
}
