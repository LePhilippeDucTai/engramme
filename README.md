# engramme

engramme is spaced repetition (FSRS) on top of a knowledge base written in
Markdown. AI agents propose flashcards while you work with them; you triage
the proposals, which is itself a moment of learning; then you review the cards
you kept. A desktop app is planned. engramme is a personal tool, released
under the MIT license.

## Status

Sub-project 1 of 6 is done: the v1 format of the vault, the `engramme check`
validator, and the capture skill that lets agents propose cards. There is no
app yet: you triage `inbox/` by hand, copying the cards you keep into the
target note, then deleting the proposal file.

## Roadmap

1. Format and capture skill: the v1 vault format, `engramme check`, the
   `engramme-capture` skill (done).
2. App MVP: Q/A and cloze review with FSRS, inbox triage, note browsing, and a
   local web fallback.
3. Offline iPhone app for the personal vault.
4. Optional AI tutor: free answers commented by a model, explanations,
   exercise variants.
5. Order-of-magnitude cards and statistics.
6. Advanced knowledge management: `[[links]]`, backlinks, search, graph.

## How it works

A vault is a folder, named by the `ENGRAMME_DIR` environment variable:
`notes/` holds your Markdown notes and their card blocks (`:::qa`,
`:::cloze`), `inbox/` holds the proposals written by agents, and `.app/log/`
will hold the review history. Agents write only to `inbox/`, through the
capture skill; you triage; the app will schedule the reviews.

## Install

Wheels for macOS arm64 and Windows x64 (Python 3.9 or later) are attached to
the [GitHub releases](https://github.com/LePhilippeDucTai/engramme/releases).
Download the one for your platform, then install it with pip or uv:

```sh
pip install engramme-0.1.0-cp39-abi3-macosx_11_0_arm64.whl   # or ...-win_amd64.whl
uv tool install ./engramme-0.1.0-cp39-abi3-macosx_11_0_arm64.whl
```

To build the wheel from the sources (Rust and uv required):

```sh
uvx maturin build --release -m crates/engramme-py/Cargo.toml --out dist
```

## Usage

```sh
export ENGRAMME_DIR="$HOME/vault"
engramme check                  # checks the vault named by ENGRAMME_DIR
engramme check <path>...        # checks vaults, notes or inbox files
engramme check examples/savoir
# checked 6 files: 0 errors, 0 warnings
```

Each problem is printed on one line as
`<file>:<line>: <error|warning>: <message> [<code>]`, followed by a summary.
The exit status is 0 when there is no error (warnings are allowed), 1 when
there is an error, and 2 when nothing was checked (invalid arguments, or no
path and `ENGRAMME_DIR` not set or not a directory).

## Documentation

- [The v1 vault format](docs/format.md)
- [The capture skill](skill/SKILL.md) and [how to install it](skill/INSTALL.md)
- [An example vault](examples/savoir/)

## Development

- `crates/engramme-format`: the parser and validator of the v1 format.
- `crates/engramme-cli`: the `engramme check` command line.
- `crates/engramme-py`: the Python package, built with maturin and PyO3.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The `wheels` GitHub Actions workflow runs these checks, builds the macOS and
Windows wheels, tests each one, and publishes a release on `v*` tags.

## License

[MIT](LICENSE)
