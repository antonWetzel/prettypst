#![doc = include_str!("../readme.md")]
#![forbid(unsafe_code, clippy::unwrap_used)]
use clap::Parser;
use prettypst::{Command, format};

fn main() {
    match format(&Command::parse()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Formatting failed because {err}.");
            std::process::exit(1);
        }
    }
}
