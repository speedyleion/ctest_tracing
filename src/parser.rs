//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

use crate::trace::Trace;

use nom::bytes::complete::{tag, take_till, take_while1};
use nom::character::complete::{char, digit1, space1};
use nom::character::is_digit;
use nom::sequence::tuple;
use nom::IResult;
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

pub fn parse<R: Read>(reader: BufReader<R>) -> Vec<Trace> {
    let mut running_tests: HashMap<String, (Duration, u32)> = HashMap::new();
    let mut traces = vec![];
    let mut trace_timer = Duration::new(0, 0);
    let mut max_thread_number = 0;
    let mut free_threads = VecDeque::new();
    for l in reader.lines() {
        let line = l.unwrap();
        if let Ok((_, test_case)) = parse_test_start(&line) {
            let thread_number = match free_threads.pop_front() {
                Some(number) => number,
                None => {
                    let number = max_thread_number;
                    max_thread_number += 1;
                    number
                }
            };
            running_tests.insert(test_case, (trace_timer, thread_number));
            continue;
        }
        if let Ok((_, (test_case, duration))) = parse_test_finish(&line) {
            // When a test is not run it will output the same as a finish message, but won't
            // have a start message, so won't exist in running_tests
            if let Some((start, thread)) = running_tests.remove(&test_case) {
                traces.push(Trace {
                    name: test_case,
                    start,
                    duration,
                    thread_number: thread,
                });
                trace_timer = start + duration;
                free_threads.push_back(thread);
            }
            continue;
        }
    }
    traces
}

//  Parse a line that indicates the start of a test.
//  Returns the name of the test that just started
//  Expected format is:
//
//      Start 30: name_of_test
//
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

