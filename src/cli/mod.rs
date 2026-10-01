use clap::{CommandFactory, Parser};

pub mod args;
mod commands;
mod output;
mod player;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Preserve the requested single-hyphen spelling while using clap's
    // canonical `--all` option internally.
    let mut argv = std::env::args_os().collect::<Vec<_>>();
    for argument in &mut argv {
        if argument == "-all" {
            *argument = "--all".into();
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
