//! The `engramme` Python package. Its `engramme` command, a console script
//! declared in `pyproject.toml`, passes `sys.argv` to [`engramme_cli::run`].
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use pyo3::pymodule;

/// The Python module `engramme`.
#[pymodule]
mod engramme {
    use std::ffi::OsString;

    use pyo3::prelude::*;

    /// Runs the command line `sys.argv` and returns its exit code. The console
    /// script passes the code to `sys.exit`, so Python exits normally.
    #[pyfunction]
    fn main(py: Python<'_>) -> PyResult<i32> {
        let argv: Vec<OsString> = py.import("sys")?.getattr("argv")?.extract()?;
        Ok(engramme_cli::run(argv))
    }
}
