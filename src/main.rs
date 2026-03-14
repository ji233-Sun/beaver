mod cli;
mod config;
mod manager;
mod runner;
mod tui;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init) => manager::init(),
        Some(Command::Manager) => tui::run_tui(),
        Some(Command::List) => manager::list(),
        Some(Command::Add { tag, cmd, desc }) => manager::add(&tag, &cmd, &desc),
        Some(Command::Remove { tag }) => manager::remove(&tag),
        Some(Command::Edit { tag, cmd, desc }) => manager::edit(&tag, cmd.as_deref(), desc.as_deref()),
        Some(Command::Run(args)) => {
            if args.is_empty() {
                bail!("No tag provided");
            }
            manager::run_by_tag(&args[0])
        }
        None => {
            // No subcommand: show help
            use clap::CommandFactory;
            Cli::command().print_help()?;
            println!();
            Ok(())
        }
    }
}
