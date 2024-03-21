use core::str;
use std::env;
use std::error::Error;
use std::fmt::Write;
use std::process::Command;
use std::time::Instant;
use std::{fs, io::Write as OtherWrite};
// use env_logger::Builder;
// use env_logger::{Env, WriteStyle};
use std::path::{Path, PathBuf}; // Use PathBuf for paths

use cargo_toml::Manifest;
use errors::BuildRunError;

use log::{debug, info, LevelFilter};

mod cmd_args;
mod errors;
mod toml_utils;

pub(crate) use structopt::StructOpt;
use toml::Table;

use crate::cmd_args::{Flags, GenQualifier};

const PACKAGE_DIR: &str = env!("CARGO_MANIFEST_DIR");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    configure_log();

    let gen_build_dir = format!("{}/.cargo/{PACKAGE_NAME}", PACKAGE_DIR.to_owned());
    debug!("PACKAGE_DIR={PACKAGE_DIR}");
    debug!("PACKAGE_NAME={PACKAGE_NAME}");
    debug!("VERSION={VERSION}");
    debug!("gen_build_dir={gen_build_dir:?}",);

    // Read manifest from source file
    let _ = toml_utils::read_cargo_toml();

    let source_stem = "factorial_main"; // Replace with actual program name
    let source_name = format!("{source_stem}.rs");
    let project_dir = env::var("PWD")?; // Set during cargo build
    let project_path = PathBuf::from(project_dir);
    let mut code_path: PathBuf = project_path.join("examples");

    code_path.push(source_name);
    let source = read_file_contents(&code_path)?;
    let toml_str = source
        .lines()
        .map(str::trim_start)
        .filter(|&line| line.starts_with("//!"))
        .map(|line| line.trim_start_matches('/').trim_start_matches('!'))
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        });

    debug!("Rust source manifest info (toml_str) = {toml_str}");

    let manifest = Manifest::from_str(&toml_str)?;
    debug!("manifest (from toml_str)={manifest:#?\n}");

    let source_toml = toml_str.parse::<Table>().unwrap();
    debug!("source_toml={source_toml:?}");

    let toml = toml::to_string(&source_toml).unwrap();
    debug!("Raw toml = {toml:?}\n");

    debug!("Toml reconstituted:");
    toml.lines().for_each(|l| println!("{l}"));

    let cargo_manifest = format!(
        r##"
    [package]
    name = "{source_stem}"
    version = "0.0.1"
    edition = "2021"

    [dependencies]
    rug = {{ version = "1.24.0", features = ["integer"] }}
    serde = {{ version = "1.0", features = ["derive"] }}

    [workspace]

    [[bin]]
    name = "{source_stem}"
    path = "/Users/donf/projects/build_run/.cargo/build_run/tmp_source.rs"
    "##
    );

    let source_toml = cargo_manifest.parse::<Table>().unwrap();
    debug!("source_toml={source_toml:#?}\n");

    let toml = toml::to_string(&source_toml).unwrap();
    debug!("Raw toml = {toml:#?}\n");

    debug!("cargo_manifest reconstituted:");
    toml.lines().for_each(|l| println!("{l}"));

    let source: &str = &source;
    let build_dir = PathBuf::from(".cargo/build_run");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    let options = cmd_args::Opt::from_args();

    let mut flags = Flags::empty();
    flags.set(Flags::VERBOSE, options.verbose);
    flags.set(Flags::TIMINGS, options.timings);

    let formatted = flags.to_string();
    let parsed: Flags = formatted.parse().unwrap();

    assert_eq!(flags, parsed);
    // assert!(flags.contains(Flags::VERBOSE));
    // assert!(flags.contains(Flags::TIMINGS));

    let result: Result<(), errors::BuildRunError> = match options.action {
        // Implement generate logic with optional verbose and timings
        // println!("Generating code (verbose: {}, timings: {})", verbose, timings);

        // match options.action {
        cmd_args::Action::All => {
            generate(
                &flags,
                &cmd_args::GenQualifier::Both,
                source,
                &cargo_manifest,
                &build_dir,
            )?;
            build(&flags, &build_dir)?;
            run(&flags, source_stem, build_dir)
        } /* Generate code and Cargo.toml, then build */
        cmd_args::Action::Generate(gen_qualifier) => {
            generate(&flags, &gen_qualifier, source, &cargo_manifest, &build_dir)
        }
        cmd_args::Action::Build => build(&flags, &build_dir),
        cmd_args::Action::GenAndBuild => {
            generate(
                &flags,
                &cmd_args::GenQualifier::Both,
                source,
                &cargo_manifest,
                &build_dir,
            )?;
            build(&flags, &build_dir)
        } /* Generate code and Cargo.toml, then build */
        cmd_args::Action::Run => run(&flags, source_stem, build_dir),
        cmd_args::Action::BuildAndRun => {
            build(&flags, &build_dir)?;
            run(&flags, source_stem, build_dir)
        }
        cmd_args::Action::CheckCargo => todo!(),
        cmd_args::Action::CheckSource => todo!(), /* Generate, build, and run */
    };

    match result {
        Ok(()) => {
            let dur = start.elapsed();
            debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());
        }
        Err(ref error) => {
            println!("Error: {error}");
        }
    }
    Ok(result?)
}

