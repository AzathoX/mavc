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
        } else if argument == "-sp" {
            *argument = "--system-player".into();
        }
    }
    normalize_combine_archive_position(&mut argv);
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

fn normalize_combine_archive_position(argv: &mut Vec<std::ffi::OsString>) {
    let Some(play_index) = argv.iter().position(|argument| argument == "play") else {
        return;
    };
    let Some(combine_index) =
        argv.iter()
            .enumerate()
            .skip(play_index + 1)
            .find_map(|(index, argument)| {
                (argument == "--combine" || argument == "-c").then_some(index)
            })
    else {
        return;
    };

    // Clap's variadic combine values otherwise consume a trailing archive path
    // and then report that it is not a numeric track ID. Move that path before
    // the options; the positional archive argument keeps the same meaning.
    let archive_index =
        argv.iter()
            .enumerate()
            .skip(combine_index + 1)
            .find_map(|(index, argument)| {
                let value = argument.to_str()?;
                let is_option = value.starts_with('-');
                let is_track_id = value.parse::<usize>().is_ok();
                (!is_option && !is_track_id).then_some(index)
            });

    if let Some(archive_index) = archive_index {
        let archive = argv.remove(archive_index);
        argv.insert(play_index + 1, archive);
    }
}
