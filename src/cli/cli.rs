use clap::{
    Parser,
    command
};

/// A utility for parsing through git repos
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// The path to the repo
    #[clap(short, long, default_value=".")]
    pub directory: String,

    /// The branch being targeted
    #[clap(short, long, default_value="main")]
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
    #[clap(short='S', long, action)]
    pub server: bool,

    /// The directory for the static server files.
    #[clap(short='D', long, default_value="./static")]
    pub server_directory: String,

    /// The port to run the server on.
    #[clap(short='P', long, default_value="8080")]
    pub server_port: u32,
}
