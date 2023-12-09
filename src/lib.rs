use clap::{Parser, Subcommand};

mod clu_errors;
use clu_errors::CluErrors;
mod grep;

#[derive(Subcommand)]
enum Utility{
    /// Grep: With grep, you can search for a regular expression contained in a specified file path. By default, grep ignores hidden files and directories (whose names start with .), but you can tell grep to also look into them by adding the command flag -H. To check out all the possibilities offered by this implementation try amazing_clu grep --help.
    Grep(grep::Grep),
    
    /// Find:
    Find
}

impl Utility{
    fn execute(self) -> Result<String,CluErrors>{
        match self{
            Self::Grep(grep) => grep.execute(),
            _=> Ok(String::new())
        }
    }
}

#[derive(Parser)]
#[command(author = "Tomás Senovilla", version = "0.1.0", about = "Small clu utilities project", long_about = None)]
pub struct Clu{
    #[command(subcommand)]
    utility: Option<Utility>
}

impl Clu{
    pub fn run() -> Result<String, CluErrors>{
        Ok(
            Self::parse()
                .utility
                .ok_or(CluErrors::MissingCommand)?
                .execute()?
        )
    }
}