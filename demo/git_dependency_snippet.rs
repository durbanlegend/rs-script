/*[toml]
[dependencies]
# rev may be necessary to force cargo to update to the latest commit, see
# https://github.com/rust-lang/cargo/issues/8555
rs-script = { git = "/Users/donf/projects/rs-script" }
*/

/// `demo/git_dependency.rs` done as a snippet, just because.
//# Purpose: Demo `git` dependencies as a snippet.
use rs_script::colors;

// The colors module has a "redundant" public main method for this demo.
colors::main();
