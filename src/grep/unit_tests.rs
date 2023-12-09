use std::{path::PathBuf, collections::HashMap};
use regex::Regex;
use crate::{grep::{Grep, context::Context, options::Options, counters::Counters},clu_errors::CluErrors};

#[test] // The execute function calls the other Grep's functions and propagates their errors. Let's test:
// 1. A successful case if dereference recursive is not set on.
// 2. A successful case if dereference recursive is set on. (1 and 2 also serves as tests for globbing cause it's executed here)
// 3. Invalid command combination error
// 4. Empty research error.
// The other errors are inherited from the Grep's functions and are tested in their corresponding tests.
fn execute_test(){
    // Non dereference recursive
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    pathbuf.push("*.txt");
    let grep = Grep{
        pattern: Box::new("grep".to_string()),
        path: vec![pathbuf.to_str().unwrap().to_string()],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: true, invert_match: false, only_matching: false},
        counters: Counters { count: false, total_count: false }
    };
    let executed = grep.execute().unwrap(); 
    assert!(executed.contains("sample_text.txt")); 
    assert!(executed.contains("3:I'm grep")); 

    // Dereference recursive
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    let grep = Grep{
        pattern: Box::new("grep".to_string()),
        path: vec![pathbuf.to_str().unwrap().to_string()],
        dereference_recursive: true,
        ignore_case: false,
        hidden_items: true,
        context: Context { after_context: 0, before_context: 0, context: 1 },
        options: Options { files_with_matches: false, line_number: true, invert_match: false, only_matching: false},
        counters: Counters { count: false, total_count: false }
    };
    let executed = grep.execute().unwrap(); // Executed contains the files identifiers where the search has been successful, we cannot ensure the order in which these results are obtained due to the concurrency of our grep, then we can just ensure that we've found what we're looking for.
    // sample_text.txt found!
    assert!(executed.contains("sample_text.txt")); 
    assert!(executed.contains("2-How are you? Who are you?\n3:I'm grep\n4-Nice to meet you"));
    // .hidden_text.txt found!
    assert!(executed.contains(".hidden_text.txt")); 
    assert!(executed.contains("1:I'm a hidden file created to test grep!"));
    // sample_text2.txt found
    assert!(executed.contains("sample_text2.txt")); 
    assert!(executed.contains("1-I'm contained into a hidden folder.\n2:I'd like to test grep"));

    // Invalid command combination error
    let grep = Grep{
        pattern: Box::new("grep".to_string()),
        path: vec![pathbuf.to_str().unwrap().to_string()],
        dereference_recursive: true,
        ignore_case: false,
        hidden_items: true,
        context: Context { after_context: 0, before_context: 0, context: 1 },
        options: Options { files_with_matches: false, line_number: true, invert_match: false, only_matching: false},
        counters: Counters { count: true, total_count: false }
    };
    assert_eq!(CluErrors::InvalidCommandCombination("grep".to_string()), grep.execute().expect_err(""));

    // Empty research error
    let grep = Grep{
        pattern: Box::new("Grep".to_string()),
        path: vec![pathbuf.to_str().unwrap().to_string()],
        dereference_recursive: true,
        ignore_case: false,
        hidden_items: true,
        context: Context { after_context: 0, before_context: 0, context: 1 },
        options: Options { files_with_matches: false, line_number: true, invert_match: false, only_matching: false},
        counters: Counters { count: false, total_count: false }
    };
    assert_eq!(CluErrors::NotFoundError, grep.execute().expect_err(""));

}

