//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use nom::bytes::complete::{tag, take_till, take_while1};
use nom::character::complete::{char, digit1, space1};
use nom::character::is_digit;
use nom::sequence::tuple;
use nom::IResult;
use std::time::Duration;
use std::io::{BufReader, BufRead, Read};
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct Trace {
    name: String,
    start: Duration,
    duration: Duration,
}

pub fn parse<R: Read>(reader: BufReader<R>) -> Vec<Trace>{
    let mut running_tests: HashMap<String, Duration> = HashMap::new();
    let mut traces = vec![];
    for l in reader.lines(){
        let line = l.unwrap();
        if let Ok((_, test_case)) = parse_test_start(&line) {
            running_tests.insert(test_case, Duration::new(0, 0));
            continue;
        }
        if let Ok((_, (test_case, duration))) = parse_test_finish(&line) {
            match running_tests.remove(&test_case) {
                Some(start) => traces.push(Trace{name: test_case, start, duration}),
                // Happens for tests that aren't run
                None => ()
            }
            continue;
        }
    }
    traces
}

/// Parse a line that indicates the start of a test.
/// Returns the name of the test that just started
/// Expected format is:
///
///     Start 30: name_of_test\n
///
fn parse_test_start(i: &str) -> IResult<&str, String> {
    let space = space1;
    let test_name = take_while1(|c| c != ' ');
    let test_number = digit1;
    let start = tag("Start");
    let colon = char(':');

    let (input, (_, _, _, _, _, _, test_name)) =
        tuple((space, start, space, test_number, colon, space, test_name))(i)?;

    Ok((input, test_name.into()))
}

/// Parse a line that indicates a test has finished
/// Returns the name of the test and the duration
/// Expected format is:
///
///     1/1 Test #1: test_stuff .......................***Failed    0.81 sec
/// or
///     1/1 Test #1: test_stuff .......................   Passed    0.74 sec
/// or
///     1/1 Test #1: test_stuff ......................***Not Run   0.00 sec
///
fn parse_test_finish(i: &str) -> IResult<&str, (String, Duration)> {
    let test_name = take_while1(|c| c != ' ');
    let colon = char(':');
    let test_result = take_till(|c| is_digit(c as u8));
    let test_number = take_till(|c| c == ':');

    let (input, (_, _, _, test_name, _, seconds_str, _, centis_str, _, _)) = tuple((
        test_number,
        colon,
        space1,
        test_name,
        test_result,
        digit1,
        char('.'),
        digit1,
        space1,
        tag("sec"),
    ))(i)?;

    // One could use `nom::number::complete::double` to parse the seconds, however this will lose
    // some precision, i.e. 3.32 seconds will turn into 3.319 in the duration
    let seconds = seconds_str.parse().unwrap();
    let centis: u64 = centis_str.parse().unwrap();
    let millis = centis * 10;
    let duration = Duration::new(seconds, 0) + Duration::from_millis(millis);
    Ok((input, (test_name.into(), duration)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_test_start() {
        let ctest_output = "    Start 1: start_of_a_test";

        assert_eq!(
            parse_test_start(ctest_output),
            Ok(("", "start_of_a_test".into()))
        );
    }

    #[test]
    fn test_parse_test_start_next_name() {
        let ctest_output = " Start 30: a_different_test";

        assert_eq!(
            parse_test_start(ctest_output),
            Ok(("", "a_different_test".into()))
        );
    }

    #[test]
    fn test_parse_failed_test_finish() {
        let ctest_output = "1/1 Test #1: test_stuff .......................***Failed    0.81 sec";

        let duration = Duration::from_millis(810);
        assert_eq!(
            parse_test_finish(ctest_output),
            Ok(("", ("test_stuff".into(), duration)))
        );
    }

    #[test]
    fn test_parse_passed_test_finish() {
        let ctest_output = "10/120 Test #20: test_me .......................   passed    3.32 sec";

        let duration = Duration::from_millis(320) + Duration::new(3, 0);
        assert_eq!(
            parse_test_finish(ctest_output),
            Ok(("", ("test_me".into(), duration)))
        );
    }

    #[test]
    fn test_parse_skipped_test_finish() {
        let ctest_output = "1/1 Test #1: test_stuff ......................***Not Run   0.00 sec";

        let duration = Duration::new(0, 0);
        assert_eq!(
            parse_test_finish(ctest_output),
            Ok(("", ("test_stuff".into(), duration)))
        );
    }

    #[test]
    fn test_parse_single_test_result() {
        let ctest_output = r#"
                Start  1: a_test";
            1/1 Test #1: a_test ......................   Passed   0.20 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let name = "a_test".into();
        let start = Duration::new(0, 0);
        let duration = Duration::from_millis(200);
        assert_eq!(parse(reader), vec![Trace{name, start, duration}]);
    }

    #[test]
    fn test_parse_single_failed_test_result() {
        let ctest_output = r#"
                Start  1: a_failing_test
            1/1 Test #1: a_failing_test ......................***Failed   10.00 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let name = "a_failing_test".into();
        let start = Duration::new(0, 0);
        let duration = Duration::new(10, 0);
        assert_eq!(parse(reader), vec![Trace{name, start, duration}]);
    }

    #[test]
    fn test_parse_skipped_test() {
        let ctest_output = r#"
            1/1 Test #1: a_failing_test ......................***Not Run   0.00 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        assert_eq!(parse(reader), vec![]);
    }
}
