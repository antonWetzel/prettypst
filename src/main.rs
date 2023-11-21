use clap::Parser;
use prettypst::{format, Command};

fn main() {
    match format(&Command::parse()) {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
