use clap::{Parser, Subcommand};

/// Write notes.
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
///     * Set the capture_stderr flag.
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
pub struct Cli {
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

    /// Capture stderr. If not captured, the child process inherits stderr from the parent. Note
    /// that if this value is false, invocations that print things like error diagnostics to stderr
    /// will not be propagated directly by jot. Default: false.
    #[clap(default_value_t = false, short, long, value_parser)]
    pub capture_stderr: bool,

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
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Creates a new note at the specified path and opens it in $EDITOR. If a file exists at the
    /// path already, this command behaves similarly to Edit if its dispatched program had returned
    /// the given path.
    New {
        /// The path at which to create the new note. This path may be absolute, or, if relative,
        /// must be relative to base-dir. This path, regardless of absoluteness, must reside
        /// beneath base_dir.
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
        /// if omitted, prints the contents of base-dir from its root.
        #[clap(value_parser)]
        subpath: Option<std::path::PathBuf>,
    },
    /// 'Synchronize' the notes. This is really just an attempt to git pull, git add -A, then git
    /// push. If an error (namely a merge conflict) occurs, an error is propagated to stderr.
    #[clap(name = "sync")]
    Synch,
}

// Proactively check for bad configurations.
// https://github.com/clap-rs/clap/blob/v3.2.12/examples/tutorial_derive/05_01_assert.rs
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
