use std::{
    env::current_dir,
    env::var,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;

mod cli;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    println!("{:?}", args);
    match args.command.as_ref().unwrap_or(&cli::Command::Edit) {
        cli::Command::Edit => edit(&args),
        cli::Command::List { .. } => list(),
        cli::Command::Synch => synch(),
    }?;

    Ok(())
}

static SHELL_ENV_VARNAME: &str = "SHELL";
static EDITOR_ENV_VARNAME: &str = "EDITOR";

fn get_env_var(varname: &str) -> Result<String> {
    var(varname).context(format!("failed to find ${} in environment", varname))
}

fn edit(args: &cli::Cli) -> Result<()> {
    println!("Edit mode!, cwd: {}", current_dir().unwrap().display());
    let shell = get_env_var(SHELL_ENV_VARNAME)?;
    let finder_exec = Command::new(shell)
        .arg(&args.shell_cmd_flag)
        .arg(&args.finder)
        .stdin(if args.capture_stderr {
            Stdio::piped()
        } else {
            Stdio::inherit()
        }) // Allow stderr to pass through for applications like fzf.
        .stderr(Stdio::inherit()) // Allow stderr to pass through for applications like fzf.
        .output()
        .context(format!("failed to execute finder: `{}`", args.finder))?;

    let stdout_output = std::str::from_utf8(finder_exec.stdout.as_ref())?;
    let stderr_output = if args.capture_stderr {
        std::str::from_utf8(finder_exec.stderr.as_ref())?
    } else {
        "<jot: stderr not captured; consider using the capture_stderr flag>"
    };

    // TODO: We should have an option for being quiet about non-zero exit codes.
    if !finder_exec.status.success() {
        // TODO: Should print more information on the unsuccessful exit, e.g. code or signal.
        // TODO: And ditto for below:
        return Err(anyhow!(
            "finder (`{}`) exited unsuccessfully with non-zero exit code\nstdout:\n\"{}\"\nstderr:\n\"{}\"",
            args.finder,
            stdout_output,
            stderr_output,
        ));
    }

    let filepath = Path::new(stdout_output.trim());
    let editor = get_env_var(EDITOR_ENV_VARNAME)?;

    // TODO: This command handling is code duplication. We can and should refactor.
    let editor_exec = Command::new(editor)
        .arg(filepath)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()?;
    if !editor_exec.status.success() {
        // TODO: Should include stderr output.
        return Err(anyhow!(
            "editor exited unsuccessfully with non-zero exit code"
        ));
    }

    Ok(())
}

fn list() -> Result<()> {
    println!("List mode!");
    Ok(())
}

fn synch() -> Result<()> {
    println!("Sync mode!");
    Ok(())
}
