use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_script_mode_success() {
    let mut cmd = Command::cargo_bin("reversible_interpreter").unwrap();
    cmd.args(["script"])
        .write_stdin("add PUSH 5; PUSH 3\nrun\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Instructions added.").and(predicate::str::contains(
                "All instructions executed. Stack: [5, 3]",
            )),
        );
}

#[test]
fn test_cli_script_mode_error() {
    let mut cmd = Command::cargo_bin("reversible_interpreter").unwrap();
    cmd.args(["script"])
        .write_stdin("add PUSH 5; DIV\nrun\n")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Error: StackUnderflow"));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("reversible_interpreter").unwrap();
    cmd.args(["script"])
        .write_stdin("add INVALID\n")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Invalid instruction")
                .and(predicate::str::contains("No valid instructions provided")),
        );
}
