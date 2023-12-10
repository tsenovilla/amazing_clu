use crate::{find::{Find, options::Options}, clu_errors::CluErrors};

#[test]
fn filter_by_name_test(){
    let parsed = vec!["sample.rs".to_string(), "sample.txt".to_string(), "sample2.txt".to_string(), "sample.py".to_string()]; // simulated parsed path passed to the function


    // Find a concrete file
    let find = Find{
        from: Box::new("".to_string()), // Not important for this test
        expression: Some(Box::new("sample.rs".to_string())),
        ignore_case: false,
        hidden_items: false,  // Not important for this test
        options: Options{name: true} // Not important for this test
    };
    assert_eq!(vec!["sample.rs".to_string()], find.filter_by_name(parsed.clone()).unwrap());

    // Find files using a pattern
    let find = Find{
        from: Box::new("".to_string()), // Not important for this test
        expression: Some(Box::new("*.txt".to_string())),
        ignore_case: false,
        hidden_items: false,  // Not important for this test
        options: Options{name: true} // Not important for this test
    };
    assert_eq!(vec!["sample.txt".to_string(), "sample2.txt".to_string()], find.filter_by_name(parsed.clone()).unwrap());

    // Find everything if no pattern specified
    let find = Find{
        from: Box::new("".to_string()), // Not important for this test
        expression: None,
        ignore_case: false,
        hidden_items: false,  // Not important for this test
        options: Options{name: true} // Not important for this test
    };
    assert_eq!(parsed, find.filter_by_name(parsed.clone()).unwrap());

    // Regex error
    let find = Find{
        from: Box::new("".to_string()), // Not important for this test
        expression: Some(Box::new("[a-z".to_string())),
        ignore_case: false,
        hidden_items: false,  // Not important for this test
        options: Options{name: true} // Not important for this test
    };
    assert_eq!(CluErrors::RegexError("[a-z".to_string()), find.filter_by_name(parsed.clone()).unwrap_err());
}