/*[toml]
[dependencies]
lazy_static = "1.4.0"
regex = "1.10.5"
rs-script = { git = "https://github.com/durbanlegend/rs-script" }
*/

/// This is the actual script used to collect demo script metadata and generate
/// demo/README.md.
///
/// Strategy and grunt work thanks to ChatGPT.
//# Purpose: Document demo scripts in a demo/README.md as a guide to the user.
use lazy_static::lazy_static;
use regex::Regex;
use rs_script::{code_utils, debug_log};
use std::collections::HashMap;
use std::fs::{self, read_dir, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
struct ScriptMetadata {
    script: String,
    purpose: Option<String>,
    crates: Vec<String>,
    script_type: Option<String>,
    description: Option<String>,
}

fn parse_metadata(file_path: &Path) -> Option<ScriptMetadata> {
    let mut content = fs::read_to_string(file_path).ok()?;

    content = if content.starts_with("#!") {
        let split_once = content.split_once('\n');
        let (shebang, rust_code) = split_once.expect("Failed to strip shebang");
        debug_log!("Successfully stripped shebang {shebang}");
        rust_code.to_string()
    } else {
        content
    };

    let mut metadata = HashMap::new();
    let mut lines = Vec::<String>::new();

    for line in content.clone().lines() {
        if line.starts_with("//#") {
            let parts: Vec<&str> = line[3..].splitn(2, ':').collect();
            if parts.len() == 2 {
                metadata.insert(parts[0].trim().to_lowercase(), parts[1].trim().to_string());
            }
        } else if line.starts_with("///") || line.starts_with("//:") {
            lines.push(line[3..].to_string() + "\n");
        }
    }
    metadata.insert("description".to_string(), lines.join(""));

    let maybe_syntax_tree = code_utils::to_ast(&content);

    let crates = match maybe_syntax_tree {
        Some(ref ast) => code_utils::infer_deps_from_ast(&ast),
        None => code_utils::infer_deps_from_source(&content),
    };

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?m)^\s*(async\s+)?fn\s+main\s*\(\s*\)").unwrap();
    }
    let main_methods = match maybe_syntax_tree {
        Some(ref ast) => code_utils::count_main_methods(ast),
        None => RE.find_iter(&content).count(),
    };

    let script_type = if main_methods >= 1 {
        "Program"
    } else {
        "Snippet"
    };

    let script = format!(
        "{}",
        file_path
            .file_name()
            .expect("Error accessing filename")
            .to_string_lossy()
    );

    // eprintln!(
    //     "{script} maybe_syntax_tree.is_some(): {}",
    //     maybe_syntax_tree.is_some()
    // );

    let purpose = metadata.get("purpose");
    let description = metadata.get("description");

    Some(ScriptMetadata {
        script,
        purpose: purpose.cloned(),
        crates,
        script_type: Some(script_type.to_string()),
        description: description.cloned(),
    })
}

fn collect_all_metadata(scripts_dir: &Path) -> Vec<ScriptMetadata> {
    let mut all_metadata = Vec::new();

    for entry in read_dir(scripts_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("Parsing {:#?}", path.display());

        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Some(metadata) = parse_metadata(&path) {
                all_metadata.push(metadata);
            }
        }
    }

    all_metadata.sort_by(|a, b| a.script.partial_cmp(&b.script).unwrap());

    all_metadata
}

fn generate_readme(metadata_list: &[ScriptMetadata], output_path: &Path) {
    let mut file = File::create(output_path).unwrap();
    writeln!(file, r#"## Running the scripts

`rs-script` uses `clap` for a standard command-line interface. Try `rs-script --help` (or -h) if
you get stuck.

### In its simplest form:


    rs_script <path to script>

###### E.g.:

    rs_script demo/hello.rs

### Passing options and arguments to a script:

Use `--` to separate options and arguments meant for the script from those meant for `rs_script` itself.

###### E.g.:

demo/fib_dashu_snippet.rs expects to be passed an integer _n_ and will compute the _nth_ number in the
Fibonacci sequence.

     rs_script demo/fib_dashu_snippet.rs -- 100

### Full syntax:

    rs_script [RS-SCRIPT OPTIONS] <path to script> [-- [SCRIPT OPTIONS] <script args>]

###### E.g.:

`demo/clap_tut_builder_01.rs` is a published example from the `clap` crate.
Its command-line signature looks like this:

    clap_tut_builder_01 [OPTIONS] [name] [COMMAND]

The arguments in their short form are:

    `-c <config_file>`      an optional configuration file
    `-d` / `-dd` / `ddd`    debug, at increasing levels of verbosity
    [name]                  an optional filename
    [COMMAND]               a command (e.g. test) to run

If we were to compile `clap_tut_builder_01` as an executable (`-x` option) and then run it, we might pass
it some parameters like this:

    clap_tut_builder_01 -dd -c my.cfg my_file test -l

and get output like this:

    Value for name: my_file
    Value for config: my.cfg
    Debug mode is on
    Printing testing lists...

Running the source from rs-script looks similar, we just replace `clap_tut_builder_01` by `rs_script demo/clap_tut_builder_01.rs --`:

*rs_script demo/clap_tut_builder_01.rs --* -dd -c my.cfg my_file test -l

Any parameters for `rs_script` should go before the `--`, e.g. we may choose use -qq to suppress `rs-script` messages:

    rs_script demo/clap_tut_builder_01.rs -qq -- -dd -c my.cfg my_file test -l

which will give identical output to the above.



##### Remember to use `--` to separate options and arguments that are intended for `rs_script` from those intended for the target script.

***
## Detailed script listing

"#
    )
    .unwrap();

    for metadata in metadata_list {
        writeln!(file, "### Script: {}\n", metadata.script).unwrap();
        writeln!(
            file,
            "**Description:** {}\n",
            metadata.description.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(
            file,
            "**Purpose:** {}\n",
            metadata.purpose.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        let crates = metadata
            .crates
            .iter()
            .map(|v| format!("`{v}`"))
            .collect::<Vec<String>>();
        writeln!(file, "**Crates:** {}\n", crates.join(", ")).unwrap();
        writeln!(
            file,
            "**Type:** {}\n",
            metadata.script_type.as_ref().unwrap_or(&String::new())
        )
        .unwrap();
        writeln!(
            file,
            "**Link:** [{}](https://github.com/durbanlegend/rs-script/blob/master/demo/{})\n",
            metadata.script, metadata.script
        )
        .unwrap();
        writeln!(file, "---\n").unwrap();
    }
}

fn main() {
    let scripts_dir = Path::new("demo");
    let output_path = Path::new("demo/README.md");

    let all_metadata = collect_all_metadata(scripts_dir);
    generate_readme(&all_metadata, output_path);

    println!("demo/README.md generated successfully.");
}
