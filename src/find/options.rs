use clap::Args;

#[derive(Args)]
#[group(multiple = false)]
pub struct Options{
    // Set this flag on to perform a research by name.
    #[arg(long)]
    pub name: bool
}