use std::{mem, slice, env, path::{Path, PathBuf}, thread::Builder, sync::mpsc};
use regex::Regex;
use crate::clu_errors::CluErrors;

pub fn get_bytes<T>(input: &T) -> &[u8] {
    let size = mem::size_of::<T>();
    unsafe {
        slice::from_raw_parts(
            input as *const T as *const u8,
            size,
        )
    }
}

// This function is used to parse a path into all the valid items. Eg: foo/txt will find all the items inside foo containing txt in its name
pub fn parse_path(path: &String, recursively_executed: bool, hidden_items: bool) -> Result<Vec<String>, CluErrors>{
    let path_object = Path::new(path);
    let current_dir = env::current_dir().map_err(|_err| CluErrors::UnableToReadDirectory)?; 
    let parent = match path_object.parent(){ // Get the dir where we research if possible. If a path pattern to search in the current directory has been introduced (eg, *.txt), then we have to return the current directory.
        Some(dir) => if dir.as_os_str().is_empty(){
            current_dir.as_ref()
        }
        else{
            dir
        },
        None => return Err(CluErrors::InputError(format!("The introduced path: '{}' isn't valid.", path)))
    };
    let reg = match path_object.file_name(){ // Get the regex use to search in parent
        Some(pattern) => {
            // If the pattern starts with *, in order to search everything in a dir (eg, foo/*) then we insert a "." before to use the global regex. Remember to take account of hidden items if necessary
            let pattern = pattern.to_str()
                .map(|pattern| if pattern.starts_with("*"){
                    if hidden_items{
                        format!(".{}", pattern)
                    }
                    else {
                        format!(r"^[^\.].{}",pattern)
                    }
                }else{
                    pattern.to_string()
                })
                .unwrap(); // Unwrap is OK as pattern comes from path which is already a valid String, then pattern.to_str cannot be None
            Regex::new(&pattern)
                .map_err(|_err| CluErrors::RegexError(String::from(pattern)))?
        },
        None => return Err(CluErrors::InputError(format!("The introduced path: '{}' isn't valid.", path)))
    };
    
    let parsed: Vec<String> = parent.read_dir().map_err(|_err|CluErrors::UnableToReadDirectory)?
            .filter_map(|item| item.ok()) // If the item is not readable we ignore it
            .filter_map(|item| item.file_name().into_string().ok()) // Again ignore if it's not readable
            .filter(|item| reg.is_match(item))
            .map(|item|{
                let mut pathbuf = PathBuf::from(parent);
                pathbuf.push(item);
                pathbuf.to_str().unwrap().to_string() // Unwrap is Ok as parent comes from path which is already a valid String
            })
            .collect();
    if parsed.len() == 0 && !recursively_executed{ // If executed recursively, there's no problem if a directory is empty
        return Err(CluErrors::InputError(format!("Reading an empty directory at: {}.", path)));
    }
    Ok(parsed)
}

// Parse path recursively down in the directories tree.
pub fn parse_path_recursively(path: &Vec<String>, hidden_items: bool) -> Result<Vec<String>,CluErrors>{
    if path.iter().all(|item| Path::new(item).is_file()){
        return Ok(path.clone()); // Base case, everything is a file
    }
    // Find which elements are files and which ones are dirs
    let mut files: Vec<String> = path
        .iter()
        .filter(|item| Path::new(item).is_file())
        .map(|file| file.to_string())
        .collect(); 
    let dirs: Vec<String> = path
        .iter()
        .filter(|item| Path::new(item).is_dir())
        .filter_map(|dir| {
            let mut dir = PathBuf::from(dir);
            dir.push("*"); // If we're going down in the directory tree, add a * to find everything inside this directory
            match dir.to_str(){
                Some(dir) => Some(dir.to_string()),
                None => None // Non readable dirs are ignored
            }
        })
        .collect();

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new(); // Call this function recursively and concurrently in order to find all the files
    for dir in dirs{
        let tx1 = tx.clone();
        handles.push(Builder::new().spawn(move || -> Result<(), CluErrors>{
            let call = parse_path_recursively(&parse_path(&dir, true, hidden_items)?, hidden_items)?;
            tx1.send(call).map_err(|_err| CluErrors::UnexpectedError)?;
            Ok(())
        }).map_err(|_err| CluErrors::UnexpectedError)?);
    }

    for handle in handles{
        handle.join().map_err(|_err| CluErrors::UnexpectedError)??; // We handle errors from the thread or from the join
    }

    loop{
        match rx.try_recv(){
            Ok(mut sent_output) => files.append(&mut sent_output),
            Err(_) => break
        }
    }
    Ok(files)
}

#[cfg(test)]
mod unit_tests;