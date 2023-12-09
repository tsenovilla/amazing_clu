use clap::Args;
use regex::{Regex,RegexBuilder};
use std::{fs,collections::HashMap, path::Path, thread::Builder, sync::mpsc};

mod context;
mod options;
mod counters;
use crate::{base, clu_errors::CluErrors, grep::{context::Context, options::Options, counters::Counters}};

#[derive(Args)]
pub struct Grep{
    /// Specify the pattern to use in your search with this argument. If you use a regular expresion, wrap it with "".
    pattern: Box<String>,

    /// Specify the path to the file where you want to perform the search. It must be a Vec<String> in order to accept automatically globbed paths (if globbing takes place due to the OS)
    path: Vec<String>,

    /// Set this flag on if your path is a directory and you want to check within all the files inside the directory and its subdirectories
    #[arg(short = 'R', long)]
    dereference_recursive: bool,

    /// Set this flag on to make the search pattern case insensitive.
    #[arg(short, long)]
    ignore_case: bool,

    /// By default, grep ignores the hidden files and directories (those starting with .). Set this flag on to explicitly search inside of them. Note: If you're looking for hidden files with a determined extension, adding to the path *.txt -H may not be enough if your OS carries out globbing automatically. If that's the case, wrap the pattern into single quotes to ensure the globbing is carried out by the utility instead of by the OS
    #[arg(short = 'H', long)]
    hidden_items:bool,

    // Context
    #[command(flatten)]
    context: Context,

    // Options
    #[command(flatten)]
    options: Options,

    // Counters
    #[command(flatten)]
    counters: Counters
}

impl Grep{

    pub fn execute(mut self) -> Result<String, CluErrors>
    {
        // Check if the introduced command combination is valid
        if !self.validate_commands(){ 
            return Err(CluErrors::InvalidCommandCombination(String::from("grep"))) 
        }
        let mut path = if self.path.len() > 1{
            std::mem::replace(&mut self.path, Vec::new()) // As self.path is not required to be part of self after assigning it here, we take it out and hold it in path, leaving an empty Vector in self. Then, the contents of path can be safely spawned among threads without having to export self with them.
        }
        else{
            base::parse_path(&self.path[0], false, self.hidden_items)? // If globbing hasn't taken place, we manually do it through the parse_path function
        };

        if self.dereference_recursive{
            path = base::parse_path_recursively(path, self.hidden_items)?;
        }

        // Gets the request and converts it into a single String to be printed
        let search = self.execute_multiple_files_grep(path)?
            .join("\n")
            .trim_end_matches("\n")
            .trim_start_matches("\n")
            .to_string();
        if search.is_empty(){ return Err(CluErrors::NotFoundError);}
        Ok(
            search
        )
    }

    // This function executes the desired action in each single file. There's so many inputs that may be passed just using self right? Check the next function out to find out the reason!
    fn execute_single_file_grep(
        reg: Regex,
        file: &String, 
        before_context: usize, 
        after_context: usize, 
        context: usize, 
        files_with_matches: bool,
        line_number: bool, 
        invert_match:bool, 
        only_matching: bool, 
        count: bool, 
        total_count: bool
    ) -> Result<Vec<String>, CluErrors>{
        if Path::new(&file).is_dir(){
            return Err(CluErrors::InputError(format!("{} is dir. If you want to use grep recursively in a directory, add the -R flag. For more information try --help", file)));
        }
        let contents = fs::read_to_string(&file).map_err(|_error| CluErrors::InputError(format!("{} cannot be read",file)))?;

        // Return a  Vec<String> containing all the Strings to be printed. We compute using the appropriate function depending on user's input
        if count{
            Ok(Self::count(reg, contents))
        }
        else if total_count{
            Ok(Self::total_count(reg, contents))
        }
        else if files_with_matches{
            if Self::files_with_matches(reg, contents, invert_match){
                Ok(vec![file.clone()])
            }
            else{
                Ok(vec![])
            }
        }
        else{
            Ok(Self::search(reg, contents, before_context, after_context, context, line_number, invert_match, only_matching))
        }
    }

