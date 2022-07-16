use std::{
    borrow::Cow,
    env::var,
    path::Path,
    process::{Command, Stdio},
    time::SystemTime,
};

use anyhow::{bail, Context, Result};
use humantime::format_rfc3339_seconds;

use crate::cli;

static SHELL_ENV_VARNAME: &str = "SHELL";

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
    let invocation = format!("{} {}", program.to_string_lossy(), joined_args_str);
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

        bail!(
            "{} (`{}`) exited unsuccessfully with non-zero exit code ({})\n\
            \tstdout:\n\
            \t\"{}\"\n\
            \tstderr:\n\
            \t\"{}\"",
            label,
            invocation,
            exit_code.map_or("N/A".to_string(), |code| code.to_string()),
            stdout_output,
            stderr_output,
        );
    }

    Ok((trimmed_stdout, exit_code))
}

fn open_editor_at_path(filepath: &std::path::Path, args: &cli::Args) -> Result<()> {
    static EDITOR_ENV_VARNAME: &str = "EDITOR";
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

    sync(args)
}

fn relative_path_to_absolute(
    args: &cli::Args,
    filepath: &std::path::PathBuf,
) -> Result<std::path::PathBuf> {
    let mut absolute_filepath = filepath.to_owned();
    if !filepath.is_absolute() {
        absolute_filepath = args.base_dir.join(absolute_filepath);
    } else {
        // If the path is absolute, let's check that it leads to something underneath base_dir.
        // Otherwise, we're creating files outside of our turf, and that is not going to fly (even
        // though the user told us to do it).
        if !absolute_filepath.starts_with(&args.base_dir) {
            bail!(
                "given path must be below base_dir; {} is not",
                absolute_filepath.display()
            )
        }
    }

    Ok(absolute_filepath)
}

pub fn new(args: &cli::Args, filepath: &std::path::PathBuf) -> Result<()> {
    let absolute_filepath = relative_path_to_absolute(args, filepath)?;

    // First, create the given file:
    if !absolute_filepath.exists() {
        std::fs::File::create(absolute_filepath)
            .context(format!("failed to create a file at {}", filepath.display()))?;
    }

    // Then, open it in $EDITOR:
    open_editor_at_path(filepath, args)?;

    Ok(())
}

fn exec_custom_invocation_cmd(mut cmd: Command, args: &cli::Args) -> Result<(String, bool)> {
    if !args.capture_std {
        // Allow stderr/stdin to pass through for applications like fzf.
        cmd.stdin(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let (finder_stdout, exit_code) =
        exec_cmd("finder", cmd, args.capture_std, args.quiet_on_ctrl_c)?;

    // If asked to be quiet on CTRL+C, then exec_cmd() will not have returned error. However, if
    // so, we don't want to make use of whatever stdout may have returned, since the finder program
    // was terminated prematurely (presumably). If so, return true as our boolean half of the
    // tuple, to indicate an early return from the caller.
    Ok((
        finder_stdout,
        args.quiet_on_ctrl_c && exit_code == Some(CTRL_C_EXIT_CODE),
    ))
}

pub fn edit(args: &cli::Args) -> Result<()> {
    // First, we should execute the finder invocation and get a chosen filepath.
    let shell = get_env_var(SHELL_ENV_VARNAME)?;
    let mut finder_cmd = Command::new(shell);
    finder_cmd.arg(&args.shell_cmd_flag).arg(&args.finder);

    if !args.capture_std {
        // Allow stderr/stdin to pass through for applications like fzf.
        finder_cmd.stdin(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let (finder_stdout, should_exit_early) = exec_custom_invocation_cmd(finder_cmd, args)?;
    if should_exit_early {
        return Ok(());
    }

    let filepath = Path::new(&finder_stdout);

    // Then, open the editor at that path.
    open_editor_at_path(filepath, args)?;

    Ok(())
}

pub fn list(args: &cli::Args, subpath: Option<std::path::PathBuf>) -> Result<()> {
    // First, change working directory into the given list_path.
    // Note that this could possibly be a no-op if none was specified.
    let listing_path = subpath.map_or(Ok(args.base_dir.clone()), |path| {
        relative_path_to_absolute(args, &path)
    })?;
    std::env::set_current_dir(&listing_path).context(format!(
        "failed to change jot's working directory to {} for listing",
        listing_path.display(),
    ))?;

    let shell = get_env_var(SHELL_ENV_VARNAME)?;
    let mut lister_cmd = Command::new(shell);
    lister_cmd.arg(&args.shell_cmd_flag).arg(&args.lister);

    if !args.capture_std {
        // Allow stderr/stdin to pass through for applications like fzf.
        lister_cmd.stdin(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let (lister_stdout, should_exit_early) = exec_custom_invocation_cmd(lister_cmd, args)?;
    if should_exit_early {
        return Ok(());
    }

    println!("{}", lister_stdout);

    // Before we can return, we need to reset the current working directory. Technically, since jot
    // is only ran for a single command at a time, this is actually not necessary, so really, we're
    // just being polite. I don't think there really is a reason to care, it just bothers me.
    std::env::set_current_dir(&args.base_dir).context(format!(
        "failed to change jot's working directory to {} for listing",
        args.base_dir.display(),
    ))?;

    Ok(())
}

pub fn sync(args: &cli::Args) -> Result<()> {
    static GIT_CMD: &str = "git";

    // First, git pull to fetch and merge upstream changes.
    // If we encounter an issue, namely a merge conflict, this will propagate an error and we will
    // abort on trying to merge our recent changes.
    let mut git_pull_exec = Command::new(GIT_CMD);
    git_pull_exec
        .arg("pull")
        .arg(&args.git_remote_name)
        .arg(&args.git_upstream_branch);
    exec_cmd("pulling", git_pull_exec, true, args.quiet_on_ctrl_c)
        .context("failed to pull upstream changes, please fix the issue and run jot sync")?;

    // Second, if we get here, git pull worked. In that case, let's stage our local changes:
    let mut git_pull_exec = Command::new(GIT_CMD);
    git_pull_exec.arg("add").arg("-A");
    exec_cmd("staging", git_pull_exec, true, args.quiet_on_ctrl_c)?;

    // Third, commit these staged changes:
    let mut git_commit_exec = Command::new(GIT_CMD);
    git_commit_exec.arg("commit");
    if args.git_custom_commit_msg {
        git_commit_exec
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit());
    } else {
        git_commit_exec
            .arg("-m")
            .arg(format!("{}", format_rfc3339_seconds(SystemTime::now())));
    }
    exec_cmd("committing", git_commit_exec, true, args.quiet_on_ctrl_c)?;

    // Fourth, push to upstream to finish the sync.
    let mut git_push_exec = Command::new(GIT_CMD);
    git_push_exec
        .arg("push")
        .arg(&args.git_remote_name)
        .arg(&args.git_upstream_branch);
    exec_cmd("pushing", git_push_exec, true, args.quiet_on_ctrl_c)
        .context("failed to push to upstream, please fix the issue and run jot sync")?;
    Ok(())
}