#[test] // The errors are propagated from the single_file function, except the one creating the Regex. Let's test that one, a successful case without case insensitive flag set on and a successful case with the case insensitive flag set on.
fn execute_multiple_files_grep_test(){
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    let mut pathbuf2 = pathbuf.clone();
    pathbuf.push("sample_text.txt");
    pathbuf2.push(".hidden_text.txt");

    // Regex error
    let grep = Grep{
        pattern: Box::new("[a-z+".to_string()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: false, invert_match: false, only_matching: false},
        counters: Counters { count: true, total_count: false }
    };

    assert_eq!(CluErrors::RegexError(grep.pattern.to_string()), grep.execute_multiple_files_grep(vec![pathbuf.to_str().unwrap().to_string(), pathbuf2.to_str().unwrap().to_string()]).expect_err(""));

    // No case insensitive succeed
    let grep = Grep{
        pattern: Box::new("Grep".to_string()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: false, invert_match: false, only_matching: false},
        counters: Counters { count: true, total_count: false }
    };
    let executed = grep.execute_multiple_files_grep(vec![pathbuf.to_str().unwrap().to_string(), pathbuf2.to_str().unwrap().to_string()]).unwrap(); // As it comes from threads we cannot ensure the disposition, however we can ensure that it contains 0 twice due to the search is Case sensitive and the files don't contain Grep. We also can ensure the length is 4.
    assert_eq!(4, executed.len());
    assert_eq!(2, executed.iter().filter(|item| **item == "0".to_string()).count());

    // Same test but case insensitive will contain 1 twice
    let grep = Grep{
        pattern: Box::new("Grep".to_string()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: true,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: false, invert_match: false, only_matching: false},
        counters: Counters { count: true, total_count: false }
    };
    let executed = grep.execute_multiple_files_grep(vec![pathbuf.to_str().unwrap().to_string(), pathbuf2.to_str().unwrap().to_string()]).unwrap(); // As it comes from threads we cannot ensure the disposition, however we can ensure that it contains 0 twice due to the search is Case sensitive and the files don't contain Grep. We also can ensure the length is 4.
    assert_eq!(4, executed.len());
    assert_eq!(2, executed.iter().filter(|item| **item == "1".to_string()).count());

}

#[test] // Testing errors in this function and just one case of successful execution, as search, files_with_matches, count and total_count functions called during a successful execution are tested in their own function tests
fn execute_single_file_grep_test(){
    let reg = Regex::new("grep").unwrap();

    // Trying to execute in a dir
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    assert_eq!(CluErrors::InputError(format!("{} is dir. If you want to use grep recursively in a directory, add the -R flag. For more information try --help",pathbuf.to_str().unwrap().to_string())), Grep::execute_single_file_grep(reg.clone(), &pathbuf.to_str().unwrap().to_string(),0,0,0,false,false,false,false,false,false).expect_err(""));

    // Trying to read something unexistent
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("text.txt");
    assert_eq!(CluErrors::InputError(format!("{} cannot be read",pathbuf.to_str().unwrap().to_string())), Grep::execute_single_file_grep(reg.clone(), &pathbuf.to_str().unwrap().to_string(),0,0,0,false,false,false,false,false,false).expect_err(""));

    // Successful call
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    pathbuf.push("sample_text.txt");
    assert_eq!(vec!["2-How are you? Who are you?".to_string(), "3:I'm grep".to_string(), "4-Nice to meet you".to_string()], Grep::execute_single_file_grep(reg.clone(), &pathbuf.to_str().unwrap().to_string(),0,0,1,false,true,false,false,false,false).unwrap());
}

