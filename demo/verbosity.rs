/*[toml]
[dependencies]
rs-script = { path = "/Users/donf/projects/rs-script" }

ratatui = "=0.26.3"
# [patch.crates-io]
# ratatui = { version = "=0.26.3" }
*/

use rs_script::log;
use rs_script::logging::Verbosity;

fn main() {
    log!(Verbosity::Quiet, "Quiet message");
    log!(Verbosity::Normal, "Normal message");
    log!(Verbosity::Verbose, "Verbose message");
}
