use clap::{Parser, Subcommand};

/// Helps you jot notes.
///
/// For arguments that take a command invocation, only the output from stdout is used for
/// execution. Output from stderr is essentially only redirected to the terminal from jot, but is
/// not used for any execution. An invocation is only considered an error if it returns with a
/// non-zero exit code. There is also no restriction placed on the invocation itself. Invocations
/// can be quite literally anything, from /bin/ls to fzf to a custom Python script.
#[derive(Parser, Debug)]
pub struct Cli {
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

    /// Whether or not entering edit mode should incur a sync after finishing.
    #[clap(default_value_t = true, short, long, value_parser)]
    pub edit_syncs: bool,
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
    // that's why. Maybe rename to Synch but keep the flag name Sync?
    /// 'Synchronizes' the notes. This is really just an attempt to git pull, then git push. If an
    /// error (namely a merge conflict) occurs, an error is propagated to stderr.
    #[clap(name = "sync")]
    Synch,
}