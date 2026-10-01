use clap::{CommandFactory, Parser};

pub mod args;
mod commands;
mod output;
mod player;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = args::Cli::parse();

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
