use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;

mod cli;
mod cmd;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    // First, set jot to be into the base_dir, since that is the point from which all our commands
    // should be executing from.
    std::env::set_current_dir(&args.base_dir).context(format!(
        "failed to change jot's working directory to {}",
        args.base_dir.display(),
    ))?;

    // Second, check that base-dir is a git repository:
    let status = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .status()
        .context("failed to determine if base-dir is a git repository")?;
    if !status.success() {
        bail!(
            "base-dir ({}) must be a git repository",
            args.base_dir.display()
        )
    }

    // Third, check that the base-dir is clean.
    let status = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("HEAD")
        .arg("--")
        .status()
        .context("failed to determine if base-dir is clean")?;
    if !status.success() {
        bail!(
            "base-dir ({}) is not clean, please fix the issue and run jot again",
            args.base_dir.display()
        )
    }

    match args.command.as_ref().unwrap_or(&cli::Command::Edit) {
        cli::Command::New { path } => cmd::new(&args, path),
        cli::Command::Edit => cmd::edit(&args),
        cli::Command::List { subpath } => cmd::list(&args, subpath.clone()),
        cli::Command::Synch => cmd::sync(&args),
    }?;

    Ok(())
}
