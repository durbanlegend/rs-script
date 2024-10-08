# TODO List

## High Priority

## Medium Priority
- [ ]  Add additional popular crates
- [ ]  More unit and integration tests
- [ ]  Simple demo https server
- [ ]  Consider releasing a copy of repl.rs as a demo script.
- [ ]  Raise clear_screen as an issue on termbg and supports-color crates?
- [ ]  Add FAQ
- [ ]  cat demo/fizz_buzz_gpt.rs | while read l; do thag_rs -qe "println!(\"{}\", \"$l\".to_uppercase());"; done
- [ ]  Config option for formatting main?
- [ ]  Config option for stdin -d highlighting preference
- [ ]  Add conversions to and from `runner` and `cargo-script-mvs`.
- [ ]  Own line editor for REPL, and/or REPL based on stdin.rs
- [ ]  Debug Firestorm double invocation.
- [ ]  Test [profile.dev] optimisation level
- [ ]  Check dead code & other #[cfg[allow(...)]; look into factoring over-long gen_build_run
- [ ]  Look for code smells
- [ ]  Look into fuzzing the tests such as test_merge_manifest.
- [ ]  Identify new functions requiring unit tests.
- [ ]  Document:
        /*[toml]
        [package]
        name = "dethagomizer"

        [[bin]]
        name = "dethag"
        path = "/Users/donf/projects/thag_rs/demo/dethagomizer.rs"
        */
- [ ]  Checklist for making releases:
       - Tip: disable ci.yml for Readme & similar tweaks that won't
              affect compilation.
       - Remember to update Cargo.toml version to the required release before tagging.
       - Leave it to cargo-dist to make the release.
       - To trigger cargo-dist:
         git tag v0.1.n -m "<Summary>"
         git push --tags
       - To revoke and redo:
         git tag -d v0.1.n
         git push origin --delete v0.1.n
         Tag again as above when ready.
       - Use changelog v0.1.<n-1>..HEAD to generate raw release notes.
       - Edit the release notes generated by cargo-dist on Github and add in
           own change log, edited as required from raw changelog output above.
       - Don't override release.yml, e.g. to try to add a workflow dispatch, as it's generated by cargo-dist.
       - Suggest give it a day to settle before publishing to crates.io.
       - find . -name .DS_Store -delete
       - cargo publish --no-verify

## Low Priority
- [ ]  Paste event in Windows slow or not happening?
- [ ]  How to insert line feed from keyboard to split line in reedline. (Supposedly shift+enter)
- [ ]  "edit" crate - how to reconfigure editors dynamically - instructions unclear.
- [ ]  Clap aliases not working in REPL.
- [ ]  How to navigate reedline history entry by entry instead of line by line.
- [ ]  See if with...(nu_resolve_style) methods of repl.rs can maybe use a closure to prevent lazy-static from triggering prematurely. Maybe add terminal test?

## Ideas / Future Enhancements
- [ ]  Consider supporting alternative TOML embedding keywords so we can run demo/regex_capture_toml.rs and demo/parse_script.rs_toml.rs.
- [ ]  Option to cat files before delete.
