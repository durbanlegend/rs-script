# rs-script

[![Crates.io](https://img.shields.io/crates/v/rs-script.svg)](https://crates.io/crates/rs-script)
[![Documentation](https://docs.rs/rs-script/badge.svg)](https://docs.rs/\n)
[![Build Status](https://github.com/durbanlegend/rs-script/workflows/CI/badge.svg)](https://github.com/durbanlegend/rs-script/actions)

## Intro

`rs-script` is a versatile script runner and REPL for Rust expressions, snippets, and programs. It's a developer tool that allows you to run and test Rust code from the command line for rapid prototyping and exploration. It aims to handle cases that are beyond the scope of the Rust playground or the average script runner, while hopefully being simple and convenient to use.

`rs-script` includes a demo library of over 150 sample scripts. If you've got something good to share, do feel free to offer it, subject to the MIT / Apache 2 licence terms.

## Quick start: ways to run `rs-script`

### * With an expression argument:

```bash
rs_script --expr '"Hello world!"'                                   # Short form: -e
```
![Repl](hellow.png)

Invoking quiet mode `(--quiet (-q)` suppresses feedback.

By default, `rs-script` and Cargo will feed back to you:

```bash
rs_script -e ' {
use jiff::{Zoned, Unit};
Zoned::now().round(Unit::Second)?
}'                                                                  # Long form: --expr
```
![Repl](jiffw.png)

### * With a script:

```bash
rs_script demo/iced_tour.rs
```

### * As a REPL (Read-Evaluate-Print Loop):

```bash
rs_script --repl                                                    # Short form: -l
```
![Repl](replw.png)

The REPL has file-backed history and access to graphical and text-based editors such as VS Code, Zed, Helix, Vim, Nano etc. if its `reedline` editor falls short for a particular task.

### * With standard input:

```bash
echo "(1..=10).product::<u32>()" | rs_script --stdin                # Short form: -s
```

### * With a TUI (Terminal User Interface) editor

```bash
rs_script --edit                                                    # Short form: -d
```
![Editor](edit1w.png)

![Edit run](edit2w.png)

### * With standard input into the TUI editor:

```bash
cat my_file.rs | rs_script --edit                                   # Short form: -d
```

This allows you to edit or append to the stdin input before submitting it to `rs-script`.

#### A note on the TUI editor
In order for the Shift-Up and Shift-Down key combinations to work on Apple Terminal, you may need to add the following to your Apple Terminal Settings | Profiles | Keyboard settings:
Shift-Up: `\033;[2A` and `Shift-Down`: \033;[2B. Use the Esc key to generate \033. This is not necessary on Iterm2 or WezTerm.

### * As a filter on standard input (loop mode):

At a minimum, loops though `stdin` running the `--loop` expression against every line. The line number and content are made available to the expression as `i` and `line` respectively.

```bash
cat demo/hello.rs | rs_script --loop 'format!("{i}.\t{line}")'      # Short form: -l
```
![Loop](loopw.png)

Note the use of the `--quiet (-q)` option above to suppress messages from Cargo build.
For a true filter that you can pipe to another process, you can use `-qq` (or `--quiet --quiet`) to suppress all non-error output.

Alternatively:

```bash
rs_script -l 'format!("{i}.\t{line}")' < demo/hello.rs              # Long form: --loop
```
Loop mode also accepts the following optional arguments supplying surrounding code, along the lines of AWK:

```bash
--cargo (-C)    for specifying dependencies etc. in Cargo.toml format.
--begin (-B)    for specifying any imports, functions/closures, declarations etc. to be run before the loop.
--end   (-E)    for specifying any summary or final logic to run after the loop.
```

### * Getting started:

You have the choice of installing `rs-script` (recommended), or you may prefer to clone it and compile it yourself and run it via `cargo run`.

* Installing gives you speed out of the box and a simpler command-line interface without invoking Cargo yourself. You can download the demo library separately.
* Cloning gives you easy access to the demo scripts library and the opportunity to make local changes or a fork. You can also use the flag `--features=debug-logs` with the environment variable `RUST_LOG=rs_script=debug` to get debug logging.

## Overview

`rs-script` uses Cargo, `syn`, `quote` and `cargo_toml` to analyse and wrap well-formed snippets and expressions into working programs. Well-formed input programs are identified as such and passed unchanged to `cargo build`.

`rs-script` uses the `syn` crate to parse valid code into an abstract syntax tree (AST). This sidesteps any confusion due to source code embedded in comments or string literals, which are the bugbear of more superficial source code analysis techniques such as regular expressions and string parsing. `rs-script` then uses the `syn` visitor mechanism to traverse the AST to identify dependencies in the code so as to generate a `Cargo.toml`. These are then filtered to remove duplicates and false positives such as built-in Rust crates, renamed crates and local modules.

Well-formedness is determined by counting any occurrences of a `main` function in the AST. The lack of a `fn main` signifies a snippet or expression, whereas more than one `fn main` may be valid but must be actively flagged as such by the user with the `--multimain (-m)` option.

If your code does not successfully parse into an AST because of a coding error, `rs-script` will fall back to using source code analysis to prepare your code for the Rust compiler, which can then show you error messages to help you find the issues.

You may provide optional metadata in a toml block as described below. `rs-script` uses the `cargo_toml` crate to parse any metadata into a manifest struct, merges in any dependencies inferred from the AST, and then uses the `toml` crate to write out the dedicated Cargo.toml file that Cargo needs to build the script. Finally, in the case of snippets and expressions, it uses the `quote` crate to embed the logic in a well-formed program template, which it then invokes Cargo to build.

All of this is quite fast: the real bottleneck will be the familiar Cargo build process downloading and compiling your dependencies on the initial build. Cargo build output will be displayed in real time so that there are no mystery delays. If you rerun the compiled script it should be lightning fast.

In this way `rs-script` attempts to handle any valid (or invalid) Rust script, be it a program, snippet or expression. It will try to generate a dedicated Cargo.toml for your script from `use` statements in your code, although for speed and precision I recommend that you embed your own in a toml block:
```/*
[toml]
[dependencies]
...
*/
```
at the start of the script, as you will see done in most of the demos. To assist with this, after each successful Cargo search `rs-script `will generate and print a basic toml block with the crate name and version under a `[dependencies]` header, for you to copy and paste into your script if you want to. It does not print a combined block, so it's up to you to merge all the dependencies into a single toml block. All dependencies can typically go under the single `[dependencies]` header in the toml block, but thanks to `cargo_toml` there is no specific limit on what valid Cargo code you can place in the toml block.

`rs-script` aims to be as comprehensive as possible without sacrificing speed and simplicity. It uses timestamps to rerun compiled scripts without unnecessary rebuilding, although you can override this behaviour. For example, a precompiled script will calculate the 35,661-digit factorial of 10,000 in under half a second on my M1 MacBook Air.

## Installation

### Minimum Supported Rust Version
The minimum supported Rust version (MSRV) for `rs-script` is 1.74.1.

### TODO >>>
You can install `rs-script` using `cargo install`:

```bash
cargo install rs-script
```
### TODO >>> Installing the starter kit (demo directory)


## Usage
Once installed, you can use the `rs_script` command (with underscore) from the command line. `rs-script` uses the clap crate to process command-line arguments including --help.

### TODO >>>
Here are some examples:

### Evaluating an expression
#### Concise fast factorial calculation for numbers up to 34 (it overflows beyond that, but see demos for bigger numbers):
```bash
rs_script -e '(1..=34).product::<u128>()'
```

#### Shoehorn a script into an expression, should the need ever arise!:
```bash
rs_script -e "$(cat demo/fizz_buzz_gpt.rs)"
```

#### Run a script in quiet mode but show timings
```bash
rs_script -tq demo/fizz_buzz_gpt.rs
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
...
89
FizzBuzz
91
92
Fizz
94
Buzz
Fizz
97
98
Fizz
Buzz
Completed run in 0.11s
rs-script completed processing script fizz_buzz_gpt.rs in 0.20s
```

### Using the REPL
```bash
rs_script -l
```
This will start an interactive REPL session where you can enter or paste in a single- or multi-line Rust expression and press Enter to run it. You can also retrieve and optionally edit an expression from history.
Having evaluated the expression you may choose to edit it, and / or the generated Cargo.toml, in your preferred editor (VS Code, Helix, Zed, nano...) and rerun it. The REPL also offers basic housekeeping functions for the temporary files generated, otherwise being in temporary space they will be cleaned up by the operating system in due course.

#### Revisiting a REPL expression from a previous session
```bash
rs_script -l repl_<nnnnnn>.rs
```
will return to edit and run a named generated script from a previous REPL session.

More informally, you can access the last 25 previous REPL commands or expressions from within the REPL function just by using the up and down arrow keys to navigate history.

#### General notes on REPL
All REPL files are created under the `rs_repl` subdirectory of your temporary directory (e.g. $TMPDIR in *nixes, and referenced as std::env::temp_dir() in Rust) so as not to clog up your system. Until such time as they are harvested by the OS you can display the locations and copy the files if desired.

The REPL feature is not suited to scripts of over about 1K characters, due to the limitations of the underlying line editor. You can overcome these limitations by using the `edit` mode instead, but by this point it's probably more convenient just to use the --stdin / -s feature instead, or save the source in a .rs file and run it from the command line.

## Features

_Rust is primarily an expression language.... In contrast, statements serve mostly to contain and explicitly sequence expression evaluation._
_— The Rust Reference_

* Runs serious Rust scripts (not just the "Hello, world!" variety) with no need to create a project.
* Aims to be the most capable and reliable script runner for Rust code.
* Crucially, specific features of dependencies may be specified, giving your scripts access to advanced functionality. Local path and git dependencies may also be specified, allowing you to access your unpublished crates.
* A choice of modes - bearing in mind the importance of expressions in Rust:
    * expression mode for small, basic expressions on the fly.
    * REPL mode offers interactivity, and accepts multi-line expressions since it respects matching braces, brackets, parens and quotes.
    * stdin mode accepts larger scripts or programs on the fly, which need not be expressions as such. Being stdin it can be used with piped input.
    * edit mode is stdin mode with the addition of basic TUI (terminal user interface) in-place editing.
    * the classic script mode runs any .rs file consisting of a valid Rust script or program.
* You can use a shebang to write scripts in Rust.
* You can even build your own commands, using the `--executable` (`-x`) option. This will compile a valid script to an executable command in the Cargo bin directory `<home>/.cargo/bin`.
* `rs-script` supports a personal library of code samples for reuse. The downloadable starter set in the demo subdirectory includes numerous examples from popular crates, as well as original examples including fast big-integer factorial and Fibonacci calculation and prototypes of TUI editing and of the adaptive colour palettes described below.
* Automatic support for light or dark backgrounds and a 16- or 256- colour palette for different message types, according to terminal capability. On Windows, `rs-script` defaults to basic ANSI-16 colours and dark mode support for reasons beyond my control, but the dark mode colours it uses have been chosen to work well with most light modes.
* In some cases you may be able to develop a module of a project individually by giving it its own main method and embedded Cargo dependencies and running it from rs-script. Failing that, you can always work on a minimally modified copy in another location. This approach allows you to develop and debug this functionality without having it break your project. For example the demo versions of colors.rs and stdin.rs were both prototypes that were fully developed as scripts before being merged into the main `rs-script` project.

## Platform Support
This crate is designed to be cross-platform and supports:

* MacOS: Tested on MacOS (M1) Sonoma.
* Linux: Tested on Zorin and (WSL2) Ubuntu.
* Windows: Tested on Windows 11:
    - PowerShell 5 and CMD under Windows Terminal and Windows Console
    - WSL2

GitHub actions test each commit on `ubuntu-latest`, `macos-latest` and `windows-latest`.

## Related projects

(With acknowledgements to the author of `rust-script`)

* `evcxr` - Perhaps the most well-known Rust REPL.
* `cargo-script` - Rust script runner (unmaintained project).
* `rust-script` - maintained fork of cargo-script.
* `cargo-eval` - maintained fork of cargo-script.
* `cargo-play` - local Rust playground.
* `irust` - limited Rust REPL.
* `runner` - experimental tool for running Rust snippets without Cargo, exploring dynamic vs static linking for speed. I have an extensively modified fork of this crate on GitHub, but I highly recommend using the current `rs-script` crate rather than that fork.
* `cargo-script-mvs` - RFC demo.

## Contributing

Contributions will be given due consideration if they fit the goals of the project. Please see CONTRIBUTING.md for more details.

## Of possible interest: AI

I made extensive use of free versions of LLMs - mainly ChatGPT and to a lesser extent Gemini - for four aspects of this project:
* problem solving
* suggestions and guidance on best practices
* generation of unit and integration tests
* grunt work of generating "first-approximation" code and boilerplate to spec.

Although these LLMs could be hit-and-miss or clichéd when it comes to specifics and to received wisdom, my experience has been that intensive dialogues with the LLMs have generally either led them to produce worthwhile solutions, or at least led me to see that there were sometimes deeper-seated issues that AI couldn't solve and to dig deeper researching on my own.

I short I found using AI hugely beneficial in terms not only of productivity but of extending the scope of work that I could comfortably take on. I didn't use any licensed or integrated features and at this stage I'm not feeling the lack of same.

## License

SPDX-License-Identifier: Apache-2.0 OR MIT

Licensed under either of

    Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

or

    MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you will be dual-licensed as above, without any additional terms or conditions.
