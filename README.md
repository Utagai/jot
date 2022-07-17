# `jot`
WIP. Minimalism + scribbling.

The entirety of `jot`'s documentation can be found in its `help` output:
```
jot
Write notes.

Jot is a _very thin_ wrapper around the local filesystem and git. It makes use of these things to
store, as well as track and distribute notes. Jot is really just a thin API around the two and does
not do anything all that special.

Jot has no configuration file. It only has CLI flags and such. Jot commands and command-specific
arguments come at the end of its usage, so jot is meant to be aliased.

Jot is based on top of git. The base-dir containining all the notes is just a git repository. This
also means that you are able to go into that repository and mess with it as you see fit. This can
make jot fail, so mess with it at your own risk. In fact, jot is remarkably stupid. All it does to
sync your notes is to pull from upstream, add any changes in the repository, commit them, and then
push back to upstream. It fails pretty much immediately the moment anything goes wrong.

For arguments that take a command invocation, only the output from stdout is used for execution. An
invocation is only considered an error if it returns with a non-zero exit code. There is also no
restriction placed on the invocation itself. Invocations can be quite literally anything, from
/bin/ls to fzf to a custom Python script.

Note that custom invocations are executed by passing the invocation to the user's $SHELL. This means
your invocation can actually be written in a shell's scripting language, and make use of things like
environment variable substitution (jot passes its environment down to its child processes). Note
that there may be differences in how different shells support command execution, for example, in
bash, one uses `-c`:

bash -c 'echo foo'

If your shell differs, please set the shell_cmd_flag flag.

Standard streams stdin & stderr are inherited by the the child process. This is done to support
applications like fzf, which need stdin and stderr for their UI. This means that if your application
uses stderr to log error information, you should either:

* Set the capture_std flag.

* Use stdout and return a non-zero exit code.

* Roll out your own logging.

Finally, note that custom invocations _do not include_ invoking $EDITOR or git. When invoking
$EDITOR, the standard streams stdout and stdin are inherited by the editor process, but stderr is
piped. When invoking git, all standard streams are inherited.

USAGE:
    jot [OPTIONS] --base-dir <BASE_DIR> --finder <FINDER> --lister <LISTER> [SUBCOMMAND]

OPTIONS:
    -b, --base-dir <BASE_DIR>
            Base directory under which all notes handled by jot must reside. This must be a git
            repository

    -c, --capture-std
            Capture stderr/stdin for custom invocations. If not captured, the child process inherits
            stderr from the parent. Note that if this value is false, invocations that print things
            like error diagnostics to stderr will not be propagated directly by jot. Default: false

    -e, --edit-syncs
            Editing should finish with a sync automatically. Default: true

    -f, --finder <FINDER>
            Specifies a command invocation that prints a single filepath to stdout upon completion

    -h, --help
            Print help information

    -l, --lister <LISTER>
            Specifies a command invocation that, given a path (relative to base-dir) as a positional
            argument, prints a listing to stdout

    -m, --git-custom-commit-msg
            Prompt for a custom git commit message when syncing. This will default to whatever
            behavior your git config suggests for a bare `git commit`

    -q, --quiet-on-ctrl-c
            Do not print any error information if an invocation fails due to exit code 130 (CTRL+C).
            Likely only valid on unix/*nix-like OSes. Default: true

    -r, --git-remote-name <GIT_REMOTE_NAME>
            Specifies the name of the remote to push/pull to/from

            [default: origin]

    -s, --shell-cmd-flag <SHELL_CMD_FLAG>
            Specifies the flag for the user's $SHELL that allows for command execution. e.g. bash
            uses `-c`

            [default: -c]

    -u, --git-upstream-branch <GIT_UPSTREAM_BRANCH>
            Specifies the name of the remote branch to push/pull to/from

            [default: main]

SUBCOMMANDS:
    edit
            Dispatch to a program that outputs a filepath to open in $EDITOR. Edit mode need not be
            explicitly called. Calling jot without any subcommand defaults to edit mode. Note that
            the finder program need not return a filepath that exists. If the filepath does not
            exist, $EDITOR will be called nevertheless on the path. Most editors will open a blank
            page, and then create the file on save. This makes Edit roughly equivalent to New, the
            primary difference being that New creates the file prior to opening it in $EDITOR
    help
            Print this message or the help of the given subcommand(s)
    list
            Dispatch to a program (e.g. tree) that outputs a listing of all notes
    new
            Creates a new note at the specified path and opens it in $EDITOR. If a file exists at
            the path already, this command behaves similarly to Edit if its dispatched program had
            returned the given path
    sync
            'Synchronize' the notes. This is really just an attempt to git pull, git add -A, git
            commit, then finally, git push. If an error (namely a merge conflict) occurs, an error
            is propagated to stderr. If you want to be prompted for a custom commit message, specify
            the git-custom-commit-msg flag, otherwise, jot will set the message to the current local
            system time in RFC3339 format
```

## Dependencies
Just a few.

* `rustc` to compile `jot`
* `git`

## p.s.
This README was unfortunately not written with `jot`.
