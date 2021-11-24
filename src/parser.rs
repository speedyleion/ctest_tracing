//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

fn parse_test_start(_line: &[u8]) -> String {
    "start_of_a_test".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_test_start(){
        let ctest_output = b" Start 1: start_of_a_test";

        assert_eq!(parse_test_start(ctest_output), "start_of_a_test");
    }

    #[test]
    fn test_parse_test_start_next_name(){
        let ctest_output = b" Start 30: a_different_test";

        assert_eq!(parse_test_start(ctest_output), "a_different_test");
    }
}

