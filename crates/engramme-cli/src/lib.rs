//! The engramme command-line interface: `engramme check [<path>…]`.
//!
//! [`run`] is the entry point of the development binary and of the Python
//! package; [`run_with`] takes the environment and the output streams as
//! parameters, for tests.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod args;
mod check;

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::{Path, PathBuf};

use args::Command;

const USAGE: &str =
    "usage: engramme check [<path>...]\n       engramme -h | --help | -V | --version\n";

const BODY: &str = "Checks each path in the order given: a directory as a vault, a file as an\ninbox proposal when its directory is named inbox, else as a note. With no\npath, checks the vault named by the ENGRAMME_DIR environment variable. Put\n-- before paths that start with -.\n\nPrints one line per problem, then a summary:\n  <file>:<line>: <error|warning>: <message> [<code>]\n\nExit status: 0 when there is no error (warnings are allowed), 1 when there is\nan error, 2 when nothing was checked (invalid arguments, or no path and\nENGRAMME_DIR not set or not a directory).\n";

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the command line `args`, whose first element is the program name
/// (ignored), reading `ENGRAMME_DIR` from the environment and writing to
/// stdout and stderr. Returns the exit code: 0 when there is no error, 1 when
/// there is an error, 2 when nothing was checked.
pub fn run<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    run_with(
        args,
        std::env::var_os("ENGRAMME_DIR").as_deref(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    )
}

/// Like [`run`], with `ENGRAMME_DIR` given as `engramme_dir`. The report, the
/// help and the version go to `out`; usage errors go to `err`.
pub fn run_with<I, S>(
    args: I,
    engramme_dir: Option<&OsStr>,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let rest = args.get(1..).unwrap_or_default();
    let code = match args::parse(rest) {
        Err(message) => usage_error(err, &message, true),
        Ok(Command::Help) => {
            let _ = out.write_all(help().as_bytes());
            0
        }
        Ok(Command::Version) => {
            let _ = writeln!(out, "engramme {VERSION}");
            0
        }
        Ok(Command::Check(paths)) if !paths.is_empty() => check::check(&paths, out),
        Ok(Command::Check(_)) => match engramme_dir.filter(|dir| !dir.is_empty()) {
            None => usage_error(err, "no path given and ENGRAMME_DIR is not set", true),
            Some(dir) if !Path::new(dir).is_dir() => usage_error(
                err,
                &format!(
                    "ENGRAMME_DIR is set to `{}`, which is not a directory",
                    one_line(dir)
                ),
                false,
            ),
            Some(dir) => check::check(&[PathBuf::from(dir)], out),
        },
    };
    let _ = out.flush();
    let _ = err.flush();
    code
}

/// Writes `error: {message}` and, when `usage`, the usage; returns 2.
fn usage_error(err: &mut dyn Write, message: &str, usage: bool) -> i32 {
    let _ = writeln!(err, "error: {message}");
    if usage {
        let _ = err.write_all(USAGE.as_bytes());
    }
    2
}

fn help() -> String {
    format!("engramme {VERSION}: check an engramme vault\n\n{USAGE}\n{BODY}")
}

/// `value` as text, with line breaks replaced by spaces, for quoting in a message.
pub(crate) fn one_line(value: &OsStr) -> String {
    value.to_string_lossy().replace(['\r', '\n'], " ")
}
