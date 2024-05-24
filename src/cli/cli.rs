use clap::{
    Parser,
    command
};

/// A utility for parsing through git repos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the repo
    #[arg(short, long)]
    pub path: String,

    /// The branch being targeted
    #[arg(short, long)]
    pub branch: String,

    /// Enable parsing by email
    #[arg(short, long, default_value=None)]
    pub email: Option<String>,

    /// Enable parsing by committer name
    #[arg(short, long, default_value=None)]
    pub committer: Option<String>,

    /// The file to write the output to
    #[arg(short, long, default_value=None)]
    pub outfile: Option<String>,
}