//  Parse a line that indicates a test has finished
//  Returns the name of the test and the duration
//  Expected format is:
//
//      1/1 Test #1: test_stuff .......................***Failed    0.81 sec
//  or
//      1/1 Test #1: test_stuff .......................   Passed    0.74 sec
//  or
//      1/1 Test #1: test_stuff ......................***Not Run   0.00 sec
//
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
                Start  1: a_test
            1/1 Test #1: a_test ......................   Passed   0.20 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let name = "a_test".into();
        let start = Duration::new(0, 0);
        let duration = Duration::from_millis(200);
        assert_eq!(
            parse(reader),
            vec![Trace {
                name,
                start,
                duration,
                thread_number: 0
            }]
        );
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
        assert_eq!(
            parse(reader),
            vec![Trace {
                name,
                start,
                duration,
                thread_number: 0
            }]
        );
    }

    #[test]
    fn test_parse_skipped_test() {
        let ctest_output = r#"
            1/1 Test #1: a_failing_test ......................***Not Run   0.00 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        assert_eq!(parse(reader), vec![]);
    }

    #[test]
    fn test_parse_serial_tests() {
        let ctest_output = r#"
                Start  1: test_one
            1/2 Test #1: test_one ......................   Passed   0.20 sec
                Start  2: test_two
            2/2 Test #2: test_two ......................   Passed   0.20 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let start = Duration::new(0, 0);
        let duration = Duration::from_millis(200);
        let second_start = duration;
        let test_1 = Trace {
            name: "test_one".into(),
            start,
            duration,
            thread_number: 0,
        };
        let test_2 = Trace {
            name: "test_two".into(),
            start: second_start,
            duration,
            thread_number: 0,
        };
        assert_eq!(parse(reader), vec![test_1, test_2]);
    }

    #[test]
    fn test_parse_parallel_tests() {
        let ctest_output = r#"
                Start  1: test_one
                Start  2: test_two
            1/2 Test #1: test_one ......................   Passed   0.20 sec
            2/2 Test #2: test_two ......................   Passed   0.30 sec"#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let start = Duration::new(0, 0);
        let test_1 = Trace {
            name: "test_one".into(),
            start,
            duration: Duration::from_millis(200),
            thread_number: 0,
        };
        let test_2 = Trace {
            name: "test_two".into(),
            start,
            duration: Duration::from_millis(300),
            thread_number: 1,
        };
        assert_eq!(parse(reader), vec![test_1, test_2]);
    }

    #[test]
    fn test_parse_parallel_with_thread_reuse() {
        let ctest_output = r#"
                Start  1: test_one
                Start  2: test_two
            1/3 Test #1: test_one ......................   Passed   0.20 sec
                Start  3: test_three
            2/3 Test #2: test_two ......................   Passed   0.30 sec
            3/3 Test #3: test_three ......................   Passed   0.50 sec
            "#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let start = Duration::new(0, 0);
        let test_1 = Trace {
            name: "test_one".into(),
            start,
            duration: Duration::from_millis(200),
            thread_number: 0,
        };
        let test_2 = Trace {
            name: "test_two".into(),
            start,
            duration: Duration::from_millis(300),
            thread_number: 1,
        };
        let test_3 = Trace {
            name: "test_three".into(),
            start: Duration::from_millis(200),
            duration: Duration::from_millis(500),
            thread_number: 0,
        };
        assert_eq!(parse(reader), vec![test_1, test_2, test_3]);
    }

    #[test]
    fn test_parse_common_blocking_test() {
        // This shows `test_one` blocking both `test_three` and `test_four`.  It could be that
        // it's a common set up fixture for them.  So thread 0 will get used for `test_one` and
        // `test_three`.  Since `test_four` is started it should grab the next available thread,
        // which would need to be `2`, as `test_two` is still running on thread `1`
        let ctest_output = r#"
                Start  1: test_one
                Start  2: test_two
            1/4 Test #1: test_one ......................   Passed   0.20 sec
                Start  3: test_three
                Start  4: test_four
            3/4 Test #3: test_three ......................   Passed   0.50 sec
            4/4 Test #3: test_four ......................   Passed   0.50 sec
            2/4 Test #2: test_two ......................   Passed   10.0 sec
            "#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let start = Duration::new(0, 0);
        let test_1 = Trace {
            name: "test_one".into(),
            start,
            duration: Duration::from_millis(200),
            thread_number: 0,
        };
        let test_2 = Trace {
            name: "test_two".into(),
            start,
            duration: Duration::new(10, 0),
            thread_number: 1,
        };
        let test_3 = Trace {
            name: "test_three".into(),
            start: Duration::from_millis(200),
            duration: Duration::from_millis(500),
            thread_number: 0,
        };
        let test_4 = Trace {
            name: "test_four".into(),
            start: Duration::from_millis(200),
            duration: Duration::from_millis(500),
            thread_number: 2,
        };
        assert_eq!(parse(reader), vec![test_1, test_3, test_4, test_2]);
    }

    #[test]
    fn test_parse_two_finish_two_start() {
        let ctest_output = r#"
                Start  1: test_one
                Start  2: test_two
            1/4 Test #1: test_one ......................   Passed   0.20 sec
            2/4 Test #2: test_two ......................   Passed   10.0 sec
                Start  3: test_three
                Start  4: test_four
            3/4 Test #3: test_three ......................   Passed   0.50 sec
            4/4 Test #3: test_four ......................   Passed   0.50 sec
            "#;

        let reader = BufReader::new(ctest_output.as_bytes());
        let start = Duration::new(0, 0);
        let test_1 = Trace {
            name: "test_one".into(),
            start,
            duration: Duration::from_millis(200),
            thread_number: 0,
        };
        let test_2 = Trace {
            name: "test_two".into(),
            start,
            duration: Duration::new(10, 0),
            thread_number: 1,
        };
        let test_3 = Trace {
            name: "test_three".into(),
            start: Duration::new(10, 0),
            duration: Duration::from_millis(500),
            thread_number: 0,
        };
        let test_4 = Trace {
            name: "test_four".into(),
            start: Duration::new(10, 0),
            duration: Duration::from_millis(500),
            thread_number: 1,
        };
        assert_eq!(parse(reader), vec![test_1, test_2, test_3, test_4]);
    }
}
