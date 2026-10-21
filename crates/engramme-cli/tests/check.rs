use std::ffi::OsStr;
use std::process::Command;

const USAGE: &str =
    "usage: engramme check [<path>...]\n       engramme -h | --help | -V | --version\n";

const HELP: &str = concat!(
    "engramme ",
    env!("CARGO_PKG_VERSION"),
    ": check an engramme vault\n\n",
    "usage: engramme check [<path>...]\n       engramme -h | --help | -V | --version\n",
    "\n",
    "Checks each path in the order given: a directory as a vault, a file as an\n",
    "inbox proposal when its directory is named inbox, else as a note. With no\n",
    "path, checks the vault named by the ENGRAMME_DIR environment variable. Put\n",
    "-- before paths that start with -.\n",
    "\n",
    "Prints one line per problem, then a summary:\n",
    "  <file>:<line>: <error|warning>: <message> [<code>]\n",
    "\n",
    "Exit status: 0 when there is no error (warnings are allowed), 1 when there is\n",
    "an error, 2 when nothing was checked (invalid arguments, or no path and\n",
    "ENGRAMME_DIR not set or not a directory).\n",
);

const VERSION: &str = concat!("engramme ", env!("CARGO_PKG_VERSION"), "\n");

/// Code, stdout and stderr of `run_with`, with `engramme` as program name.
fn cli(args: &[&str], engramme_dir: Option<&str>) -> (i32, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let argv = std::iter::once("engramme").chain(args.iter().copied());
    let code = engramme_cli::run_with(argv, engramme_dir.map(OsStr::new), &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// Code, stdout and stderr of the `engramme-dev` binary, run from the crate.
fn binary(args: &[&str], engramme_dir: Option<&str>) -> (Option<i32>, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_engramme-dev"));
    command.args(args).current_dir(env!("CARGO_MANIFEST_DIR"));
    match engramme_dir {
        Some(dir) => command.env("ENGRAMME_DIR", dir),
        None => command.env_remove("ENGRAMME_DIR"),
    };
    let output = command.output().unwrap();
    (
        output.status.code(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

fn usage_error(message: &str) -> String {
    format!("error: {message}\n{USAGE}")
}

#[test]
fn help_goes_to_stdout() {
    for args in [
        &["--help"][..],
        &["-h"],
        &["check", "--help"],
        &["check", "-h"],
        &["check", "x.md", "-h"],
        &["--help", "--bogus"],
        &["check", "-h", "--bogus"],
    ] {
        assert_eq!(
            cli(args, None),
            (0, HELP.to_owned(), String::new()),
            "{args:?}"
        );
    }
}

#[test]
fn version_goes_to_stdout() {
    for args in [&["--version"][..], &["-V"], &["-V", "check"]] {
        assert_eq!(
            cli(args, None),
            (0, VERSION.to_owned(), String::new()),
            "{args:?}"
        );
    }
}

#[test]
fn usage_errors_exit_with_2() {
    let cases: [(&[&str], &str); 10] = [
        (&[], "missing command"),
        (&["chek"], "unknown command `chek`"),
        (&["help"], "unknown command `help`"),
        (&["--bogus"], "unknown option `--bogus`"),
        (&["-x"], "unknown option `-x`"),
        (&["check", "--bogus"], "unknown option `--bogus`"),
        (&["check", "-"], "unknown option `-`"),
        (&["check", "--version"], "unknown option `--version`"),
        (&["check", "--bogus", "-h"], "unknown option `--bogus`"),
        (&["check", "a.md", "-x"], "unknown option `-x`"),
    ];
    for (args, message) in cases {
        assert_eq!(
            cli(args, None),
            (2, String::new(), usage_error(message)),
            "{args:?}"
        );
    }
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = engramme_cli::run_with(Vec::<String>::new(), None, &mut out, &mut err);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert_eq!(
        String::from_utf8(err).unwrap(),
        usage_error("missing command")
    );
}

#[test]
#[cfg(unix)]
fn non_utf8_arguments_do_not_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let run = |args: Vec<OsString>| {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = engramme_cli::run_with(args, None, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    };
    let (code, _, err) = run(vec![
        "engramme".into(),
        OsString::from_vec(b"-\xff".to_vec()),
    ]);
    assert_eq!(code, 2);
    assert!(
        err.starts_with("error: unknown option `-\u{fffd}`"),
        "{err}"
    );
    let (code, out, _) = run(vec![
        "engramme".into(),
        "check".into(),
        OsString::from_vec(b"x\xff.md".to_vec()),
    ]);
    assert_eq!(code, 1);
    assert!(
        out.starts_with("x\u{fffd}.md:1: error: cannot read file: "),
        "{out}"
    );
}

#[test]
fn missing_engramme_dir() {
    let expected = (
        2,
        String::new(),
        usage_error("no path given and ENGRAMME_DIR is not set"),
    );
    assert_eq!(cli(&["check"], None), expected);
    assert_eq!(cli(&["check", "--"], None), expected);
    assert_eq!(cli(&["check"], Some("")), expected);
}

#[test]
fn engramme_dir_must_be_a_directory() {
    for dir in ["tests/fixtures/missing", "Cargo.toml"] {
        assert_eq!(
            cli(&["check"], Some(dir)),
            (
                2,
                String::new(),
                format!("error: ENGRAMME_DIR is set to `{dir}`, which is not a directory\n")
            )
        );
    }
}

/// Checks that `out` is the `unreadable-file` line of `path` then `summary`.
fn assert_unreadable(out: &str, path: &str, summary: &str) {
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 2, "{out}");
    assert!(
        lines[0].starts_with(&format!("{path}:1: error: cannot read file: ")),
        "{out}"
    );
    assert!(lines[0].ends_with(" [unreadable-file]"), "{out}");
    assert_eq!(lines[1], summary);
    assert!(out.ends_with('\n'));
}

#[test]
fn missing_path_is_a_diagnostic() {
    for (args, path) in [
        (
            &["check", "tests/fixtures/missing.md"][..],
            "tests/fixtures/missing.md",
        ),
        (&["check", "--", "-h"], "-h"),
    ] {
        let (code, out, err) = cli(args, None);
        assert_eq!(code, 1, "{args:?}");
        assert_eq!(err, "");
        assert_unreadable(&out, path, "checked 0 files: 1 error, 0 warnings");
    }
}

#[test]
fn binary_wires_run() {
    assert_eq!(
        binary(&[], None),
        (Some(2), String::new(), usage_error("missing command"))
    );
    assert_eq!(
        binary(&["--version"], None),
        (Some(0), VERSION.to_owned(), String::new())
    );
    assert_eq!(
        binary(&["check"], Some("Cargo.toml")),
        (
            Some(2),
            String::new(),
            "error: ENGRAMME_DIR is set to `Cargo.toml`, which is not a directory\n".to_owned()
        )
    );
}
