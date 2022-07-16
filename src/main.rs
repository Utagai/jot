use clap::Parser;

mod cli;

fn main() {
    let args = cli::Cli::parse();
    println!("{:?}", args);
    match args.command.unwrap_or(cli::Command::Edit) {
        cli::Command::Edit => edit(),
        cli::Command::List { .. } => list(),
        cli::Command::Synch => synch(),
    }
}

fn edit() {
    println!("Edit mode!")
}

fn list() {
    println!("List mode!");
}

fn synch() {
    println!("Sync mode!");
}
