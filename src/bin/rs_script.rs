#![allow(clippy::uninlined_format_args)]

use rs_script::{execute, get_args};
use std::error::Error;

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = get_args();
    execute(args)?;

    Ok(())
}
