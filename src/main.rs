use std::{
    borrow::Cow,
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

fn exec_cmd(label: &str, mut cmd: Command, captured_stderr: bool) -> Result<String> {
    let program = cmd.get_program();
    let joined_args_str = cmd
        .get_args()
        .map(|os_str| os_str.to_string_lossy())
        .collect::<Vec<Cow<'_, str>>>()
        .join(" ");
    let invocation = format!("{:?} {:?}", program, joined_args_str);
    let exec = cmd
        .output()
        .context(format!("failed to execute {}: `{}`", label, invocation,))?;

    let stdout_output = std::str::from_utf8(exec.stdout.as_ref())?;
    let stderr_output = if captured_stderr {
        std::str::from_utf8(exec.stderr.as_ref())?
    } else {
        "<jot: stderr not captured>"
    };

    // TODO: We should have an option for being quiet about non-zero exit codes.
    // TODO: Should print more information on the unsuccessful exit, e.g. code or signal.
    if !exec.status.success() {
        return Err(anyhow!(
            "{} (`{}`) exited unsuccessfully with non-zero exit code\nstdout:\n\"{}\"\nstderr:\n\"{}\"",
            label,
            invocation,
            stdout_output,
            stderr_output,
        ));
    }

    Ok(stdout_output.trim().to_string())
}

fn edit(args: &cli::Cli) -> Result<()> {
    let shell = get_env_var(SHELL_ENV_VARNAME)?;
    let mut finder_cmd = Command::new(shell);
    finder_cmd
        .arg(&args.shell_cmd_flag)
        .arg(&args.finder)
        .stdin(if args.capture_stderr {
            Stdio::piped()
        } else {
            Stdio::inherit()
        }) // Allow stderr to pass through for applications like fzf.
        .stderr(Stdio::inherit()); // Allow stderr to pass through for applications like fzf.

    let finder_stdout = exec_cmd("finder", finder_cmd, args.capture_stderr)?;
    let filepath = Path::new(&finder_stdout);
    let editor = get_env_var(EDITOR_ENV_VARNAME)?;

    // TODO: This command handling is code duplication. We can and should refactor.
    let mut editor_exec = Command::new(editor);
    editor_exec
        .arg(filepath)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit());
    exec_cmd(&format!("${}", EDITOR_ENV_VARNAME), editor_exec, true)?;

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
