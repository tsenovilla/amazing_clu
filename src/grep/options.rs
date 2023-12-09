use clap::Args;

#[derive(Args)]
#[group(multiple=true)] // These arguments are grouped because they are not compatible with the arguments in Counters, so we use this ArgGroup to handle this possibility
pub struct Options{
    /// Set this flag on to find which files matche the pattern. 
    /// Used in combination with -v will return the files whose contents does NOT much the pattern. 
    /// The behavior is not affected if used together with -n or -o, an error is not thrown anyway.
    /// Not allowed to use it together with -c or --total-count.
    #[arg(short='l', long)]
    pub files_with_matches: bool,

    /// Set this flag on to enumerate the lines where the occurrence happened. Not allowed to use it together with -c or --total-count.
    #[arg(short='n', long)]
    pub line_number: bool,

    /// Set this flag on to find the lines where the pattern is not satisfied. Not allowed to use it together with -c or --total-count.
    #[arg(short = 'v', long)]
    pub invert_match: bool,

    /// Set this flag on to find only the expressions that match the pattern, not their corresponding lines.
    /// If used with -v, then -v takes preference, as the inverted lines won't contain the pattern.
    #[arg(short, long)]
    pub only_matching: bool
}