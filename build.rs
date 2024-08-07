use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

#[allow(clippy::doc_markdown)]
/// Create a separate test for each individual script in demo/, to ensure that it builds
/// successfully. We don't try to run them for logistical reasons, but at least we
/// identify abandoned scripts. Given that there are so many of these scripts, avoid
/// Cargo's default behaviour of running all tests in parallel. --test-threads=3 seems
/// to work best on my MacBook Air M1.
/// Suggested command: `RUST_LOG=rs_script=debug cargo test --features=debug-logs -- --nocapture --test-threads=3
/// You may want to adjust the test-threads value further depending on your hardware.
fn main() {
    // Get the OUT_DIR environment variable
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    eprintln!("OUT_DIR={out_dir}");
    fs::create_dir_all(&out_dir).expect("Failed to create destination directory");
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut file = fs::File::create(&dest_path).expect("Failed to create generated_tests.rs");

    let demo_dir = Path::new("demo");
    assert!(
        demo_dir.exists() && demo_dir.is_dir(),
        "demo directory does not exist"
    );

    let skip_scripts_on_windows = [
        "crossbeam_channel_stopwatch.rs",
        "factorial_main_rug.rs",
        "factorial_main_rug_product.rs",
        "fib_big_clap_rug.rs",
        "fib_doubling_iterative_purge_rug.rs",
        "fib_fac_rug.rs",
        "fib_matrix_rug.rs",
        "rug_arbitrary_precision_nums.rs",
    ];

    let multimain = ["flume_async.rs", "flume_select.rs"];

    let stable_only = ["duration_main.rs", "duration_snippet.rs"];

    /*
    let source_stem: &str = source_name
        .strip_suffix(rs_script::RS_SUFFIX)
        .expect("Problem stripping Rust suffix");
    let target_dir_path = TMPDIR
        .join("rs-script")
        .join(source_stem)
        .join("target/debug");
    let target_path = if cfg!(windows) {
        target_dir_path.join(source_stem.to_string() + ".exe")
    } else {
        target_dir_path.join(&source_stem)
    };
    */

    for entry in fs::read_dir(demo_dir).expect("Failed to read demo directory") {
        let entry = entry.expect("Failed to get directory entry");
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let source_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .expect("Failed to get source file name");

            // Skip scripts on Windows
            if cfg!(target_os = "windows") && skip_scripts_on_windows.contains(&source_name) {
                continue;
            }

            // Skip nightly-only scripts if on stable config
            if cfg!(not(feature = "nightly")) && stable_only.contains(&source_name) {
                eprintln!("Skipping nightly-only test {source_name}");
                continue;
            }

            let test_name = source_name.replace('.', "_");

            writeln!(
                file,
                r#"
#[test]
fn check_{test_name}() {{
    {{
        use std::process::Command;
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("-cfgnq{more_options}")
            .arg({source_path:?})
            .output()
            .expect("Failed to execute command");

        if !output.status.success() {{
            panic!(
                "Failed to build file: {source_name}\nstdout: {{}}\nstderr: {{}}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }}

        // // Get the file stem
        // let file_stem = {source_name:?}.trim_end_matches(".rs");

        // // Construct the destination directory path
        // let mut dest_dir = env::temp_dir();
        // dest_dir.push("rs-script");
        // dest_dir.push(file_stem);

        // let target_dir = &dest_dir.join("target/debug");
        // // Delete the destination directory after building the file
        // if let Err(e) = fs::remove_dir_all(&target_dir) {{
        //     eprintln!("Failed to remove directory {test_name}: {{}}, {{e:?}}", dest_dir.display());
        // }}
    }}
}}
"#,
                // source_name = &source_name,
                source_path = &path.to_str().expect("Failed to get source path"),
                more_options = if multimain.contains(&source_name) {
                    "m"
                } else {
                    ""
                }
            )
            .expect("Failed to write test function");
        }
    }
}
