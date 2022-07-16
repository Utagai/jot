use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod cmd;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    std::env::set_current_dir(&args.base_dir).context(format!(
        "failed to change jot's working directory to {}",
        args.base_dir.display(),
    ))?;

    // TODO: Maybe we should also check that the repo is clean and not in conflict?

    match args.command.as_ref().unwrap_or(&cli::Command::Edit) {
        cli::Command::New { path } => cmd::new(&args, path),
        cli::Command::Edit => cmd::edit(&args),
        cli::Command::List { subpath } => cmd::list(&args, subpath.clone()),
        cli::Command::Synch => cmd::sync(&args),
    }?;

    Ok(())
}
