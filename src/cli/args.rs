use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "mavc",
    disable_version_flag = true,
    about = "Weighted music archive tool",
    after_help = "Previous CLI syntax:\n  mavc help|--help|-h\n  mavc version|--version|-V|-v\n  mavc package -m <audio-file>... [-a <output.mavc>]\n  mavc package <manifest.json> <output.mavc>\n  mavc music -m <audio-file>... [-a <output.mavc>]\n  mavc music -a <output.mavc> -m <audio-file>...\n  mavc create <manifest.json> <output.mavc>\n  mavc [-+|--add] <audio-file>... --to <archive.mavc>...\n  mavc [-x|--remove] <track-id>... --from <archive.mavc>...\n  mavc list <archive.mavc>\n  mavc play [-r|--random | -o|--one <track-id>] [-sp|--system-player | -bp|--browser-player] <archive.mavc>\n  mavc inspect <archive.mavc> [track-id]\n  mavc inspect <track-id> <archive.mavc>\n  mavc pick <archive.mavc>\n  mavc weight <archive.mavc> <track-id>=<weight>\n  mavc extract <archive.mavc> <track-id> [output-file]"
)]
pub struct Cli {
    /// Print version information.
    #[arg(
        short = 'V',
        long = "version",
        short_alias = 'v',
        alias = "VERSION",
        action = clap::ArgAction::SetTrue
    )]
    pub version: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create an archive from audio files or a JSON manifest.
    Package(PackageArgs),
    /// Create an archive from an audio-file list.
    Music(MusicArgs),
    /// Create an archive from a JSON manifest.
    Create(ManifestArgs),
    /// Add audio files to one or more archives.
    #[command(short_flag = '+', long_flag = "add", hide = true)]
    Add(AddArgs),
    /// Remove tracks from one or more archives.
    #[command(short_flag = 'x', long_flag = "remove", hide = true)]
    Remove(RemoveArgs),
    /// List tracks in an archive.
    List(ArchiveArgs),
    /// Play tracks from an archive.
    Play(PlayArgs),
    /// Show archive or track metadata.
    Inspect(InspectArgs),
    /// Pick a track using its configured weight.
    Pick(ArchiveArgs),
    /// Update a track's selection weight.
    Weight(WeightArgs),
    /// Extract one track or all tracks from an archive.
    Extract(ExtractArgs),
    /// Legacy shortcut: `mavc -m file.mp3`.
    #[command(short_flag = 'm', long_flag = "music-list", hide = true)]
    MusicShortcut(LegacyMusicArgs),
}

#[derive(Debug, Args)]
pub struct PackageArgs {
    /// Audio files to package (music-list mode).
    #[arg(short = 'm', long = "music-list", num_args = 1..)]
    pub files: Vec<PathBuf>,
    /// Output archive path (music-list mode).
    #[arg(short = 'a', long = "as")]
    pub output: Option<PathBuf>,
    /// Manifest path (manifest mode).
    pub manifest: Option<PathBuf>,
    /// Output archive path (manifest mode).
    pub manifest_output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MusicArgs {
    #[arg(short = 'm', long = "music-list", required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(short = 'a', long = "as")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct LegacyMusicArgs {
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    #[arg(short = 'a', long = "as")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ManifestArgs {
    pub manifest: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Audio files to add.
    #[arg(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
    /// Destination archives.
    #[arg(long = "to", required = true, num_args = 1..)]
    pub archives: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Track IDs to remove.
    #[arg(required = true, num_args = 1..)]
    pub ids: Vec<usize>,
    /// Archives to update.
    #[arg(long = "from", required = true, num_args = 1..)]
    pub archives: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ArchiveArgs {
    pub archive: PathBuf,
}

#[derive(Debug, Args)]
pub struct PlayArgs {
    /// Select a weighted random track (the default).
    #[arg(short = 'r', long, conflicts_with = "one")]
    pub random: bool,
    /// Select one track by ID.
    #[arg(short = 'o', long, value_name = "TRACK_ID", conflicts_with = "random")]
    pub one: Option<usize>,
    /// Play every track in archive order.
    #[arg(long, conflicts_with_all = ["random", "one"])]
    pub all: bool,
    /// Mix two or more tracks at the same time.
    #[arg(
        short = 'c',
        long,
        value_name = "TRACK_ID",
        num_args = 2..,
        conflicts_with_all = ["random", "one", "all", "browser_player"]
    )]
    pub combine: Vec<usize>,
    /// Open a temporary browser player page.
    #[arg(
        short = 'b',
        long = "browser-player",
        alias = "broswer-player",
        conflicts_with_all = ["random", "one", "combine"]
    )]
    pub browser_player: bool,
    pub archive: PathBuf,
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    /// Archive path, with an optional track ID.
    pub archive_or_id: String,
    /// Track ID when the archive path comes first.
    pub second: Option<String>,
}

#[derive(Debug, Args)]
pub struct WeightArgs {
    pub archive: PathBuf,
    /// Assignment in the form TRACK_ID=WEIGHT.
    pub assignment: String,
}

#[derive(Debug, Args)]
pub struct ExtractArgs {
    pub archive: PathBuf,
    /// Extract every track into the current directory (`-all` is also accepted).
    #[arg(long, conflicts_with_all = ["track_id", "output"])]
    pub all: bool,
    #[arg(required_unless_present = "all")]
    pub track_id: Option<usize>,
    pub output: Option<PathBuf>,
}
