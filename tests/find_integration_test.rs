use std::{path::PathBuf, env};
use assert_cmd::Command;

#[test]
// The execute function is the public API of the find mode and it's what is called when the user writes find in the CLI
fn find_execute_test(){
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap(); //If unwrap fails, we won't be able to find the binary and the test fails.
    let mut binary = PathBuf::from(manifest_dir); // We build the path to the binary in the debug mode!
    binary.push("target");
    binary.push("debug");
    binary.push("amazing_clu");

    // Path to the test folder
    let mut from = PathBuf::new();
    from.push(".");
    from.push("tests");
    from.push("find_files");

    // This variable will hold the output
    let mut stdout;

    // Find everything in the directory tree except the hidden files and those contained in hidden folders.
    let mut cmd = Command::new(binary.clone());
    cmd.arg("find").arg(from.to_str().unwrap());
    cmd.assert().success(); // Ensure the execution succeeded.
    unsafe{ stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout);} // The output is for sure a valid String
    // Found the desired files
    assert!(stdout.contains("found_file1.txt"));
    assert!(stdout.contains("found_html.html"));
    assert!(stdout.contains("subfolder1/found_file2.txt"));
    assert!(stdout.contains("subfolder1/subfolder2/found_file3.txt"));
    // But not the hidden files :)
    assert!(!stdout.contains(".hidden_file1.txt"));
    assert!(!stdout.contains("subfolder1/.hidden_subfolder/hidden_file2.txt"));

    // Also find the hidden files
    cmd.arg("-H");
    cmd.assert().success(); // Ensure the execution succeeded.
    unsafe {stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout);}
    // Found the desired files
    assert!(stdout.contains("found_file1.txt"));
    assert!(stdout.contains("found_html.html"));
    assert!(stdout.contains("subfolder1/found_file2.txt"));
    assert!(stdout.contains("subfolder1/subfolder2/found_file3.txt"));
    assert!(stdout.contains(".hidden_file1.txt"));
    assert!(stdout.contains("subfolder1/.hidden_subfolder/hidden_file2.txt"));

    // Find only the HTML files
    let mut cmd = Command::new(binary.clone());
    cmd.arg("find").arg(from.to_str().unwrap()).arg("-e").arg("*.html");
    cmd.assert().success(); // Ensure the execution succeeded.
    unsafe {stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout);}
    // Found the HTML
    assert!(stdout.contains("found_html.html"));
    // But not the TXT
    assert!(!stdout.contains("found_file1.txt"));
    assert!(!stdout.contains("subfolder1/found_file2.txt"));
    assert!(!stdout.contains("subfolder1/subfolder2/found_file3.txt"));
    assert!(!stdout.contains(".hidden_file1.txt"));
    assert!(!stdout.contains("subfolder1/.hidden_subfolder/hidden_file2.txt"));


    // Empty research
    let mut cmd = Command::new(binary.clone());
    cmd.arg("find").arg(from.to_str().unwrap()).arg("-e").arg("*.js"); 
    cmd.assert().success();
    cmd.assert().stdout("The request didn't produce any output.\n");

    // Research in several directories 
    let mut from2 = PathBuf::new();
    from2.push(".");
    from2.push("tests");
    from2.push("base_files");
    let mut cmd = Command::new(binary.clone());
    cmd.arg("find").arg(from.to_str().unwrap()).arg(from2.to_str().unwrap());
    cmd.assert().success();
    unsafe {stdout = String::from_utf8_unchecked(cmd.assert().get_output().to_owned().stdout);}
    assert!(stdout.contains("found_file1.txt")); // This is inside tests/find_files
    assert!(stdout.contains("sample_text.txt")); // This is inside tests/base_files

}