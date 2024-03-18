use std::env;
use std::error::Error;
// use std::fs::File;
use std::process::Command;
use std::{fs, io::Write};

// use env_logger::Builder;
// use env_logger::{Env, WriteStyle};
use std::path::{Path, PathBuf}; // Use PathBuf for paths

use errors::BuildRunError;
use log::{debug, info, LevelFilter};

mod errors;

fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::Instant::now();

    // Configure log level
    // let env = Env::new().filter("RUST_LOG"); //.default_write_style_or("auto");
    // let mut binding = Builder::new();
    // let builder = binding.parse_env(env);
    // builder.write_style(WriteStyle::Always);
    // builder.init();
    // Configure log level
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // Example source code and Cargo.toml content
    let source_stem = "factorial_main"; // Replace with actual program name
    let source_name = format!("{source_stem}.rs");
    let project_dir = env::var("PWD")?; // Set during cargo build
    let project_path = PathBuf::from(project_dir);
    // let target_dir = project_path.join("target");
    let mut code_path: PathBuf = project_path.join("examples");

    code_path.push(source_name);
    let source = read_file_contents(&code_path)?;
    //       format!(
    //       r##"
    // fn main() {{
    //   println!("Hello from program {source_stem}, programmatically generated by Cargo!");
    // }}
    // "##
    //   );

    let cargo_manifest = format!(
        r##"
    [package]
    name = "{source_stem}"
    version = "0.0.1"
    edition = "2021"

    [dependencies]
    rug = {{ version = "1.24.0", features = ["integer"] }}

    [workspace]

    [[bin]]
    name = "{source_stem}"
    path = "/Users/donf/projects/build_run/.cargo/build_run/tmp_source.rs"
    "##
    );

    let result = {
        let source: &str = &source;
        let cargo_manifest: &str = &cargo_manifest;
        let build_dir = generate(source, cargo_manifest)?;
        let dur = start.elapsed();
        debug!(
            "Completed generation in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );

        let start_build = std::time::Instant::now();
        // Build the Rust program using Cargo (with manifest path)
        build(&build_dir)?;
        let dur = start_build.elapsed();
        debug!(
            "Completed build in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );

        let start_run = std::time::Instant::now();
        let run = run(source_stem, build_dir);
        let dur = start_run.elapsed();
        debug!(
            "Completed run in {}.{}s",
            dur.as_secs(),
            dur.subsec_millis()
        );
        run
    };

    match result {
        Ok(output) => {
            println!("Build output:");
            output.lines().for_each(|line| debug!("{line}"));
        }
        Err(error) => {
            println!("Error: {error}");
        }
    }
    let dur = start.elapsed();
    debug!("Completed in {}.{}s", dur.as_secs(), dur.subsec_millis());

    Ok(())
}

fn generate(source: &str, cargo_manifest: &str) -> Result<PathBuf, errors::BuildRunError> {
    let build_dir = PathBuf::from(".cargo/build_run");
    if !build_dir.exists() {
        fs::create_dir_all(&build_dir)?; // Use fs::create_dir_all for directories
    }

    let source_path = build_dir.join("tmp_source.rs");
    let mut source_file = fs::File::create(&source_path)?;
    source_file.write_all(source.as_bytes())?;
    let relative_path = source_path;
    let mut absolute_path = std::env::current_dir()?;
    absolute_path.push(relative_path);
    debug!("Absolute path of generated program: {absolute_path:?}");

    let cargo_toml_path = build_dir.join("Cargo.toml");

    // Don't overwrite Cargo.toml if not changed - see if it will remember it's compiled.
    let prev_cargo_toml = read_file_contents(&cargo_toml_path)?;
    if !cargo_manifest.eq(&prev_cargo_toml) {
        let mut cargo_toml = fs::File::create(&cargo_toml_path)?;

        cargo_toml.write_all(cargo_manifest.as_bytes())?;
        debug!("cargo_toml_path={cargo_toml_path:?}");
    }
    Ok(build_dir)
}

fn build(build_dir: &Path) -> Result<(), errors::BuildRunError> {
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
    Ok(())
}

fn run(source_stem: &str, build_dir: PathBuf) -> Result<String, errors::BuildRunError> {
    // Run the built program
    let mut run_command = Command::new(format!("./target/debug/{source_stem}"));
    run_command.current_dir(build_dir);
    let run_output = run_command.output()?;

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
    Ok(output.to_string())
}

fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}