    // This function is call when the path of files to be explored is known to perform the grep action in each of them concurrently
    fn execute_multiple_files_grep(
        &self,
        paths: Vec<String>
    ) -> Result<Vec<String>, CluErrors>{

        let mut search = Vec::new();

        // These arguments are going to be spawned into threads in order to call the required functions for each file. As all of them are copy arguments, we make here a copy of each of them in order to send it among the threads. Why not passing self to the functions? Well, as self cannot implement the copy trait due to its pattern and path arguments, passing self to functions that will be called inside threads means that we have to handle how to pass self through the threads. An Arc, Mutex construction might be a solution, however, as our threads only mission is to call other functions, the MutexGuard would be locked until the end of each thread and then there's no concurrency. In this way, defining function that don't depend on self but on its copy arguments, we can spawn the threads more efficiently.
        let before_context = self.context.before_context;
        let after_context = self.context.after_context;
        let context = self.context.context;
        let files_with_matches = self.options.files_with_matches;
        let line_number = self.options.line_number;
        let invert_match = self.options.invert_match;
        let only_matching = self.options.only_matching;
        let count = self.counters.count;
        let total_count = self.counters.total_count;

        let mut handles = Vec::new();
        let (tx, rx) = mpsc::channel();
        for file in paths{
            let tx1 = tx.clone();
            // Build the regex. We build one for each thread as otherwise we need to use an Arc,Mutex construction, however, as reg is used in the Grep functions, the Mutex'd be locked until the end of the execution in each thread, so the only concurrent part would be the send of the message which isn't good enough. 
            let reg = RegexBuilder::new(self.pattern.as_str())
                .case_insensitive(self.ignore_case)
                .build()
                .map_err(|_err| CluErrors::RegexError(self.pattern.to_string()))?;
            handles.push(Builder::new().spawn(move || -> Result<(), CluErrors>{
                let mut call = Self::execute_single_file_grep(reg, &file, before_context, after_context, context, files_with_matches, line_number, invert_match, only_matching, count, total_count)?;
                if call.is_empty(){ // This is not an error, the search is just empty for this file but it can be successful somewhere else
                    return Ok(());
                }
                let mut output = vec![
                    if !files_with_matches{format!("\n\t----{file}----\n")} else{String::new()}
                ];
                output.append(&mut call);
                tx1.send(output).map_err(|_err|CluErrors::UnexpectedError)?;
                Ok(())
            }).map_err(|_err| CluErrors::UnexpectedError)?);
        }
        for handle in handles{
            handle.join().map_err(|_err| CluErrors::UnexpectedError)??; // We handle errors from the thread or from the join
        }
        loop{ // The output for the files are pending in the receiver, let's get them!
            match rx.try_recv(){
                Ok(mut sent_output) => search.append(&mut sent_output),
                Err(_) => break
            }
        }

        Ok(search)
    }


    // This function is the core of Grep. It computes the search in so many cases, except if the user requested a count or a files with matches
    fn search(
        reg: Regex, 
        contents: String,
        before_context: usize,
        after_context: usize,
        context: usize,
        line_number_flag: bool,
        invert_match:bool,
        only_matching: bool
    ) -> Vec<String>{
        // Collect the lines that match the pattern together with its line number into a Vec. Note that if invert_match is selected, the inversion is applied while filtering.
        let filtered_lines = contents
            .lines()
            .enumerate()
            .filter(|(_line_number, line)| reg.is_match(line) ^invert_match)
            .collect::<Vec<(usize, &str)>>();

        // We compute after and before, also create a HashMap that will store the lines out of the pattern_match affected by them.
        let mut context_lines = HashMap::new();
        let mut before = before_context;
        let mut after= after_context;
        if context > 0{
            before = context;
            after = context;
        }
        let contents_length = contents.lines().count(); // This quantity may be used several times

        // If a context has been defined, we save the lines to use (note that they don't match the pattern as otherwise they'll be shown anyway, so they'll be shown complete even if -o is selected) into the context_lines HashMap.
        if after > 0 || before > 0{
            context_lines = Self::context_lines(&contents, &filtered_lines, after, before, contents_length);
        }

        //This is the vector String to output. The starting point are the lines that matched the pattern
        filtered_lines
            .iter()
            .map(|(line_number, line)| { // Create an iterator of Vec<String> containing the lines to be printed. This also includes the context affected lines!
                let mut output = Vec::new();
                // Compute this line range affected by the context
                let lines_before = usize::try_from(*line_number as isize - before as isize).unwrap_or(0);
                let lines_after = if line_number+after > contents_length-1 {contents_length-1} else {line_number + after};
                // The context lines must also respect the order, so we push the before lines to the output Vec before pushing the current line. Note that we push each context line just once in the whole program.
                (lines_before..*line_number)
                .for_each(|position|{
                    if context_lines.contains_key(&position){
                        output.push(Self::output_search_lines(position, context_lines.remove_entry(&position).unwrap().1, true, line_number_flag))
                    }
                });
                // Now push the current line. If -o is activated, we have to push each single ocurrence
                if only_matching && !invert_match{
                    reg
                        .find_iter(line)
                        .for_each(|matched| 
                            output.push(Self::output_search_lines(*line_number, matched.as_str(), false, line_number_flag))
                        );
                }
                else{
                    output.push(Self::output_search_lines(*line_number, line, false, line_number_flag));
                }
                // Push the lines after.
                (*line_number..lines_after+1)
                .for_each(|position|{
                    if context_lines.contains_key(&position){
                        output.push(Self::output_search_lines(position, context_lines.remove_entry(&position).unwrap().1, true, line_number_flag))
                    }
                });
                output
            })
            .flatten()
            .collect()
    }

