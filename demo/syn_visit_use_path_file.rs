/*[toml]
[dependencies]
quote = "1.0.36"
syn = { version = "2.0.71", features = ["extra-traits", "full", "parsing", "visit", "visit-mut"] }
*/

use std::{env, fs, path::PathBuf};

fn main() {
    use ::syn::{visit::*, *};

    let mut args = env::args_os();
    let _ = args.next(); // executable name

    let filepath = match (args.next(), args.next()) {
        (Some(arg), None) => PathBuf::from(arg),
        _ => panic!("Couldn't find filepath arg"),
    };
    let content = fs::read_to_string(&filepath).expect("Error reading file");
    let code: File = parse_file(&content).unwrap();

    struct FindCrates;
    impl<'ast> Visit<'ast> for FindCrates {
        fn visit_use_path(&mut self, node: &'ast syn::UsePath) {
            println!("{:#?}", node);
        }
    }

    FindCrates.visit_file(&code);
}
