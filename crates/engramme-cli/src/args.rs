use std::ffi::OsString;
use std::path::PathBuf;

use crate::one_line;

/// A parsed command line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    /// `-h`, `--help`.
    Help,
    /// `-V`, `--version`.
    Version,
    /// `check [<path>…]`.
    Check(Vec<PathBuf>),
}

/// Parses the arguments after the program name. The error is the message,
/// without `error: `.
pub(crate) fn parse(args: &[OsString]) -> Result<Command, String> {
    let Some((first, rest)) = args.split_first() else {
        return Err("missing command".to_owned());
    };
    match first.to_str() {
        Some("-h" | "--help") => return Ok(Command::Help),
        Some("-V" | "--version") => return Ok(Command::Version),
        Some("check") => {}
        _ if is_option(first) => return Err(format!("unknown option `{}`", one_line(first))),
        _ => return Err(format!("unknown command `{}`", one_line(first))),
    }
    let mut paths = Vec::new();
    let mut options = true;
    for arg in rest {
        if options {
            match arg.to_str() {
                Some("-h" | "--help") => return Ok(Command::Help),
                Some("--") => {
                    options = false;
                    continue;
                }
                _ if is_option(arg) => {
                    return Err(format!("unknown option `{}`", one_line(arg)));
                }
                _ => {}
            }
        }
        paths.push(PathBuf::from(arg));
    }
    Ok(Command::Check(paths))
}

fn is_option(arg: &OsString) -> bool {
    arg.as_encoded_bytes().first() == Some(&b'-')
}
