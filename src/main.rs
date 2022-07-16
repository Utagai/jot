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

static SHELL_ENV_VAR: &str = "SHELL";
static EDITOR_ENV_VAR: &str = "EDITOR";

fn edit(args: &cli::Cli) -> Result<()> {
    println!("Edit mode!, cwd: {}", current_dir().unwrap().display());
    // TODO: Possibly export this var + context to a mini helper.
    // TODO: Can we actually implement .context() for option and result such that we introduce a
    // contextf() which automatically runs format!()?
    let shell =
        var(SHELL_ENV_VAR).context(format!("failed to find ${} in environment", SHELL_ENV_VAR))?;
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
    let editor = var(EDITOR_ENV_VAR)
        .context(format!("failed to find ${} in environment", EDITOR_ENV_VAR))?;
    println!("EDITOR: {}", editor);

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
