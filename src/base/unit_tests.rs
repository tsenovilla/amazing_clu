use crate::{base, clu_errors::CluErrors};
use std::path::PathBuf;

#[test]
fn get_bytes_test(){
    let number: u8 = 3;
    assert_eq!(&[3], base::get_bytes(&number));
    let bools: [bool;3] = [true, false, false];
    assert_eq!(&[1,0,0], base::get_bytes(&bools));
}

#[test]
fn parse_path_test(){
    // Well parsed test
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("base_files");
    pathbuf.push("*");
    // Without hidden items
    let result_without_hidden = base::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).unwrap();
    assert_eq!(1, result_without_hidden.len()); // It contains sample_text.txt

    // With hidden items
    let result_with_hidden = base::parse_path(&pathbuf.to_str().unwrap().to_string(), false, true).unwrap();
    assert_eq!(3, result_with_hidden.len()); // It contains the same as the previous + .hidden_folder + .hidden_text.txt

    // Invalid path due to invalid parent dir test
    assert_eq!(CluErrors::InputError("The introduced path: '' isn't valid.".to_string()), base::parse_path(&"".to_string(),false,false).unwrap_err());

    // Invalid regex in the search path
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("[a-s+");
    assert_eq!(CluErrors::RegexError("[a-s+".to_string()), base::parse_path(&pathbuf.to_str().unwrap().to_string(),false,false).unwrap_err());

    // Invalid file name in the search path
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("..");
    assert_eq!(CluErrors::InputError(format!("The introduced path: '{}' isn't valid.", pathbuf.to_str().unwrap())), base::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).unwrap_err());

    // Unable to read directory test. This happens if the user doesn't have permission to read the directory, or if it's introduced a regex that passes the filter but whose parent dir doesn't exist. We will use this second case to carry out the test
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("sc");
    pathbuf.push("*");
    assert_eq!(CluErrors::UnableToReadDirectory, base::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).unwrap_err());

}

#[test]
fn parse_path_recursively_test(){
    // Base case test, the path only contains files
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("base_files");
    pathbuf.push("sample_text.txt");
    let query = base::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], false).unwrap();
    assert_eq!(1, query.len()); 

    // Recursive case test, finding all the files in the directory tree
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("base_files");
    let without_hidden_files = base::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], false).unwrap();
    let with_hidden_files = base::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], true).unwrap();
    assert_eq!(1, without_hidden_files.len()); 
    assert_eq!(3, with_hidden_files.len()); 

    // The errors that may occur here are:
    // - Errors propagated from parse_path (already tested).
    // - Concurrency errors. There's no way to create a unit test of that as the concurrency is defined in the function so we cannot close the channel or panick a thread from here
}