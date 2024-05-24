use clap::{
    Parser,
    command
};

/// A utility for parsing through git repos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// The path to the repo
    #[clap(short, long)]
    pub path: String,

    /// The branch being targeted
    #[clap(short, long)]
    pub branch: String,

    /// Enable parsing by email
    #[clap(short, long, default_value=None)]
    pub email: Option<String>,

    /// Enable parsing by committer name
    #[clap(short, long, default_value=None)]
    pub committer: Option<String>,

    /// The file to write the output to
    #[clap(short, long, default_value=None)]
    pub outfile: Option<String>,

    /// Flag as to if this should start a server.
    #[clap(short, long, action)]
    pub server: bool,

    /// The directory for the static server files.
    #[clap(short='D', long, default_value=None)]
    pub server_directory: Option<String>,
}
