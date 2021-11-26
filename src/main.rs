//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use clio::{Input, Output};
use ctest_tracing::parser::parse;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{BufReader, Write};
use std::path::Path;
use structopt::StructOpt;

/// Converts ctest result output into Chrome's trace event JSON format.
///
/// The output is minified JSON, which one should be able to visualize
/// by navigating to `chrome://tracing` in a Chrome browser.
///
/// This should walk silently over non ctest output.  The intent is to
/// allow one to pipe wrappers around ctest, for example a CMake target
/// which invokes ctest.  The CMake target may have build output as
/// well.
#[derive(StructOpt)]
#[structopt(name = "ctest_tracing", verbatim_doc_comment)]
struct Opt {
    /// Input file, use '-' for stdin
    #[structopt(parse(try_from_os_str = Input::try_from_os_str), default_value="-")]
    input: Input,

    /// Output file '-' for stdout
    /// When a file is specified any parent directories will be
    /// created if they don't exist
    #[structopt(long, short, parse(try_from_os_str = try_from_os_str_with_parents), default_value="-", verbatim_doc_comment)]
    output: Output,
}

// Creates all parent directories for `path`.  If `path` has no parent
// directories this is a no-op.
fn make_parent_dir(path: &OsStr) -> std::io::Result<()> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

// A wrapper around `Output::try_form_os_str()` which will create parent
// directories as needed for the provided `path`.
fn try_from_os_str_with_parents(path: &OsStr) -> std::result::Result<Output, std::ffi::OsString> {
    if path != "-" {
        make_parent_dir(path).map_err(|e| {
            let mut message = OsString::new();
            message.push("Error creating parent directory for \"");
            message.push(path);
            message.push("\": ");
            message.push(e.to_string());
            message
        })?;
    }
    Output::try_from_os_str(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opt = Opt::from_args();

    let reader = BufReader::new(opt.input.lock());
    let traces = parse(reader)?;

    let serialized_data = serde_json::to_string(&traces)?;
    opt.output.write_all(serialized_data.as_bytes())?;

    Ok(())
}
