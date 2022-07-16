use clap::{Parser, Subcommand};

// TODO: We call call debug_assert() in a test as per clap documentation recommendations.
/// Helps you jot notes.
///
/// For arguments that take a command invocation, only the output from stdout is used for
/// execution. An invocation is only considered an error if it returns with a
/// non-zero exit code. There is also no restriction placed on the invocation itself. Invocations
/// can be quite literally anything, from /bin/ls to fzf to a custom Python script.
///
/// Note that invocations are executed by passing the invocation to the user's $SHELL. This means
/// your invocation can actually be written in a shell's scripting language, and make use of things
/// like environment variable substitution (jot passes its environment down to its child
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
/// When invoking $EDITOR, the standard streams stdout and stdin are inherited by the editor
/// process. stderr is piped.
#[derive(Parser, Debug)]
pub struct Cli {
    // NOTE: If you ever update any flag or subcommand's name, please search and replace all
    // instances of the flag name, as we may have references to it in doc strings or error messages
    // that won't be picked up by an LSP rename.
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The base directory under which all notes handled by jot must reside. This must be a git
    /// repository.
    #[clap(short, long, parse(from_os_str))]
    pub base_dir: std::path::PathBuf,

    /// A command invocation that prints a single filepath to stdout upon completion.
    #[clap(short, long, value_parser)]
    pub finder: String,

    /// A command invocation that, given a path (relative to base_dir) as a positional argument,
    /// prints a listing to stdout.
    #[clap(short, long, value_parser)]
    pub lister: String,

    /// Whether or not entering edit mode should incur a sync after finishing. Default: true.
    #[clap(default_value_t = true, short, long, value_parser)]
    pub edit_syncs: bool,

    /// Whether or not stderr should be captured. If not captured, the child process inherits it
    /// from the parent. Note that if this value is false, invocations that print things like error
    /// diagnostics to stderr will not be propagated from jot. Default: false.
    #[clap(default_value_t = false, short, long, value_parser)]
    pub capture_stderr: bool,

    /// Which flag to specify to the user's $SHELL that allows for command execution. e.g. bash uses `-c`.
    #[clap(default_value = "-c", short, long, value_parser)]
    pub shell_cmd_flag: String,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Dispatch to a program that outputs a filepath to open in $EDITOR. Edit mode need not be
    /// explicitly called. Calling jot without any subcommand defaults to edit mode.
    Edit,
    /// Dispatches to a program (e.g. tree) that outputs a listing of all notes.
    List {
        /// An argument for a subtree in the tree from which
        /// to begin the listing.
        #[clap(value_parser)]
        subpath: Option<std::path::PathBuf>,
    },
    /// 'Synchronizes' the notes. This is really just an attempt to git pull, then git push. If an
    /// error (namely a merge conflict) occurs, an error is propagated to stderr.
    #[clap(name = "sync")]
    Synch,
}
