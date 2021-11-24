//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use nom::bytes::complete::{take_while1, tag};
use nom::IResult;
use nom::sequence::tuple;
use nom::character::complete::{char, digit1, space1};
use std::time::Duration;

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

    let (input, (_, _, _, _, _, _, test_name)) = tuple((space, start, space, test_number, colon, space, test_name))(i)?;

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
    Ok(("", ("test_stuff".into(),Duration::from_millis(81))))

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_test_start(){
        let ctest_output = "    Start 1: start_of_a_test";

        assert_eq!(parse_test_start(ctest_output), Ok(("", "start_of_a_test".into())));
    }

    #[test]
    fn test_parse_test_start_next_name(){
        let ctest_output = " Start 30: a_different_test";

        assert_eq!(parse_test_start(ctest_output), Ok(("", "a_different_test".into())));
    }

    #[test]
    fn test_parse_failed_test_finish(){
        let ctest_output = "1/1 Test #1: test_stuff .......................***Failed    0.81 sec";

        let duration = Duration::from_millis(81);
        assert_eq!(parse_test_finish(ctest_output), Ok(("", ("test_stuff".into(), duration))));
    }

    #[test]
    fn test_parse_passed_test_finish(){
        let ctest_output = "10/120 Test #20: test_me .......................   passed    3.32 sec";

        let duration = Duration::from_millis(320) + Duration::new(3, 0);
        assert_eq!(parse_test_finish(ctest_output), Ok(("", ("test_me".into(), duration))));
    }
}


