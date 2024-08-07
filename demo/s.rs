/*[toml]
[dependencies]
bitflags = "2.5.0"
clap = { version = "4.5.7", features = ["cargo", "derive"] }
rs-script = { path = "/Users/donf/projects/rs-script" }

*/

/// A prototype of the cmd_args module of rs-script itself.
///
/// E.g. `rs_script -tv demo/cmd_args.rs -- -gbrtv demo/hello.rs -- -fq Hello world`
//# Purpose: Prototype CLI.
use rs_script::errors::BuildRunError;
use rs_script::log;
use rs_script::logging::{self, Verbosity};
use rs_script::RS_SUFFIX;

use bitflags::bitflags;
use clap::Parser;
use core::{fmt, str};
use std::error::Error;

// rs-script script runner and REPL
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Parser, Debug)]
#[command(name = "rs_script")]
pub struct Cli {
    /// Optional name of a script to run
    pub script: Option<String>,
    /// Set the arguments for the script
    #[arg(last = true)]
    pub args: Vec<String>,
    /// Set verbose mode
    #[arg(short, long)]
    pub verbose: bool,
    /// Display timings
    #[arg(short, long)]
    pub timings: bool,
    /// Generate Rust source and individual cargo .toml if compiled file is stale
    #[arg(short = 'g', long = "gen")]
    pub generate: bool,
    /// Build script if compiled file is stale
    #[arg(short, long)]
    pub build: bool,
    /// Force generation of Rust source and individual Cargo.toml, and build, even if compiled file is not stale
    #[arg(short, long)]
    pub force: bool,
    ///  (Default) Carry out generation and build steps (if necessary or forced) and run the compiled script
    #[arg(short, long, default_value = "true")]
    pub all: bool,
    /// Run without rebuilding if already compiled
    #[arg(short, long)]
    pub run: bool,
    /// REPL mode (read–eval–print loop) for Rust expressions. Option: existing script name.
    #[arg(short = 'l', long, conflicts_with_all(["all", "generate", "build", "run"]))]
    pub repl: bool,
    /// Evaluate a quoted expression on the fly
    #[arg(short, long = "expr", conflicts_with_all(["all", "generate", "build", "run", "repl", "script", "stdin"]))]
    pub expression: Option<String>,
    /// Read script from stdin.
    #[arg(short, long, conflicts_with_all(["all", "expression", "generate", "build", "run", "repl", "script"]))]
    pub stdin: bool,
    /// Read script from stdin with edit.
    #[arg(short = 'd', long, conflicts_with_all(["all", "expression", "generate", "build", "run", "repl", "script"]))]
    pub edit: bool,
    /// Suppress unnecessary output
    #[arg(short, long, conflicts_with("verbose"))]
    pub quiet: bool,
}

// Getter for clap command-line arguments
pub fn get_args() -> Cli {
    Cli::parse()
}

pub fn validate_args(args: &Cli, proc_flags: &ProcFlags) -> Result<(), Box<dyn Error>> {
    if let Some(ref script) = args.script {
        if !script.ends_with(RS_SUFFIX) {
            return Err(Box::new(BuildRunError::Command(format!(
                "Script name must end in {RS_SUFFIX}"
            ))));
        }
    } else if !proc_flags.contains(ProcFlags::EXPR)
        && !proc_flags.contains(ProcFlags::REPL)
        && !proc_flags.contains(ProcFlags::STDIN)
        && !proc_flags.contains(ProcFlags::EDIT)
    {
        return Err(Box::new(BuildRunError::Command(
            "Missing script name".to_string(),
        )));
    }
    Ok(())
}

bitflags! {
    // You can `#[derive]` the `Debug` trait, but implementing it manually
    // can produce output like `A | B` instead of `Flags(A | B)`.
    // #[derive(Debug)]
    #[derive(Clone, PartialEq, Eq)]
    /// Processing flags for ease of handling command-line options
    pub struct ProcFlags: u32 {
        const GENERATE = 1;
        const BUILD = 2;
        const FORCE = 4;
        const RUN = 8;
        const ALL = 16;
        const VERBOSE = 32;
        const TIMINGS = 64;
        const REPL = 128;
        const EXPR = 256;
        const STDIN = 512;
        const EDIT = 1024;
        const QUIET = 2048;
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

impl str::FromStr for ProcFlags {
    type Err = bitflags::parser::ParseError;

    fn from_str(flags: &str) -> Result<Self, Self::Err> {
        bitflags::parser::from_str(flags)
    }
}

// Set up the processing flags from the command line arguments and pass them back.
// # Errors
//
// Will return `Err` if there is an error parsing the flags to set up and internal
// correctness chack.
// # Panics
//
// Will panic if the internal correctness check fails.
pub fn get_proc_flags(args: &Cli) -> Result<ProcFlags, Box<dyn Error>> {
    let is_expr = args.expression.is_some();
    let proc_flags = {
        let mut proc_flags = ProcFlags::empty();
        // TODO: out? once clap default_value_ifs is working
        proc_flags.set(
            ProcFlags::GENERATE,
            args.generate | args.force | args.all | is_expr,
        );
        proc_flags.set(
            ProcFlags::BUILD,
            args.build | args.force | args.all | is_expr,
        );
        proc_flags.set(ProcFlags::FORCE, args.force);
        proc_flags.set(ProcFlags::QUIET, args.quiet);
        proc_flags.set(ProcFlags::VERBOSE, args.verbose);
        proc_flags.set(ProcFlags::TIMINGS, args.timings);
        proc_flags.set(ProcFlags::RUN, args.run | args.all);
        proc_flags.set(ProcFlags::ALL, args.all);
        if !(proc_flags.contains(ProcFlags::ALL)) {
            proc_flags.set(ProcFlags::ALL, args.generate & args.build & args.run);
        }
        proc_flags.set(ProcFlags::REPL, args.repl);
        proc_flags.set(ProcFlags::EXPR, is_expr);
        proc_flags.set(ProcFlags::STDIN, args.stdin);
        proc_flags.set(ProcFlags::EDIT, args.edit);

        let verbosity = if args.verbose {
            Verbosity::Verbose
        } else if args.quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        };
        logging::set_global_verbosity(verbosity);

        // Check all good
        let formatted = proc_flags.to_string();
        let parsed = formatted
            .parse::<ProcFlags>()
            .map_err(|e| BuildRunError::FromStr(e.to_string()))?;

        assert_eq!(proc_flags, parsed);

        Ok::<ProcFlags, BuildRunError>(proc_flags)
    }?;
    Ok(proc_flags)
}

#[allow(dead_code)]
fn main() {
    let opt = Cli::parse();

    if opt.verbose {
        log!(Verbosity::Normal, "Verbosity enabled");
    }

    if opt.timings {
        log!(Verbosity::Normal, "Timings enabled");
    }

    if opt.generate {
        log!(Verbosity::Normal, "Generating source and cargo .toml file");
    }

    if opt.build {
        log!(Verbosity::Normal, "Build option selected");
    }

    if opt.force {
        log!(Verbosity::Normal, "Force option selected");
    }

    if opt.run {
        log!(Verbosity::Normal, "Run option selected");
    }

    log!(Verbosity::Normal, "Script to run: {:?}", opt.script);
    if !opt.args.is_empty() {
        log!(Verbosity::Normal, "With arguments:");
        for arg in &opt.args {
            log!(Verbosity::Normal, "{arg}");
        }
    }
}
