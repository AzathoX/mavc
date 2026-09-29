# MAVC  mutiple audio version collection

**English** | [简体中文](README.zh-CN.md)

Have you ever noticed that a song can have several versions, such as English
and Chinese versions, covers by different singers, or alternate arrangements?
Each version takes several minutes to play, so listening through all of them
can take a while. MAVC packages related versions into one `.mavc` archive and
randomly selects a track when you play it, adding variety to each listen.

MAVC is a Rust music archive library and command-line tool for MP3, FLAC, and
AAC. Each archive contains a description, an MVLC music index, and the original
audio data. Track weights control how often each track is selected during
weighted random playback. MAVC is currently in the refinement stage of version
1.0. This is also a learning and research project, and I’ll explore support
for more players in the future.

## CLI help

Run `mavc --help`, `mavc -h`, or `mavc help` to display this command list.

```text
mavc — weighted music archive tool
Usage:
  mavc help|--help|-h
  mavc version|--version|-V|-v
  mavc package -m <audio-file>... [-a <output.mavc>]
  mavc package <manifest.json> <output.mavc>
  mavc music -m <audio-file>... [-a <output.mavc>]
  mavc music -a <output.mavc> -m <audio-file>...
  mavc create <manifest.json> <output.mavc>
  mavc list <archive.mavc>
  mavc play [-r|--random | -o|--one <track-id>] [-sp|--system-player | -bp|--browser-player] <archive.mavc>
  mavc inspect <archive.mavc> [track-id]
  mavc inspect <track-id> <archive.mavc>
  mavc pick <archive.mavc>
  mavc weight <archive.mavc> <track-id>=<weight>
  mavc extract <archive.mavc> <track-id> [output-file]
```

## Commands

- **`help`, `--help`, `-h`** — Show command usage.
- **`version`, `--version`, `-V`, `-v`** — Show the MAVC version.
- **`package -m <files>... [-a <output.mavc>]`** — Package audio files. Without `-a`, one file produces `<filename>.mavc`; multiple files produce `music.mavc`.
- **`package <manifest.json> <output.mavc>`** — Package tracks described by a JSON manifest.
- **`music -m <files>... [-a <output.mavc>]`** — Shortcut for packaging a music list. The `-m` and `-a` options may appear in either order.
- **`create <manifest.json> <output.mavc>`** — Alias for manifest-based packaging.
- **`list <archive.mavc>`** — List tracks and show their IDs, formats, and weights.
- **`play [-r|--random] <archive.mavc>`** — Choose a track by its weight and play it with MAVC's built-in player. Random playback is the default.
- **`play -o|--one <track-id> <archive.mavc>`** — Play one specified track ID.
- **`play -sp|--system-player <archive.mavc>`** — Extract the selected track temporarily and open it with the system's associated audio app.
- **`play -bp|--browser-player <archive.mavc>`** — Save a temporary audio file and HTML player page in a `mavc-browser-*` folder under the current working directory, then open the page in the default browser. Browser autoplay settings may require clicking Play.
- **`inspect <archive.mavc> [track-id]`** or **`inspect <track-id> <archive.mavc>`** — Show archive information and available track metadata, such as title, artist/singer, album, genre, year, duration, bitrate, sample rate, channels, and bit depth. Missing tags are omitted.
- **`pick <archive.mavc>`** — Print a weighted random track selection without playing it.
- **`weight <archive.mavc> <track-id>=<weight>`** — Update a track's random-selection weight in the archive and report the old and new values. Weight must be finite and non-negative; use `0` to exclude a track from random selection.
- **`extract <archive.mavc> <track-id> [output-file]`** — Extract a track. If no output filename is given, MAVC uses the track's stored filename and extension.

`-r`/`--random` and `-o`/`--one` are mutually exclusive. The browser player also
accepts the legacy misspelling `--broswer-player`. The `inspect` command reads
metadata from older archives when it is not cached in the MVLC index.

## Archive format

Version 1 uses little-endian integers. Its 22-byte header contains the `MAVC`
magic, container version (`u16`), description length (`u32`), MVLC index length
(`u32`), and audio payload length (`u64`). The description and index are JSON
objects, followed by the concatenated audio payloads. Track offsets in the
index are relative to the beginning of the payload section. The container and
MVLC index version are both currently `1`.

## Windows

Install Rust with rustup, then build from PowerShell in the project folder:

```powershell
cargo build --release
.\target\release\mavc.exe --help
.\target\release\mavc.exe package -m .\song.mp3 .\song.flac -a .\music.mavc
.\target\release\mavc.exe play -r .\music.mavc
```

The built-in player uses the default Windows audio device. The `-sp` and `-bp`
options open the audio file or browser page through Windows file associations.

## License

MAVC is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
