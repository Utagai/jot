use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    println!("{:?}", args);
    match args.command.as_ref().unwrap_or(&cli::Command::Edit) {
        cli::Command::Edit => cmd::edit(&args),
        cli::Command::List { .. } => cmd::list(),
        cli::Command::Synch => cmd::synch(),
    }?;

    Ok(())
}
