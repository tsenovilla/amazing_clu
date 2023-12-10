use clap::Args;
use regex::RegexBuilder;

mod options;
use crate::{base, clu_errors::CluErrors, find::options::Options};

#[derive(Args)]
pub struct Find{
    
    /// Specify the directory where you want to start searching from
    from:Box<String>,

    /// Specify what you're looking for using a Regex. If empty, find we'll look for all the files in the directory tree. Note: It's better to wrap this argument with quotes, otherwise your shell may reject it. Example: If you write *.txt, your shell may try to find something called *.txt in your current directory and reject the expression directly, then it's better to use find . '*.txt'
    expression: Option<Box<String>>, //It must be a Vec<String> in order to accept automatically globbed paths (if globbing takes place)

    /// Set this flag on to make the search expression case insensitive.
    #[arg(short, long)]
    ignore_case: bool,

    /// By default, find ignores the hidden files and directories (those starting with .). Set this flag on to explicitly include them in the search. Note: If you're looking for hidden files with a determined extension, adding to the path *.txt -H may not be enough if your shell carries out globbing automatically. If that's the case, wrap the pattern into single quotes to ensure the globbing is carried out by the utility instead of by the shell.
    #[arg(short = 'H', long)]
    hidden_items:bool,

    #[command(flatten)]
    options: Options
}

impl Find{

    pub fn execute(self) -> Result<String, CluErrors>{        
        // Find the requested files
        let parsed = base::parse_path_recursively(vec![self.from.to_string()], self.hidden_items)?;

        let filter = self.filter_by_name(parsed)?;

        let found = filter
            .join("\n")
            .trim_end_matches("\n")
            .trim_start_matches("\n")
            .to_string();
        if found.is_empty(){ return Err(CluErrors::NotFoundError);}
        Ok(
            found
        )
    }

    fn filter_by_name(&self, parsed: Vec<String>) -> Result<Vec<String>, CluErrors>{
        // Extract the expression into a String depending on it's shape
        let expression = match &self.expression{
            Some(expression) => {
                if expression.starts_with("*"){
                    format!(".{}", expression)
                }
                else{
                    expression.to_string()
                }
            },
            None => ".*".to_string()
        };

        // Build the Regex
        let reg = RegexBuilder::new(&expression)
            .case_insensitive(self.ignore_case)
            .build()
            .map_err(|_err| CluErrors::RegexError(expression))?;

        Ok(
            parsed
                .iter()
                .filter_map(|item|{
                    if reg.is_match(item){ Some(item.to_string()) }
                    else{ None }
                })
                .collect::<Vec<String>>()
        )
    }

}

#[cfg(test)]
mod unit_tests;