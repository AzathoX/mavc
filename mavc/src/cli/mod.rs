use clap::FromArgMatches;

pub mod args;
mod commands;
mod locale;
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
    normalize_add_file_position(&mut argv);
    normalize_combine_archive_position(&mut argv);
    let mut command = locale::command(locale::detect());
    let matches = command
        .try_get_matches_from_mut(argv)
        .unwrap_or_else(|error| error.exit());
    let cli = args::Cli::from_arg_matches(&matches)?;

    if cli.version {
        println!("mavc {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match cli.command {
        Some(command) => commands::run(command),
        None => {
            command.print_help()?;
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

// Clap's variadic --to otherwise consumes trailing positional audio files.
// In destination-first form, the first supported audio suffix starts FILES.
// An explicit `--` remains available to disambiguate unusual archive names.
fn normalize_add_file_position(argv: &mut Vec<std::ffi::OsString>) {
    let action_index = if argv.get(1).is_some_and(|arg| arg == "do") {
        2
    } else {
        1
    };
    if !argv
        .get(action_index)
        .is_some_and(|arg| arg == "add" || arg == "--add" || arg == "-+")
    {
        return;
    }
    // The caller already supplied an unambiguous positional boundary.
    if argv.iter().skip(action_index + 1).any(|arg| arg == "--") {
        return;
    }
    let first_value = action_index + 1;
    let is_destination = |arg: &std::ffi::OsStr| {
        arg == "--to" || arg.to_str().is_some_and(|value| value.starts_with("--to="))
    };
    if !argv.get(first_value).is_some_and(|arg| is_destination(arg)) {
        return;
    }
    let mut has_archive = false;
    let mut reading_files = false;
    let mut files = Vec::new();
    let mut index = first_value;
    while index < argv.len() {
        let argument = &argv[index];
        if argument == "--" {
            break;
        }
        if is_destination(argument) {
            has_archive = argument
                .to_str()
                .is_some_and(|value| value.starts_with("--to=") && value.len() > 5);
            reading_files = false;
        } else if argument
            .to_str()
            .is_some_and(|value| value.starts_with('-'))
        {
            break;
        } else {
            let is_audio = std::path::Path::new(argument)
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    ["mp3", "flac", "aac", "wav", "ogg"]
                        .iter()
                        .any(|audio| extension.eq_ignore_ascii_case(audio))
                });
            if reading_files || (has_archive && is_audio) {
                reading_files = true;
                files.push(argv.remove(index));
                continue;
            }
            has_archive = true;
        }
        index += 1;
    }
    argv.splice(first_value..first_value, files);
}
