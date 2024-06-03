//! [dependencies]
//! bitflags = "2.5.0"
//! bpaf = { version = "0.9.11", features = ["derive"] }
//! bpaf_derive = "0.5.10"

use bitflags::bitflags;
// use bpaf::Parser;
use bpaf_derive::Bpaf;
use core::fmt;
use std::str::FromStr;

/// Script Runner
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Bpaf)]
// #[derive(Parser, Debug)]
#[bpaf(options)]
pub(crate) struct Options {
    /// Sets verbose mode
    #[bpaf(short, long)]
    pub(crate) verbose: bool,
    /// Displays timings
    #[bpaf(short, long)]
    pub(crate) timings: bool,
    /// Generates Rust source and individual cargo .toml
    #[bpaf(short, long("gen"))]
    pub(crate) generate: bool,
    /// Builds script
    #[bpaf(short, long)]
    pub(crate) build: bool,
    /// Generates, builds and runs script (default: true)
    #[bpaf(short, long, fallback(true))]
    pub(crate) all: bool,
    /// Runs compiled script
    #[bpaf(short, long)]
    pub(crate) run: bool,
    /// Sets the script to run
    #[bpaf(positional("SCRIPT"))]
    pub(crate) script: String,
    /// Sets the arguments for the script
    #[bpaf(positional("ARGS"))]
    pub(crate) args: Vec<String>,
}

pub(crate) fn get_opt() -> Options {
    options().run()
}

#[allow(dead_code)]
fn main() {
    println!("In {}", env!("CARGO_PKG_NAME"));

    let opt = get_opt();

    if opt.verbose {
        println!("Verbosity enabled");
    }

    if opt.timings {
        println!("Timings enabled");
    }

    if opt.generate {
        println!("Generating source and cargo .toml file");
    }

    if opt.build {
        println!("Building something");
    }

    if opt.run {
        println!("Running script");
    }

    println!("Running script: {}", opt.script);
    if !opt.args.is_empty() {
        println!("With arguments:");
        for arg in &opt.args {
            println!("{}", arg);
        }
    }
}

bitflags! {
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(PartialEq, Eq)]
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const RUN = 4;
        const ALL = 8;
        const VERBOSE = 16;
        const TIMINGS = 32;
    }
}

impl fmt::Debug for ProcFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl fmt::Display for ProcFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

impl FromStr for ProcFlags {
    type Err = bitflags::parser::ParseError;

    fn from_str(flags: &str) -> Result<Self, Self::Err> {
        bitflags::parser::from_str(flags)
    }
}