use clap::Args;

#[derive(Args)]
#[group(multiple=false)] // Counters are not compatible with Options, in addition they are not compatible with each other
pub struct Counters{
    /// Set this flag on to count the number of lines containing the pattern. Not allowed to use it together with -l, -n, -v, -o or --total-count.
    #[arg(short, long)]
    pub count: bool,

    /// Set this flag on to count the number of times the pattern is matched. Not allowed to use it together with -l, -n, -v, -o or -c.
    #[arg(long)]
    pub total_count: bool,
}