// Configure log level
fn configure_log() {
    // let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    // let mut binding = Builder::new();
    // let builder = binding.parse_env(env);
    // builder.write_style(WriteStyle::Always);
    // builder.init();

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();
}

fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

fn generate(
    flags: &Flags,
    gen_qualifier: &cmd_args::GenQualifier,
    source: &str,
    cargo_manifest: &str,
    build_dir: &Path,
) -> Result<(), errors::BuildRunError> {
    let start_gen = Instant::now();

    if matches!(gen_qualifier, GenQualifier::Both | GenQualifier::Source) {
        let source_path = build_dir.join("tmp_source.rs");
        let mut source_file = fs::File::create(&source_path)?;
        source_file.write_all(source.as_bytes())?;
        let relative_path = source_path;
        let mut absolute_path = std::env::current_dir()?;
        absolute_path.push(relative_path);
        debug!("Absolute path of generated program: {absolute_path:?}");
    }

    if matches!(gen_qualifier, GenQualifier::Both | GenQualifier::CargoToml) {
        let cargo_toml_path = build_dir.join("Cargo.toml");

        // Don't overwrite Cargo.toml if not changed - see if it will remember it's compiled.
        let prev_cargo_toml = read_file_contents(&cargo_toml_path)?;
        if !cargo_manifest.eq(&prev_cargo_toml) {
            let mut cargo_toml = fs::File::create(&cargo_toml_path)?;

            OtherWrite::write_all(&mut cargo_toml, cargo_manifest.as_bytes())?;
            debug!("cargo_toml_path={cargo_toml_path:?}");
        }
    }

    let dur = start_gen.elapsed();
    debug!(
        "Completed generation in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );
    if flags.contains(Flags::TIMINGS) {
        println!(
            "Completed generation in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}

// Build the Rust program using Cargo (with manifest path)
fn build(flags: &Flags, build_dir: &Path) -> Result<(), errors::BuildRunError> {
    let start_build = Instant::now();
    let mut build_command = Command::new("cargo");
    build_command
        .args(["build", "--verbose"])
        .current_dir(build_dir);
    let build_output = build_command.output()?;
    if build_output.status.success() {
        let success_msg = String::from_utf8_lossy(&build_output.stdout);
        info!("##### Build succeeded!");
        success_msg.lines().for_each(|line| {
            debug!("{line}");
        });
    } else {
        let error_msg = String::from_utf8_lossy(&build_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(errors::BuildRunError::Command(
            "Cargo build failed".to_string(),
        ));
    }

    let dur = start_build.elapsed();
    debug!(
        "Completed build in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if flags.contains(Flags::TIMINGS) {
        println!(
            "Completed build in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}

// Run the built program
fn run(flags: &Flags, source_stem: &str, build_dir: PathBuf) -> Result<(), errors::BuildRunError> {
    let start_run = Instant::now();

    let relative_path = format!("./target/debug/{source_stem}");
    let mut absolute_path = build_dir;
    absolute_path.push(relative_path);
    debug!("Absolute path of generated program: {absolute_path:?}");

    let mut run_command = Command::new(format!("{}", absolute_path.display()));
    debug!("Run command is {run_command:?}");

    let run_output = run_command.spawn()?.wait_with_output()?;

    if run_output.status.success() {
        let success_msg = String::from_utf8_lossy(&run_output.stdout);
        info!("##### Build succeeded!");
        success_msg.lines().for_each(|line| {
            debug!("{line}");
        });
    } else {
        let error_msg = String::from_utf8_lossy(&run_output.stderr);
        error_msg.lines().for_each(|line| {
            debug!("{line}");
        });
        return Err(errors::BuildRunError::Command(
            "Cargo run failed".to_string(),
        ));
    }

    let output = String::from_utf8_lossy(&run_output.stdout);

    println!("Build output:");
    output.lines().for_each(|line| debug!("{line}"));

    let dur = start_run.elapsed();
    debug!(
        "Completed run in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );

    if flags.contains(Flags::TIMINGS) {
        println!(
            "Completed run in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
    }

    Ok(())
}
