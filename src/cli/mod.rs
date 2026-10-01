use clap::{CommandFactory, Parser};

pub mod args;
mod commands;
mod output;
mod player;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Preserve single-hyphen command spellings while using clap's canonical
    // long options internally.
    let mut argv = std::env::args_os().collect::<Vec<_>>();
    for argument in &mut argv {
        if argument == "-all" {
            *argument = "--all".into();
        } else if argument == "-bp" {
            *argument = "--browser-player".into();
        }
    }
    let cli = args::Cli::parse_from(argv);

    if cli.version {
        println!("mavc {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match cli.command {
        Some(command) => commands::run(command),
        None => {
            args::Cli::command().print_help()?;
            Ok(())
        }
    }
}
