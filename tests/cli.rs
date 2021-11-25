//          Copyright Nick G 2021.
// Distributed under the Boost Software License, Version 1.0.
//    (See accompanying file LICENSE or copy at
//          https://www.boost.org/LICENSE_1_0.txt)
//
// Used for testing the main cli interface of the application

use assert_cmd::Command;
use std::fs;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn reading_from_tempfile() -> Result<(), Box<dyn std::error::Error>> {
    let ctest_output = r#"
                Start  1: test_one
            1/1 Test #1: test_one ......................   Passed   0.20 sec
            "#;
    let mut file = NamedTempFile::new()?;
    file.write_all(ctest_output.as_bytes())?;
    let mut cmd = Command::cargo_bin("ctest_tracing")?;

    cmd.arg(file.path());
    let expected = "[{\"name\":\"test_one\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":0,\"dur\":200000,\"pid\":0,\"tid\":0}]";
    cmd.assert().stdout(expected);
    Ok(())
}

#[test]
fn reading_from_tempfile_multiple_tests() -> Result<(), Box<dyn std::error::Error>> {
    let ctest_output = r#"
                Start  1: test_one
            1/2 Test #1: test_one ......................   Passed   0.20 sec
                Start  2: test_two
            2/2 Test #2: test_two ......................   Passed   0.30 sec
            "#;
    let mut file = NamedTempFile::new()?;
    file.write_all(ctest_output.as_bytes())?;
    let mut cmd = Command::cargo_bin("ctest_tracing")?;

    cmd.arg(file.path());
    let expected = "[{\"name\":\"test_one\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":0,\"dur\":200000,\"pid\":0,\"tid\":0},{\"name\":\"test_two\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":200000,\"dur\":300000,\"pid\":0,\"tid\":0}]";
    cmd.assert().stdout(expected);
    Ok(())
}

#[test]
fn reading_from_stdin() -> Result<(), Box<dyn std::error::Error>> {
    let ctest_output = r#"
                Start  1: test_one
            1/1 Test #1: test_one ......................   Passed   0.20 sec
            "#;
    let mut cmd = Command::cargo_bin("ctest_tracing")?;

    cmd.write_stdin(ctest_output);
    let expected = "[{\"name\":\"test_one\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":0,\"dur\":200000,\"pid\":0,\"tid\":0}]";
    cmd.assert().stdout(expected);
    Ok(())
}

#[test]
fn writing_to_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let ctest_output = r#"
                Start  1: test_one
            1/1 Test #1: test_one ......................   Passed   0.20 sec
            "#;
    let mut cmd = Command::cargo_bin("ctest_tracing")?;

    let dir = tempdir()?;
    let file_path = dir.path().join("temp_output_file.json");
    cmd.arg("-")
        .arg("-o")
        .arg(file_path.as_os_str())
        .write_stdin(ctest_output);

    let expected = "[{\"name\":\"test_one\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":0,\"dur\":200000,\"pid\":0,\"tid\":0}]";
    cmd.assert().stdout("");

    let contents = fs::read_to_string(file_path)?;
    assert_eq!(expected, contents);
    Ok(())
}

#[test]
fn writing_to_nested_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let ctest_output = r#"
                Start  1: test_one
            1/1 Test #1: test_one ......................   Passed   0.20 sec
            "#;
    let mut cmd = Command::cargo_bin("ctest_tracing")?;

    let dir = tempdir()?;
    let file_path = dir.path().join("nested/file/name.json");
    cmd.arg("-")
        .arg("-o")
        .arg(file_path.as_os_str())
        .write_stdin(ctest_output);

    let expected = "[{\"name\":\"test_one\",\"cat\":\"test\",\"ph\":\"X\",\"ts\":0,\"dur\":200000,\"pid\":0,\"tid\":0}]";
    cmd.assert().stdout("");

    let contents = fs::read_to_string(file_path)?;
    assert_eq!(expected, contents);
    Ok(())
}
