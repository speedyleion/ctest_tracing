//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)

fn main() {
    println!("Hello, world!");
}

fn parse_test_start(_line: &[u8]) -> String {
    "start_of_a_test".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_test_start(){
        let ctest_output = b" Start 1: start_of_a_test";

        assert_eq!(parse_test_start(ctest_output), "start_of_a_test");
    }
}

