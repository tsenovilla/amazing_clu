use clap::Args;

#[derive(Args)]
#[group(multiple = true)]
pub struct Context{
    /// Use this argument to show a determined number of lines after the lines where the pattern has been matched. 
    /// Using it together with -l, -c or --total-count does not have any effect in the output
    #[arg(short='A', long, default_value_t = 0)]
    pub after_context: usize,

    /// Use this argument to show a determined number of lines bbefore the lines where the pattern has been matched. 
    /// Using it together with -l, -c or --total-count does not have any effect in the output
    #[arg(short='B', long, default_value_t = 0)]
    pub before_context: usize,

    /// Equivalent to set -A and -B to the same number. If -C is set, it'll shadow -A and -B.
    #[arg(short='C', long, default_value_t = 0)]
    pub context: usize,
}