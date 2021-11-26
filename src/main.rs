//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use clio::{Input, Output};
use ctest_tracing::parser::parse;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufReader, Write};
use std::path::Path;
use structopt::StructOpt;

/// Converts ctest result output into Chrome's trace event JSON format.
///
/// The output is minified JSON, which one should be able to visualize
/// by navigating to `chrome://tracing` in a Chrome browser.
#[derive(StructOpt)]
#[structopt(name = "ctest_tracing", verbatim_doc_comment)]
struct Opt {
    /// Input file, use '-' for stdin
    #[structopt(parse(try_from_os_str = Input::try_from_os_str), default_value="-")]
    input: Input,

    /// Output file '-' for stdout
    /// When a file is specified any parent directories will be
    /// created if they don't exist
    #[structopt(long, short, parse(try_from_os_str = try_from_os_str), default_value="-", verbatim_doc_comment)]
    output: Output,
}

fn make_parent_dir(path: &OsStr) {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
}

fn try_from_os_str(path: &OsStr) -> std::result::Result<Output, std::ffi::OsString> {
    if path != "-" {
        make_parent_dir(path);
    }
    Output::try_from_os_str(path)
}

fn main() -> std::io::Result<()> {
    let mut opt = Opt::from_args();

    let reader = BufReader::new(opt.input.lock());
    let traces = parse(reader);

    let serialized_data = serde_json::to_string(&traces).unwrap();
    opt.output.write_all(serialized_data.as_bytes()).unwrap();

    Ok(())
}