    // To call if -c is set. It counts how many lines contain the pattern
    fn count(reg: Regex, contents: String) -> Vec<String>{
        vec![
            contents
            .lines()
            .filter(|line| reg.is_match(line))
            .count()
            .to_string()
        ]
    }

    // To call if --total-count is set. It contains the number of times the pattern is matched.
    fn total_count(reg: Regex, contents: String) -> Vec<String>{
        vec![
            reg.find_iter(&contents)
            .count()
            .to_string()
        ]
    }

    // To call if files_with_matches is set, it finds the files containing something that matchs the pattern
    fn files_with_matches(reg: Regex, contents: String, invert_match:bool) -> bool{
        reg.find(&contents).is_some() ^invert_match
    }

    // This function is called by search to determine the HashMap of lines affected by the context
    fn context_lines<'a>
    (  
        contents: &'a str,
        filtered_lines: &Vec<(usize, &str)>,
        after: usize,
        before: usize,
        contents_length: usize
    ) -> HashMap<usize, &'a str>{
        let mut context_lines= HashMap::new();
        // A starting point are the files that matched the pattern, passed here by the search function
        filtered_lines
            .iter()
            .for_each(|(line_number, _line)|{ 
                // For each line we find how many lines are before and after that may be affected by context
                let lines_before = usize::try_from(*line_number as isize - before as isize).unwrap_or(0);
                let lines_after = if line_number+after > contents_length-1 {contents_length-1} else {line_number + after};
                // We find the lines affected by the context of the current line:
                // 1. We ensure they've not been taken in count yet.
                // 2. We ensure they're not part of the lines that matched the pattern.
                let lines = (lines_before..lines_after+1)
                    .filter(|position|
                        !context_lines.contains_key(position) 
                        && 
                        !filtered_lines.iter().any(|(contained, _line)| contained==position)
                    )
                    .collect::<Vec<usize>>();
                // Add the resulting lines to the HashMap
                contents
                    .lines()
                    .enumerate()
                    .filter(|(line_number, _line)| lines.contains(line_number))
                    .for_each(|(line_number, line)| {context_lines.insert(line_number, line);});
            });
        context_lines
    }

    // This function is called by search. It just produce a String containing its corresponding line_number if needed
    fn output_search_lines(line_number: usize, line: &str, is_context_line: bool, line_number_flag: bool) -> String{
        format!(
            "{}{}{}", 
            if line_number_flag{(line_number+1).to_string()} else {"".to_string()}, 
            if line_number_flag{if is_context_line{"-"} else{":"}}else{""},
            line
        )
    } 

    // This function is called by execute to determine if a command of the group Options has been used in combination with a command of the group Counters. Note that this only works because both are structs composed by bools, then it's enough to check its bytes.
    fn validate_commands(&self) -> bool{
        !(
            base::get_bytes(&self.options).iter().any(|&x| x!=0) 
            && 
            base::get_bytes(&self.counters).iter().any(|&x| x!=0)
        )
    }

}

#[cfg(test)]
mod unit_tests;