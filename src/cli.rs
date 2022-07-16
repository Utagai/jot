use clap::{Parser, Subcommand};

/// Write notes.
///
///
///
/// Jot has no configuration file. It only has CLI flags and such. Jot commands and
/// command-specific arguments come at the end of its usage, so jot is meant to be aliased.
///
/// Jot is based on top of git. The base-dir containining all the notes is just a git repository.
/// This also means that you are able to go into that repository and mess with it as you see fit.
/// This can make jot fail, so mess with it at your own risk. In fact, jot is remarkably stupid.
/// All it does to sync your notes is to pull from upstream, add any changes in the repository,
/// commit them, and then push back to upstream. It fails pretty much immediately the moment
/// anything goes wrong.
///
/// For arguments that take a command invocation, only the output from stdout is used for
/// execution. An invocation is only considered an error if it returns with a
/// non-zero exit code. There is also no restriction placed on the invocation itself. Invocations
/// can be quite literally anything, from /bin/ls to fzf to a custom Python script.
///
/// Note that custom invocations are executed by passing the invocation to the user's $SHELL. This
/// means your invocation can actually be written in a shell's scripting language, and make use of
/// things like environment variable substitution (jot passes its environment down to its child
/// processes). Note that there may be differences in how different shells support command
/// execution, for example, in bash, one uses `-c`:
///
/// bash -c 'echo foo'
///
/// If your shell differs, please set the shell_cmd_flag flag.
///
/// Standard streams stdin & stderr are inherited by the the child process. This is done to support
/// applications like fzf, which need stdin and stderr for their UI. This means that if your
/// application uses stderr to log error information, you should either:
///
///     * Set the capture_std flag.
///
///     * Use stdout and return a non-zero exit code.
///
///     * Roll out your own logging.
///
/// Finally, note that custom invocations _do not include_ invoking $EDITOR or git.
/// When invoking $EDITOR, the standard streams stdout and stdin are inherited by the editor
/// process, but stderr is piped.
/// When invoking git, all standard streams are inherited.
#[derive(Parser, Debug)]
pub struct Args {
    // NOTE: If you ever update any flag or subcommand's name, please search and replace all
    // instances of the flag name, as we may have references to it in doc strings or error messages
    // that won't be picked up by an LSP rename.
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// Base directory under which all notes handled by jot must reside. This must be a git
    /// repository.
    #[clap(short, long, parse(from_os_str))]
    pub base_dir: std::path::PathBuf,

    /// Specifies a command invocation that prints a single filepath to stdout upon completion.
    #[clap(short, long, value_parser)]
    pub finder: String,

    /// Specifies a command invocation that, given a path (relative to base-dir) as a positional
    /// argument, prints a listing to stdout.
    #[clap(short, long, value_parser)]
    pub lister: String,

    /// Editing should finish with a sync automatically. Default: true.
    #[clap(default_value_t = true, short, long, value_parser)]
    pub edit_syncs: bool,

    /// Capture stderr/stdin for custom invocations. If not captured, the child process inherits
    /// stderr from the parent. Note that if this value is false, invocations that print things
    /// like error diagnostics to stderr will not be propagated directly by jot. Default: false.
    #[clap(default_value_t = false, short, long, value_parser)]
    pub capture_std: bool,

    /// Specifies the flag for the user's $SHELL that allows for command execution. e.g. bash uses `-c`.
    #[clap(default_value = "-c", short, long, value_parser)]
    pub shell_cmd_flag: String,

    /// Do not print any error information if an invocation fails due to exit code 130 (CTRL+C).
    /// Likely only valid on unix/*nix-like OSes. Default: true.
    #[clap(default_value_t = true, short, long, value_parser)]
    pub quiet_on_ctrl_c: bool,

    /// Specifies the name of the remote to push/pull to/from.
    #[clap(default_value = "origin", short = 'r', long, value_parser)]
    pub git_remote_name: String,

    /// Specifies the name of the remote branch to push/pull to/from.
    #[clap(default_value = "main", short = 'u', long, value_parser)]
    pub git_upstream_branch: String,

    /// Prompt for a custom git commit message when syncing. This will default to whatever behavior
    /// your git config suggests for a bare `git commit`.
    #[clap(default_value_t = false, short = 'm', long, value_parser)]
    pub git_custom_commit_msg: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Creates a new note at the specified path and opens it in $EDITOR. If a file exists at the
    /// path already, this command behaves similarly to Edit if its dispatched program had returned
    /// the given path.
    New {
        /// The path at which to create the new note. This path may be absolute, or, if relative,
        /// must be relative to base-dir. This path, regardless of absoluteness, must reside
        /// beneath base-dir.
        #[clap(value_parser)]
        path: std::path::PathBuf,
    },
    /// Dispatch to a program that outputs a filepath to open in $EDITOR. Edit mode need not be
    /// explicitly called. Calling jot without any subcommand defaults to edit mode. Note that the
    /// finder program need not return a filepath that exists. If the filepath does not exist,
    /// $EDITOR will be called nevertheless on the path. Most editors will open a blank page, and
    /// then create the file on save. This makes Edit roughly equivalent to New, the primary
    /// difference being that New creates the file prior to opening it in $EDITOR.
    Edit,
    /// Dispatch to a program (e.g. tree) that outputs a listing of all notes.
    List {
        /// The path representing the subtree from which to begin the listing. This is optional and
        /// if omitted, runs the invocation from base-dir. This path may be absolute, or, if relative,
        /// must be relative to base-dir. This path, regardless of absoluteness, must reside
        /// beneath base-dir. Note that this is effectively setting the working directory for the
        /// invocation, it does not get passed to the invocation.
        #[clap(value_parser)]
        subpath: Option<std::path::PathBuf>,
    },
    /// 'Synchronize' the notes. This is really just an attempt to git pull, git add -A, git
    /// commit, then finally, git push. If an error (namely a merge conflict) occurs, an error is
    /// propagated to stderr. If you want to be prompted for a custom commit message, specify the
    /// git-custom-commit-msg flag, otherwise, jot will set the message to the current local system
    /// time in RFC3339 format.
    #[clap(name = "sync")]
    Synch,
}

// Proactively check for bad configurations.
// https://github.com/clap-rs/clap/blob/v3.2.12/examples/tutorial_derive/05_01_assert.rs
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert();
}
