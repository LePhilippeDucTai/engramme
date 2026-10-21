//! Development binary: `cargo run -p engramme-cli -- check <path>`.

fn main() {
    std::process::exit(engramme_cli::run(std::env::args_os()))
}