#[test]
fn search_test(){
    let reg = Regex::new("grep").unwrap();
    let contents = "Hey you\nHow are you? Who are you?\nI'm grep\nNice to meet you";
    // Without flags
    assert_eq!(vec!["I'm grep".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, false, false, false));
    // Before context to 1
    assert_eq!(vec!["How are you? Who are you?".to_string(), "I'm grep".to_string()],Grep::search(reg.clone(), contents.to_string(), 1, 0, 0, false, false, false));
    // After context to 1
    assert_eq!(vec!["I'm grep".to_string(), "Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 1, 0, false, false, false));
    // Before and after contect to 2
    assert_eq!(contents.split("\n").map(|item|item.to_string()).collect::<Vec<String>>(),Grep::search(reg.clone(), contents.to_string(), 2, 2, 0, false, false, false));
    // Context to 1
    assert_eq!(vec!["How are you? Who are you?".to_string(), "I'm grep".to_string(), "Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 1, false, false, false));
    // Before and after context to 2 but overriden by context to 1
    assert_eq!(vec!["How are you? Who are you?".to_string(), "I'm grep".to_string(), "Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 2, 2, 1, false, false, false));
    // Line number flag activated
    assert_eq!(vec!["3:I'm grep".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, true, false, false));
    // Line number flag + context
    assert_eq!(vec!["2-How are you? Who are you?".to_string(), "3:I'm grep".to_string(), "4-Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 1, true, false, false));
    // Invert match flag activated
    assert_eq!(vec!["Hey you".to_string(),"How are you? Who are you?".to_string(), "Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, false, true, false));
    // Line number + invert_match
    assert_eq!(vec!["1:Hey you".to_string(),"2:How are you? Who are you?".to_string(), "4:Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, true, true, false));
    // Line number + invert_match + context
    assert_eq!(vec!["1:Hey you".to_string(),"2:How are you? Who are you?".to_string(), "3-I'm grep".to_string(), "4:Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 1, true, true, false));
    // Only_matching flag
    assert_eq!(vec!["grep".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, false, false, true));
    // Only_matching + context
    assert_eq!(vec!["How are you? Who are you?".to_string(),"grep".to_string(),"Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 1, false, false, true));
    // Only_matching + line_number
    assert_eq!(vec!["3:grep".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, true, false, true));
    // Only_matching + context + line_number
    assert_eq!(vec!["2-How are you? Who are you?".to_string(),"3:grep".to_string(),"4-Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 1, true, false, true));
    // Only_matching + invert_match -> Invert match override only_matching
    assert_eq!(vec!["Hey you".to_string(),"How are you? Who are you?".to_string(), "Nice to meet you".to_string()],Grep::search(reg.clone(), contents.to_string(), 0, 0, 0, false, true, true));
}

#[test]
fn count_test(){
    let contents = "Hey you\nHow are you? Who are you?\nI'm grep\nNice to meet you";
    let reg1 = Regex::new("you").unwrap();
    let reg2 = Regex::new(r"[A-Z][a-z]").unwrap();

    assert_eq!(vec![String::from("3")], Grep::count(reg1, contents.to_string()));
    assert_eq!(vec![String::from("3")], Grep::count(reg2, contents.to_string()));
}

#[test]
fn total_count_test(){
    let contents = "Hey you\nHow are you? Who are you?\nI'm grep\nNice to meet you";
    let reg1 = Regex::new("you").unwrap();
    let reg2 = Regex::new(r"[A-Z][a-z]").unwrap();
    assert_eq!(vec![String::from("4")], Grep::total_count(reg1, contents.to_string()));
    assert_eq!(vec![String::from("4")], Grep::total_count(reg2, contents.to_string()));
}

#[test]
fn files_with_matches_test(){
    let contents = "Hey you\nHow are you? Who are you?\nI'm grep\nNice to meet you";
    let contents2 = "Hey";
    let reg = Regex::new("you").unwrap();
    assert_eq!(true, Grep::files_with_matches(reg.clone(), contents.to_string(), false));
    assert_eq!(false, Grep::files_with_matches(reg.clone(), contents.to_string(), true));
    assert_eq!(false, Grep::files_with_matches(reg.clone(), contents2.to_string(), false));
    assert_eq!(true, Grep::files_with_matches(reg, contents2.to_string(), true));
}

#[test]
fn context_lines_test(){
    let contents = "Hey you\nHow are you? Who are you?\nI'm grep\nNice to meet you";
    let filtered_lines = vec![(2 as usize, "I'm grep")];

    // Before test
    assert_eq!(HashMap::from([(1 as usize, "How are you? Who are you?")]), Grep::context_lines(contents, &filtered_lines, 0, 1, 4));
    // After test
    assert_eq!(HashMap::from([(3 as usize, "Nice to meet you")]), Grep::context_lines(contents, &filtered_lines, 1, 0, 4));
    // Before + after test
    assert_eq!(HashMap::from([(0,"Hey you"),(1, "How are you? Who are you?"),(3, "Nice to meet you")]), Grep::context_lines(contents, &filtered_lines, 1, 2, 4));
}

#[test]
fn parse_path_test(){
    // Well parsed test
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    pathbuf.push("*");
    // Without hidden items
    let result_without_hidden = Grep::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).unwrap();
    assert_eq!(1, result_without_hidden.len()); // It contains sample_text.txt

    // With hidden items
    let result_with_hidden = Grep::parse_path(&pathbuf.to_str().unwrap().to_string(), false, true).unwrap();
    assert_eq!(3, result_with_hidden.len()); // It contains the same as the previous + .hidden_folder + .hidden_text.txt

    // Invalid path due to invalid parent dir test
    assert_eq!(CluErrors::InputError("The introduced path: '' isn't valid.".to_string()), Grep::parse_path(&"".to_string(),false,false).expect_err(""));

    // Invalid regex in the search path
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("[a-s+");
    assert_eq!(CluErrors::RegexError("[a-s+".to_string()), Grep::parse_path(&pathbuf.to_str().unwrap().to_string(),false,false).expect_err(""));

    // Invalid file name in the search path
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("..");
    assert_eq!(CluErrors::InputError(format!("The introduced path: '{}' isn't valid.", pathbuf.to_str().unwrap())), Grep::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).expect_err(""));

    // Unable to read directory test. This happens if the user doesn't have permission to read the directory, or if it's introduced a regex that passes the filter but whose parent dir doesn't exist. We will use this second case to carry out the test
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("sc");
    pathbuf.push("*");
    assert_eq!(CluErrors::UnableToReadDirectory, Grep::parse_path(&pathbuf.to_str().unwrap().to_string(), false, false).expect_err(""));

}

#[test]
fn parse_path_recursively_test(){
    // Base case test, the path only contains files
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    pathbuf.push("sample_text.txt");
    let query = Grep::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], false).unwrap();
    assert_eq!(1, query.len()); 

    // Recursive case test, finding all the files in the directory tree
    let mut pathbuf = PathBuf::new();
    pathbuf.push(".");
    pathbuf.push("tests");
    pathbuf.push("grep_files");
    let without_hidden_files = Grep::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], false).unwrap();
    let with_hidden_files = Grep::parse_path_recursively(vec![pathbuf.to_str().unwrap().to_string()], true).unwrap();
    assert_eq!(1, without_hidden_files.len()); 
    assert_eq!(3, with_hidden_files.len()); 

    // The errors that may occur here are:
    // - Errors propagated from parse_path (already tested).
    // - Concurrency errors. There's no way to create a unit test of that as the concurrency is defined in the function so we cannot close the channel or panick a thread from here
}

#[test]
fn output_search_lines_test(){
    assert_eq!("Hey".to_string(),Grep::output_search_lines(0, "Hey", false, false));
    assert_eq!("Hey".to_string(),Grep::output_search_lines(0, "Hey", true, false));
    assert_eq!("1:Hey".to_string(),Grep::output_search_lines(0, "Hey", false, true));
    assert_eq!("1-Hey".to_string(),Grep::output_search_lines(0, "Hey", true, true));
}

#[test]
fn validate_commands_test(){
    let no_options_no_counters = Grep{
        pattern: Box::new(String::new()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: false, invert_match: false, only_matching: false},
        counters: Counters { count: false, total_count: false }
    };
    let only_options = Grep{
        pattern: Box::new(String::new()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: true, line_number: false, invert_match: true, only_matching: true},
        counters: Counters { count: false, total_count: false }
    };

    let only_counters = Grep{
        pattern: Box::new(String::new()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: false, line_number: false, invert_match: false, only_matching: false},
        counters: Counters { count: true, total_count: false }
    };

    let both_options_and_counters = Grep{
        pattern: Box::new(String::new()),
        path: vec![],
        dereference_recursive: false,
        ignore_case: false,
        hidden_items: false,
        context: Context { after_context: 0, before_context: 0, context: 0 },
        options: Options { files_with_matches: true, line_number: false, invert_match: true, only_matching: true},
        counters: Counters { count: true, total_count: false }
    };
    
    assert!(no_options_no_counters.validate_commands());
    assert!(only_options.validate_commands());
    assert!(only_counters.validate_commands());
    assert!(!both_options_and_counters.validate_commands());
}    