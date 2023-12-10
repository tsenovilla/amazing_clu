use std::{path::PathBuf, env};
use assert_cmd::Command;

#[test] 
// The execute function is the public API of the grep mode and it's what is called when the user writes grep in the CLI
// This function calls the other Grep's functions and propagates their errors. Let's test:
// 1. A successful case if dereference recursive is not set on.
// 2. A successful case if dereference recursive is set on. (1 and 2 also serves as tests for globbing cause it's executed here)
// 3. Invalid command combination error
// 4. Empty research error.
// The other errors are inherited from the Grep's functions and are tested in their corresponding tests.
fn grep_execute_test(){
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap(); //If unwrap fails, we won't be able to find the binary and the test fails.
    let mut binary = PathBuf::from(manifest_dir); // We build the path to the binary in the debug mode!
    binary.push("target");
    binary.push("debug");
    binary.push("amazing_clu");

    // Path to the test folder
    let mut path = PathBuf::new();
    path.push(".");
    path.push("tests");
    path.push("grep_files");

    // The standard output'll be here
    let mut stdout;

    // Non dereference recursive test
    path.push("*.txt"); // Seatch for .txt files
    let mut cmd = Command::new(binary.clone());
    cmd.arg("grep").arg("grep").arg(path.to_str().unwrap()).arg("-n");
    cmd.assert().success(); // Ensure the command were well run
    unsafe {stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout)} //The output is for sure a valid String
    // The output contains the name of the file and the matched pattern. It does not contain the hidden text
    assert!(stdout.contains("sample_text.txt"));
    assert!(stdout.contains("3:I'm grep"));
    assert!(!stdout.contains(".hidden_text.txt"));

    // Dereference recursive test
    path.pop(); // Search in the whole grep_files directory
    let mut cmd = Command::new(binary.clone());
    cmd.arg("grep").arg("grep").arg(path.to_str().unwrap()).arg("-R").arg("-H").arg("-C").arg("1").arg("-n");
    cmd.assert().success();
    // The output contains the files identifiers where the search has been successful, we cannot ensure the order in which these results are obtained due to the concurrency of our grep, then we can just ensure that we've found what we're looking for.
    unsafe { stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout)}
    // sample_text.txt found!
    assert!(stdout.contains("sample_text.txt"));
    assert!(stdout.contains("2-How are you? Who are you?\n3:I'm grep\n4-Nice to meet you"));
    // .hidden_text.txt found!
    assert!(stdout.contains(".hidden_text.txt"));
    assert!(stdout.contains("1:I'm a hidden file created to test grep!"));
    // sample_text2.txt found
    assert!(stdout.contains("sample_text2.txt"));
    assert!(stdout.contains("1-I'm contained into a hidden folder.\n2:I'd like to test grep"));

    // Invalid command combination error
    let mut cmd = Command::new(binary.clone());
    cmd.arg("grep").arg("grep").arg(path.to_str().unwrap()).arg("-R").arg("-H").arg("-C").arg("1").arg("-n").arg("-c");
    cmd.assert().success(); // Note that even in error, we're handling them to show a nice output message, then the command is successfully executed
    cmd.assert().stdout("Introduced an invalid combination of commands in grep mode. For more information try --help.\n");

    // Empty research error
    let mut cmd = Command::new(binary.clone());
    cmd.arg("grep").arg("Grep").arg(path.to_str().unwrap()).arg("-R").arg("-H").arg("-C").arg("1").arg("-n");
    cmd.assert().success();
    cmd.assert().stdout("The request didn't produce any output.\n");

}