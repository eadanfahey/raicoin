extern crate raicoin;
use raicoin::cli;
use std::process;

fn main() {
    cli::run().unwrap_or_else(|err| {
        println!("{}", err.to_string());
        process::exit(1);
    });
}
