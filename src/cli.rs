use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "be", about = "Turn complex shell commands into simple tags")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize .be directory in current path
    Init,

    /// Open TUI manager
    Manager,

    /// List all saved commands
    List,

    /// Add a new command
    Add {
        /// Tag name (alphanumeric, underscore, hyphen)
        tag: String,
        /// Shell command to execute
        #[arg(short, long)]
        cmd: String,
        /// Description of the command
        #[arg(short, long, default_value = "")]
        desc: String,
    },

    /// Remove a command by tag
    Remove {
        /// Tag name to remove
        tag: String,
    },

    /// Edit an existing command
    Edit {
        /// Tag name to edit
        tag: String,
        /// New shell command
        #[arg(short, long)]
        cmd: Option<String>,
        /// New description
        #[arg(short, long)]
        desc: Option<String>,
    },

    /// Run a command by tag (catch-all for `be <tag>`)
    #[command(external_subcommand)]
    Run(Vec<String>),
}
