//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use ctest_tracing::parser::parse;
use clap::{App, Arg};
use std::fs::File;
use std::io::BufReader;

fn main() -> std::io::Result<()> {
    let matches = App::new("Ctest Tracing")
        .about("Generates JSON output for use in chrome testing")
        .arg(Arg::with_name("INPUT")
            .help("The input ctest data file to use")
            .required(true)
            .index(1))
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();

    let f = File::open(input_file)?;
    let reader = BufReader::new(f);
    let traces = parse(reader);

    let serialized_data = serde_json::to_string(&traces).unwrap();
    println!("{}", serialized_data);

    Ok(())
}
