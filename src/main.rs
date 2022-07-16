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

static CTRL_C_EXIT_CODE: i32 = 130;

fn get_env_var(varname: &str) -> Result<String> {
    var(varname).context(format!("failed to find ${} in environment", varname))
}

fn exec_cmd(
    label: &str,
    mut cmd: Command,
    captured_stderr: bool,
    quiet_on_ctrl_c: bool,
) -> Result<(String, Option<i32>)> {
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

    let trimmed_stdout = stdout_output.trim().to_string();

    let exit_code = exec.status.code();
    if !exec.status.success() {
        if quiet_on_ctrl_c && exit_code == Some(CTRL_C_EXIT_CODE) {
            return Ok((trimmed_stdout, exit_code));
        }

        // TODO: This format string is getting gnarly as fuck... we need to break it up somehow...
        return Err(anyhow!(
            "{} (`{}`) exited unsuccessfully with non-zero exit code ({})\n\tstdout:\n\t\"{}\"\n\tstderr:\n\t\"{}\"",
            label,
            invocation,
            exit_code.map_or("N/A".to_string(), |code| code.to_string()),
            stdout_output,
            stderr_output,
        ));
    }

    Ok((trimmed_stdout, exit_code))
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

    let (finder_stdout, exit_code) = exec_cmd(
        "finder",
        finder_cmd,
        args.capture_stderr,
        args.quiet_on_ctrl_c,
    )?;

    let filepath = Path::new(&finder_stdout);
    if args.quiet_on_ctrl_c && exit_code == Some(CTRL_C_EXIT_CODE) {
        // If asked to be quiet on CTRL+C, then exec_cmd() will not have returned error. However,
        // if so, we don't want to make use of whatever stdout may have returned, since the finder
        // program was terminated prematurely (presumably). So, instead, just return early and
        // don't mess with $EDITOR.
        return Ok(());
    }

    let editor = get_env_var(EDITOR_ENV_VARNAME)?;

    let mut editor_exec = Command::new(editor);
    editor_exec
        .arg(filepath)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit());
    exec_cmd(
        &format!("${}", EDITOR_ENV_VARNAME),
        editor_exec,
        true,
        args.quiet_on_ctrl_c,
    )?;

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
