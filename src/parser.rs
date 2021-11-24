//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use nom::bytes::complete::{take_while1, tag};
use nom::IResult;
use nom::sequence::tuple;
use nom::character::complete::{char, digit1, space1};

/// Parse a line that indicates the start of a test.
/// Returns the name of the test that just started
/// Expected format is:
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_test_start(){
        let ctest_output = " Start 1: start_of_a_test";

        assert_eq!(parse_test_start(ctest_output), Ok(("", "start_of_a_test".into())));
    }

    #[test]
    fn test_parse_test_start_next_name(){
        let ctest_output = " Start 30: a_different_test";

        assert_eq!(parse_test_start(ctest_output), Ok(("", "a_different_test".into())));
    }
}


