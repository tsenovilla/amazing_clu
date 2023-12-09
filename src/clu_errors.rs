#[derive(Debug, PartialEq)]
pub enum CluErrors{
    MissingCommand,
    InvalidCommandCombination(String),
    InputError(String),
    UnableToReadDirectory,
    NotFoundError,
    RegexError(String),
    UnexpectedError
}

impl std::fmt::Display for CluErrors{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::MissingCommand => write!(f, "Missing utility command. For more information try --help."),
            Self::InvalidCommandCombination(mode) => write!(f,"Introduced an invalid combination of commands in {mode} mode. For more information try --help."),
            Self::InputError(reason) => write!(f, "Input error. {reason}."),
            Self::UnableToReadDirectory => write!(f, "We've encountered a problem reading one of the provided directories, please ensure that the path is correct and that lecture permissions are enabled."),
            Self::NotFoundError => write!(f, "The request didn't produce any output."),
            Self::RegexError(regex) => write!(f, "The introduced regex {regex} isn't valid."),
            Self::UnexpectedError => write!(f, "The execution stopped due to an unexpected error.")
        }
    }
}