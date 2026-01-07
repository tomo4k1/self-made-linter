use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Files to parse
    #[arg(required = true)]
    pub files: Vec<String>,

    /// Apply auto-fixes
    #[arg(long)]
    pub fix: bool,

    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
}
