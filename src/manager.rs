use anyhow::{bail, Result};

use crate::config::Config;
use crate::runner;

/// Initialize .be directory.
pub fn init() -> Result<()> {
    let path = Config::init()?;
    println!("Initialized .be at {}", path.display());
    Ok(())
}

/// List all saved commands.
pub fn list() -> Result<()> {
    let (config, _) = Config::load()?;

    if config.commands.is_empty() {
        println!("No commands saved yet. Use `be add <tag> -c <cmd>` to add one.");
        return Ok(());
    }

    // Calculate column widths for alignment
    let max_tag_len = config.commands.keys().map(|k| k.len()).max().unwrap_or(0);
    let tag_width = max_tag_len.max(3); // minimum width for "Tag"

    println!("{:<width$}  Description", "Tag", width = tag_width);
    println!("{:<width$}  -----------", "---", width = tag_width);

    for (tag, entry) in &config.commands {
        let desc = if entry.description.is_empty() {
            &entry.command
        } else {
            &entry.description
        };
        println!("{:<width$}  {}", tag, desc, width = tag_width);
    }

    Ok(())
}

/// Add a new command.
pub fn add(tag: &str, cmd: &str, desc: &str) -> Result<()> {
    let (mut config, path) = Config::load()?;
    config.add(tag, cmd, desc)?;
    config.save(&path)?;
    println!("Added '{}': {}", tag, cmd);
    Ok(())
}

/// Remove a command.
pub fn remove(tag: &str) -> Result<()> {
    let (mut config, path) = Config::load()?;
    config.remove(tag)?;
    config.save(&path)?;
    println!("Removed '{}'", tag);
    Ok(())
}

/// Edit an existing command.
pub fn edit(tag: &str, cmd: Option<&str>, desc: Option<&str>) -> Result<()> {
    if cmd.is_none() && desc.is_none() {
        bail!("Nothing to edit. Provide -c <cmd> and/or -d <desc>.");
    }

    let (mut config, path) = Config::load()?;
    config.edit(tag, cmd, desc)?;
    config.save(&path)?;
    println!("Updated '{}'", tag);
    Ok(())
}

/// Run a command by tag.
pub fn run_by_tag(tag: &str) -> Result<()> {
    let (config, _) = Config::load()?;

    match config.get(tag) {
        Some(entry) => runner::run_and_exit(&entry.command),
        None => bail!("Unknown tag '{}'. Use `be list` to see available commands.", tag),
    }
}
