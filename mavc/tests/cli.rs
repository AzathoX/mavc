use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const MP3: &[u8] = include_bytes!("fixtures/silence.mp3");
const FLAC: &[u8] = include_bytes!("fixtures/silence.flac");
const AAC: &[u8] = include_bytes!("fixtures/silence.aac");
const WAV: &[u8] = include_bytes!("fixtures/silence.wav");
const OGG: &[u8] = include_bytes!("fixtures/silence.ogg");

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "mavc-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("song.mp3"), MP3).unwrap();
        std::fs::write(path.join("manifest.json"), r#"{"describe":{"title":"test"},"tracks":[{"path":"song.mp3","title":"song","weight":1.0}]}"#).unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_mavc"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
    fn ok(&self, args: &[&str]) -> Output {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
    fn archive(&self) {
        self.ok(&["music", "-m", "song.mp3", "-a", "test.mavc"]);
    }
    fn open(&self) -> mavc::Archive {
        mavc::Archive::open(self.0.join("test.mavc")).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn help() {
    Fixture::new().ok(&["help"]);
}
#[test]
fn version() {
    assert!(
        String::from_utf8_lossy(&Fixture::new().ok(&["--version"]).stdout)
            .contains(env!("CARGO_PKG_VERSION"))
    );
}
#[test]
fn package_music() {
    let f = Fixture::new();
    f.ok(&["package", "-m", "song.mp3", "-a", "test.mavc"]);
    assert_eq!(f.open().index.tracks.len(), 1);
}
#[test]
fn package_manifest() {
    let f = Fixture::new();
    f.ok(&["package", "manifest.json", "test.mavc"]);
    assert_eq!(f.open().describe.title, "test");
}
#[test]
fn music() {
    let f = Fixture::new();
    f.archive();
    assert_eq!(f.open().index.tracks.len(), 1);
}
#[test]
fn music_shortcut() {
    let f = Fixture::new();
    f.ok(&["-m", "song.mp3", "-a", "test.mavc"]);
    assert_eq!(f.open().index.tracks.len(), 1);
}
#[test]
fn create() {
    let f = Fixture::new();
    f.ok(&["create", "manifest.json", "test.mavc"]);
    assert_eq!(f.open().index.tracks.len(), 1);
}
#[test]
fn do_add() {
    for flag in ["-+", "--add", "add"] {
        for dest in ["--to"] {
            let f = Fixture::new();
            f.archive();
            f.ok(&["do", flag, "song.mp3", dest, "test.mavc", dest, "test.mavc"]);
            assert_eq!(f.open().index.tracks.len(), 3);
        }
    }
}
#[test]
fn do_remove() {
    for flag in ["-x", "--remove", "remove"] {
        for dest in ["--from"] {
            let f = Fixture::new();
            f.archive();
            f.ok(&["do", flag, "0", dest, "test.mavc"]);
            assert!(f.open().index.tracks.is_empty());
        }
    }
}
#[test]
fn legacy_add_remove() {
    let f = Fixture::new();
    f.archive();
    f.ok(&["--add", "song.mp3", "--to", "test.mavc"]);
    f.ok(&["--remove", "0", "--from", "test.mavc"]);
    assert_eq!(f.open().index.tracks[0].id, 1);
}
#[test]
fn list() {
    let f = Fixture::new();
    f.archive();
    assert!(String::from_utf8_lossy(&f.ok(&["list", "test.mavc"]).stdout).contains("song"));
}
#[test]
fn inspect() {
    let f = Fixture::new();
    f.archive();
    for args in [
        vec!["inspect", "test.mavc"],
        vec!["inspect", "test.mavc", "0"],
        vec!["inspect", "0", "test.mavc"],
    ] {
        f.ok(&args);
    }
}
#[test]
fn pick() {
    let f = Fixture::new();
    f.archive();
    assert!(String::from_utf8_lossy(&f.ok(&["pick", "test.mavc"]).stdout).starts_with("0\t"));
}
#[test]
fn weight() {
    let f = Fixture::new();
    f.archive();
    f.ok(&["weight", "test.mavc", "0=0.25"]);
    assert_eq!(f.open().index.tracks[0].weight, 0.25);
}
#[test]
fn extract() {
    let f = Fixture::new();
    f.archive();
    f.ok(&["extract", "test.mavc", "0", "out.mp3"]);
    assert_eq!(std::fs::read(f.0.join("out.mp3")).unwrap(), MP3);
}
#[test]
fn extract_all() {
    let f = Fixture::new();
    f.archive();
    std::fs::remove_file(f.0.join("song.mp3")).unwrap();
    f.ok(&["extract", "test.mavc", "-all"]);
    assert_eq!(std::fs::read(f.0.join("song.mp3")).unwrap(), MP3);
}
#[test]
fn play_invalid_track() {
    let f = Fixture::new();
    f.archive();
    let output = f.run(&["play", "-o", "99", "test.mavc"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no track with id 99"));
}
#[test]
fn do_requires_action_and_destination() {
    let f = Fixture::new();
    for args in [
        vec!["do"],
        vec!["do", "--add", "song.mp3"],
        vec!["do", "--remove", "0"],
        vec!["do", "--remove", "song.mp3", "--from", "test.mavc"],
    ] {
        assert!(!f.run(&args).status.success());
    }
}
#[test]
fn no_track_limit() {
    let f = Fixture::new();
    f.archive();
    for _ in 0..6 {
        f.ok(&["do", "-+", "song.mp3", "--to", "test.mavc"]);
    }
    assert_eq!(f.open().index.tracks.len(), 7);
}
#[test]
fn no_payload_limit() {
    let f = Fixture::new();
    f.archive();
    // A valid tiny FLAC with large, sparse PADDING metadata blocks. Decoding
    // still checks real audio; the archive payload genuinely exceeds 150 MB.
    use std::io::{Seek, SeekFrom, Write};
    let source = f.0.join("large.flac");
    let mut file = std::fs::File::create(&source).unwrap();
    let mut prefix = FLAC[..42].to_vec();
    prefix[4] = 0; // STREAMINFO is followed by padding blocks.
    file.write_all(&prefix).unwrap();
    for index in 0..10 {
        let padding = 15_000_000u32;
        file.write_all(&[
            if index == 9 { 0x81 } else { 1 },
            (padding >> 16) as u8,
            (padding >> 8) as u8,
            padding as u8,
        ])
        .unwrap();
        file.seek(SeekFrom::Current(i64::from(padding))).unwrap();
    }
    file.write_all(&FLAC[42..]).unwrap();
    drop(file);
    let length = std::fs::metadata(&source).unwrap().len();
    assert!(length > 150_000_000);
    f.ok(&["do", "--add", "large.flac", "--to", "test.mavc"]);
    assert_eq!(f.open().index.tracks[1].length, length);
    f.ok(&["do", "--remove", "0", "--from", "test.mavc"]);
    assert_eq!(f.open().index.tracks.len(), 1);
}
#[test]
fn do_multiple_archives() {
    let f = Fixture::new();
    f.archive();
    std::fs::copy(f.0.join("test.mavc"), f.0.join("other.mavc")).unwrap();
    f.ok(&["do", "-+", "song.mp3", "--to", "test.mavc", "other.mavc"]);
    f.ok(&[
        "do",
        "-x",
        "0",
        "1",
        "--from",
        "test.mavc",
        "--from",
        "other.mavc",
    ]);
    for name in ["test.mavc", "other.mavc"] {
        assert!(
            mavc::Archive::open(f.0.join(Path::new(name)))
                .unwrap()
                .index
                .tracks
                .is_empty()
        );
    }
}

#[test]
fn remove_exports_before_deleting() {
    for (source, destination) in [("--from", "--to")] {
        let f = Fixture::new();
        f.archive();
        f.ok(&[
            "do",
            "-x",
            "0",
            source,
            "test.mavc",
            destination,
            "export/nested",
        ]);
        assert!(f.open().index.tracks.is_empty());
        assert_eq!(
            std::fs::read(f.0.join("export/nested/song.mp3")).unwrap(),
            MP3
        );
    }
}
#[test]
fn remove_exports_to_multiple_directories() {
    let f = Fixture::new();
    f.archive();
    f.ok(&[
        "do",
        "--remove",
        "0",
        "0",
        "--from",
        "test.mavc",
        "--to",
        "one",
        "two",
        "--to",
        "three",
    ]);
    for name in ["one", "two", "three"] {
        assert_eq!(std::fs::read(f.0.join(name).join("song.mp3")).unwrap(), MP3);
    }
    assert!(f.open().index.tracks.is_empty());
}
#[test]
fn remove_export_failure_preserves_archive() {
    let f = Fixture::new();
    f.archive();
    let before = std::fs::read(f.0.join("test.mavc")).unwrap();
    // Existing output, invalid directory, and missing track must all preserve the archive.
    for args in [
        vec!["do", "-x", "0", "--from", "test.mavc", "--to", "."],
        vec!["do", "-x", "0", "--from", "test.mavc", "--to", "song.mp3"],
        vec![
            "do",
            "-x",
            "0",
            "99",
            "--from",
            "test.mavc",
            "--to",
            "export",
        ],
    ] {
        assert!(!f.run(&args).status.success());
        assert_eq!(std::fs::read(f.0.join("test.mavc")).unwrap(), before);
    }
    assert!(!f.0.join("export/song.mp3").exists());
    assert_eq!(std::fs::read(f.0.join("song.mp3")).unwrap(), MP3);
}
#[test]
fn remove_duplicate_export_names_preserves_archives() {
    let f = Fixture::new();
    f.archive();
    std::fs::copy(f.0.join("test.mavc"), f.0.join("other.mavc")).unwrap();
    assert!(
        !f.run(&[
            "do",
            "-x",
            "0",
            "--from",
            "test.mavc",
            "other.mavc",
            "--to",
            "export"
        ])
        .status
        .success()
    );
    for name in ["test.mavc", "other.mavc"] {
        assert_eq!(
            mavc::Archive::open(f.0.join(name))
                .unwrap()
                .index
                .tracks
                .len(),
            1
        );
    }
    assert!(!f.0.join("export/song.mp3").exists());
}

fn localized_help(f: &Fixture, args: &[&str], locale: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_mavc"))
        .current_dir(&f.0)
        .env_remove("LANGUAGE")
        .env_remove("LC_MESSAGES")
        .env_remove("LANG")
        .env("LC_ALL", locale)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn help_language_covers_all_commands() {
    let f = Fixture::new();
    for (locale, usage, options) in [
        ("en_US.UTF-8", "Usage:", "Options:"),
        ("zh_TW.UTF-8", "用法：", "選項："),
        ("zh_CN.UTF-8", "用法：", "选项："),
        ("ja_JP.UTF-8", "使い方：", "オプション："),
        ("es_ES.UTF-8", "Uso:", "Opciones:"),
        ("de_DE.UTF-8", "Aufruf:", "Optionen:"),
    ] {
        for command in [
            "package", "music", "create", "do", "list", "play", "inspect", "pick", "weight",
            "extract",
        ] {
            let text = localized_help(&f, &[command, "--help"], locale);
            assert!(text.contains(usage), "{locale} {command}: {text}");
            assert!(text.contains(options), "{locale} {command}: {text}");
            if locale.starts_with("en") {
                assert!(
                    text.is_ascii(),
                    "English help contains untranslated text: {text}"
                );
            } else {
                assert!(
                    !text.contains("Print help") && !text.contains("Usage:"),
                    "{text}"
                );
            }
        }
        for args in [
            vec!["--help"],
            vec!["help"],
            vec![],
            vec!["do", "--remove", "--help"],
            vec!["do", "--add", "--help"],
            vec!["help", "do", "remove"],
            vec!["do", "help", "remove"],
        ] {
            let text = localized_help(&f, &args, locale);
            if args == ["do", "--remove", "--help"] {
                println!("\n========== {locale} | mavc do --remove --help ==========\n{text}");
            }
            assert!(text.contains(usage), "{locale} {args:?}: {text}");
            assert!(!text.contains("->"), "{locale} {args:?}: {text}");
            if locale.starts_with("en") {
                assert!(text.is_ascii(), "{text}");
            } else {
                assert!(
                    !text.contains("Print help") && !text.contains("Usage:"),
                    "{text}"
                );
            }
        }
    }
}

#[test]
fn unknown_and_c_locales_use_english_help() {
    let f = Fixture::new();
    for locale in ["C", "POSIX", "C.UTF-8", "fr_FR.UTF-8", "unknown"] {
        let text = localized_help(&f, &["do", "--remove", "--help"], locale);
        assert!(text.contains("Usage:") && text.contains("Export before removal"));
        assert!(text.is_ascii());
    }
}

#[test]
fn chinese_help_follows_script_and_region() {
    let f = Fixture::new();
    for (locale, simplified) in [
        ("zh_CN.UTF-8", true),
        ("zh_SG.UTF-8", true),
        ("zh-Hans", true),
        ("zh-Hans-TW", true),
        ("zh", true),
        ("zh_TW.UTF-8", false),
        ("zh_HK.UTF-8", false),
        ("zh_MO.UTF-8", false),
        ("zh-Hant", false),
        ("zh-Hant-CN", false),
    ] {
        let text = localized_help(&f, &["do", "--remove", "--help"], locale);
        if simplified {
            assert!(
                text.contains("选项：") && text.contains("导出") && text.contains("文件路径"),
                "{locale}: {text}"
            );
            assert!(
                !text.contains("選項") && !text.contains("匯出") && !text.contains("檔案"),
                "{locale}: {text}"
            );
        } else {
            assert!(
                text.contains("選項：") && text.contains("匯出") && text.contains("檔案路徑"),
                "{locale}: {text}"
            );
            assert!(
                !text.contains("选项") && !text.contains("导出") && !text.contains("文件"),
                "{locale}: {text}"
            );
        }
    }
}

#[test]
fn do_add_accepts_archives_before_files() {
    for action in ["add", "--add", "-+"] {
        for destination in ["--to"] {
            let f = Fixture::new();
            f.archive();
            std::fs::copy(f.0.join("test.mavc"), f.0.join("other.mavc")).unwrap();
            std::fs::write(f.0.join("second.FLAC"), FLAC).unwrap();
            f.ok(&[
                "do",
                action,
                destination,
                "test.mavc",
                "other.mavc",
                "song.mp3",
                "second.FLAC",
            ]);
            for name in ["test.mavc", "other.mavc"] {
                let archive = mavc::Archive::open(f.0.join(name)).unwrap();
                assert_eq!(archive.index.tracks.len(), 3);
                archive
                    .extract_track(f.0.join(name), 2, f.0.join("verify.flac"))
                    .unwrap();
                assert_eq!(std::fs::read(f.0.join("verify.flac")).unwrap(), FLAC);
            }
        }
    }
}

#[test]
fn do_add_archives_first_supports_repeated_destinations_and_spaces() {
    let f = Fixture::new();
    f.archive();
    std::fs::copy(f.0.join("test.mavc"), f.0.join("other archive.mavc")).unwrap();
    std::fs::write(f.0.join("new song.aac"), AAC).unwrap();
    f.ok(&[
        "do",
        "-+",
        "--to=test.mavc",
        "--to",
        "other archive.mavc",
        "new song.aac",
    ]);
    assert_eq!(f.open().index.tracks.len(), 2);
    assert_eq!(
        mavc::Archive::open(f.0.join("other archive.mavc"))
            .unwrap()
            .index
            .tracks
            .len(),
        2
    );
}

#[test]
fn do_add_archives_first_explicit_separator() {
    let f = Fixture::new();
    f.archive();
    // Archive paths need not end in .mavc; -- makes the boundary explicit.
    std::fs::copy(f.0.join("test.mavc"), f.0.join("archive.mp3")).unwrap();
    f.ok(&["do", "add", "--to", "archive.mp3", "--", "song.mp3"]);
    assert_eq!(
        mavc::Archive::open(f.0.join("archive.mp3"))
            .unwrap()
            .index
            .tracks
            .len(),
        2
    );
}

#[test]
fn imports_accept_valid_mp3_flac_and_aac() {
    let f = Fixture::new();
    std::fs::write(f.0.join("song.flac"), FLAC).unwrap();
    std::fs::write(f.0.join("song.aac"), AAC).unwrap();
    f.ok(&[
        "package",
        "-m",
        "song.mp3",
        "song.flac",
        "song.aac",
        "-a",
        "test.mavc",
    ]);
    assert_eq!(f.open().index.tracks.len(), 3);
    for (id, expected) in [MP3, FLAC, AAC].into_iter().enumerate() {
        f.open()
            .extract_track(f.0.join("test.mavc"), id, f.0.join("verify.audio"))
            .unwrap();
        assert_eq!(std::fs::read(f.0.join("verify.audio")).unwrap(), expected);
    }
}

#[test]
fn create_commands_reject_damaged_audio_without_touching_output() {
    for args in [
        vec!["music", "-m", "song.mp3", "bad.mp3", "-a", "output.mavc"],
        vec!["package", "-m", "song.mp3", "bad.mp3", "-a", "output.mavc"],
        vec!["package", "bad-manifest.json", "output.mavc"],
        vec!["create", "bad-manifest.json", "output.mavc"],
        vec!["-m", "song.mp3", "bad.mp3", "-a", "output.mavc"],
    ] {
        let f = Fixture::new();
        std::fs::write(f.0.join("bad.mp3"), b"not music").unwrap();
        std::fs::write(f.0.join("bad-manifest.json"), r#"{"describe":{"title":"bad"},"tracks":[{"path":"song.mp3","title":"valid","weight":1},{"path":"bad.mp3","title":"damaged","weight":1}]}"#).unwrap();
        let output = f.run(&args);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("bad.mp3") && error.contains("damaged or invalid audio"),
            "{error}"
        );
        assert!(!f.0.join("output.mavc").exists());
        std::fs::write(f.0.join("output.mavc"), b"preserve existing output").unwrap();
        assert!(!f.run(&args).status.success());
        assert_eq!(
            std::fs::read(f.0.join("output.mavc")).unwrap(),
            b"preserve existing output"
        );
    }
}

#[test]
fn add_rejects_damaged_audio_before_updating_any_archive() {
    for action in ["add", "--add", "-+"] {
        let f = Fixture::new();
        f.archive();
        let original = std::fs::read(f.0.join("test.mavc")).unwrap();
        std::fs::write(f.0.join("other.mavc"), &original).unwrap();
        std::fs::write(f.0.join("bad.mp3"), b"damaged audio").unwrap();
        let output = f.run(&[
            "do",
            action,
            "song.mp3",
            "bad.mp3",
            "--to",
            "test.mavc",
            "other.mavc",
        ]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("bad.mp3"));
        for name in ["test.mavc", "other.mavc"] {
            assert_eq!(std::fs::read(f.0.join(name)).unwrap(), original);
        }
    }
}

#[test]
fn rejects_empty_header_only_and_truncated_audio() {
    for (extension, valid) in [
        ("mp3", MP3),
        ("flac", FLAC),
        ("aac", AAC),
        ("wav", WAV),
        ("ogg", OGG),
    ] {
        for damaged in [
            vec![],
            valid[..7].to_vec(),
            valid[..valid.len() - 1].to_vec(),
        ] {
            let f = Fixture::new();
            let name = format!("bad.{extension}");
            std::fs::write(f.0.join(&name), damaged).unwrap();
            let output = f.run(&["music", "-m", &name, "-a", "output.mavc"]);
            assert!(!output.status.success(), "{name} was accepted");
            assert!(String::from_utf8_lossy(&output.stderr).contains(&name));
            assert!(!f.0.join("output.mavc").exists());
        }
    }
}

#[test]
fn rejects_corrupt_frames_after_valid_audio() {
    for (extension, mut bytes, offset) in [
        ("mp3", MP3.to_vec(), 417),
        ("flac", FLAC.to_vec(), 26),
        ("aac", AAC.to_vec(), 13 + 7),
    ] {
        // MP3: corrupt the second frame's sync header. AAC: corrupt the
        // second frame's coded data. FLAC: corrupt the stored MD5 checksum.
        bytes[offset] ^= 0xff;
        let f = Fixture::new();
        let name = format!("corrupt.{extension}");
        std::fs::write(f.0.join(&name), bytes).unwrap();
        let output = f.run(&["music", "-m", &name, "-a", "output.mavc"]);
        assert!(!output.status.success(), "{name} was accepted");
        assert!(!f.0.join("output.mavc").exists());
    }
}

#[test]
fn rejects_audio_with_misleading_extension() {
    let f = Fixture::new();
    std::fs::write(f.0.join("fake.mp3"), FLAC).unwrap();
    assert!(
        !f.run(&["music", "-m", "fake.mp3", "-a", "output.mavc"])
            .status
            .success()
    );
    assert!(!f.0.join("output.mavc").exists());
}

#[test]
fn flac_trailing_data_preserves_complete_audio() {
    let f = Fixture::new();
    let mut bytes = FLAC.to_vec();
    bytes.extend_from_slice(b"extra tail data");
    std::fs::write(f.0.join("tagged.flac"), &bytes).unwrap();
    for command in ["music", "package"] {
        f.ok(&[command, "-m", "tagged.flac", "-a", "test.mavc"]);
    }
    f.ok(&["do", "-+", "tagged.flac", "--to", "test.mavc"]);
    f.ok(&["extract", "test.mavc", "1", "copy.flac"]);
    assert_eq!(std::fs::read(f.0.join("copy.flac")).unwrap(), bytes);
}

#[test]
fn flac_trailing_data_does_not_hide_audio_damage() {
    for checksum_damage in [false, true] {
        let f = Fixture::new();
        let mut bytes = FLAC.to_vec();
        if checksum_damage {
            bytes[26] ^= 0xff;
        } else {
            bytes.pop();
        }
        bytes.extend_from_slice(b"extra tail data");
        std::fs::write(f.0.join("damaged.flac"), bytes).unwrap();
        let output = f.run(&["music", "-m", "damaged.flac", "-a", "output.mavc"]);
        assert!(!output.status.success());
        assert!(!f.0.join("output.mavc").exists());
    }
}

#[test]
fn flac_trailing_data_cannot_bypass_declared_sample_count() {
    let f = Fixture::new();
    let mut bytes = FLAC.to_vec();
    // Declare twice as many samples, with an absent (all-zero) MD5 checksum.
    bytes[25] = 32;
    bytes[26..42].fill(0);
    bytes.extend_from_slice(b"extra tail data");
    std::fs::write(f.0.join("missing.flac"), bytes).unwrap();
    assert!(
        !f.run(&["music", "-m", "missing.flac", "-a", "output.mavc"])
            .status
            .success()
    );
    assert!(!f.0.join("output.mavc").exists());
}

#[test]
fn wav_and_ogg_import_through_all_entry_points() {
    for (extension, bytes) in [("wav", WAV), ("ogg", OGG)] {
        let f = Fixture::new();
        let name = format!("song.{extension}");
        std::fs::write(f.0.join(&name), bytes).unwrap();
        for command in ["music", "package"] {
            f.ok(&[command, "-m", &name, "-a", "test.mavc"]);
            assert_eq!(f.open().index.tracks[0].codec, extension);
            assert_eq!(
                f.open().index.tracks[0].metadata.sample_rate_hz,
                Some(44100)
            );
        }
        std::fs::write(f.0.join("new-manifest.json"), format!(r#"{{"describe":{{"title":"test"}},"tracks":[{{"path":"{name}","title":"song","weight":1.0}}]}}"#)).unwrap();
        f.ok(&["package", "new-manifest.json", "test.mavc"]);
        f.ok(&["do", "-+", "--to", "test.mavc", &name]);
        f.ok(&["extract", "test.mavc", "1", "copy.audio"]);
        assert_eq!(std::fs::read(f.0.join("copy.audio")).unwrap(), bytes);
        f.ok(&["do", "-x", "0", "--from", "test.mavc", "--to", "export"]);
        assert_eq!(std::fs::read(f.0.join("export").join(name)).unwrap(), bytes);
    }
}

#[test]
fn wav_and_ogg_damage_preserves_existing_archives() {
    for (extension, bytes) in [("wav", WAV), ("ogg", OGG)] {
        let f = Fixture::new();
        f.archive();
        let original = std::fs::read(f.0.join("test.mavc")).unwrap();
        let name = format!("bad.{extension}");
        std::fs::write(f.0.join(&name), &bytes[..bytes.len() - 1]).unwrap();
        for command in ["music", "package"] {
            assert!(
                !f.run(&[command, "-m", &name, "-a", "test.mavc"])
                    .status
                    .success()
            );
            assert_eq!(std::fs::read(f.0.join("test.mavc")).unwrap(), original);
        }
        assert!(
            !f.run(&["do", "-+", "song.mp3", &name, "--to", "test.mavc"])
                .status
                .success()
        );
        assert_eq!(std::fs::read(f.0.join("test.mavc")).unwrap(), original);
    }
}

#[test]
fn wav_rejects_invalid_chunk_lengths_and_sample_alignment() {
    for offset in [4, 32, 40] {
        let f = Fixture::new();
        let mut bytes = WAV.to_vec();
        bytes[offset] ^= 1;
        std::fs::write(f.0.join("bad.wav"), bytes).unwrap();
        assert!(
            !f.run(&["music", "-m", "bad.wav", "-a", "test.mavc"])
                .status
                .success()
        );
    }
}

fn ogg_pages(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut pages = vec![];
    let mut offset = 0;
    while offset < bytes.len() {
        let count = bytes[offset + 26] as usize;
        let size = 27
            + count
            + bytes[offset + 27..offset + 27 + count]
                .iter()
                .map(|n| *n as usize)
                .sum::<usize>();
        pages.push((offset, size));
        offset += size;
    }
    pages
}

fn update_ogg_crc(bytes: &mut [u8]) {
    bytes[22..26].fill(0);
    let mut crc = 0u32;
    for byte in bytes.iter() {
        crc ^= u32::from(*byte) << 24;
        for _ in 0..8 {
            crc = if crc & 0x80000000 != 0 {
                (crc << 1) ^ 0x04c11db7
            } else {
                crc << 1
            };
        }
    }
    bytes[22..26].copy_from_slice(&crc.to_le_bytes());
}

#[test]
fn ogg_rejects_checksum_sequence_and_missing_end_marker() {
    let pages = ogg_pages(OGG);
    let (last, size) = *pages.last().unwrap();
    for mode in 0..5 {
        let f = Fixture::new();
        let mut bytes = OGG.to_vec();
        match mode {
            0 => bytes[last + size - 1] ^= 1, // Damaged payload and CRC.
            1 => {
                bytes[last + 18] += 1;
                update_ogg_crc(&mut bytes[last..]);
            }
            2 => {
                bytes[last + 5] &= !4;
                update_ogg_crc(&mut bytes[last..]);
            }
            3 => bytes.truncate(last), // Missing entire last page.
            _ => {
                // Valid page CRC, but invalid Vorbis audio packets.
                let payload = last + 27 + bytes[last + 26] as usize;
                bytes[payload..].fill(0xff);
                update_ogg_crc(&mut bytes[last..]);
            }
        }
        std::fs::write(f.0.join("bad.ogg"), bytes).unwrap();
        assert!(
            !f.run(&["music", "-m", "bad.ogg", "-a", "test.mavc"])
                .status
                .success(),
            "mode {mode}"
        );
        assert!(!f.0.join("test.mavc").exists());
    }
}

#[test]
fn mp3_and_aac_accept_tags_and_reject_later_frame_damage() {
    for (extension, audio, frame_len) in [("mp3", MP3, 417), ("aac", AAC, 13)] {
        let f = Fixture::new();
        let mut bytes = b"ID3\x04\x00\x00\x00\x00\x00\x00".to_vec();
        bytes.extend_from_slice(audio);
        let mut tag = [0; 128];
        tag[..3].copy_from_slice(b"TAG");
        bytes.extend_from_slice(&tag);
        let name = format!("tagged.{extension}");
        std::fs::write(f.0.join(&name), &bytes).unwrap();
        f.ok(&["music", "-m", &name, "-a", "test.mavc"]);
        let original = std::fs::read(f.0.join("test.mavc")).unwrap();
        bytes[10 + frame_len] = 0;
        std::fs::write(f.0.join(&name), bytes).unwrap();
        assert!(
            !f.run(&["do", "-+", &name, "--to", "test.mavc"])
                .status
                .success()
        );
        assert_eq!(std::fs::read(f.0.join("test.mavc")).unwrap(), original);
    }
}

#[test]
fn wav_accepts_pcm_float_extensible_and_unknown_chunks() {
    for (encoding, bits, extensible) in [
        (1u16, 8u16, false),
        (1, 16, false),
        (1, 24, false),
        (1, 32, false),
        (3, 32, false),
        (3, 64, false),
        (1, 24, true),
        (3, 32, true),
    ] {
        let f = Fixture::new();
        let align = 2 * (bits / 8);
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&(if extensible { 0xfffeu16 } else { encoding }).to_le_bytes());
        fmt.extend_from_slice(&2u16.to_le_bytes());
        fmt.extend_from_slice(&44100u32.to_le_bytes());
        fmt.extend_from_slice(&(44100u32 * u32::from(align)).to_le_bytes());
        fmt.extend_from_slice(&align.to_le_bytes());
        fmt.extend_from_slice(&bits.to_le_bytes());
        if extensible {
            fmt.extend_from_slice(&22u16.to_le_bytes());
            fmt.extend_from_slice(&bits.to_le_bytes());
            fmt.extend_from_slice(&3u32.to_le_bytes());
            fmt.extend_from_slice(&encoding.to_le_bytes());
            fmt.extend_from_slice(&[0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71]);
        }
        let data = vec![0; usize::from(align) * 16];
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(
            &(4 + 8 + fmt.len() as u32 + 10 + 8 + data.len() as u32).to_le_bytes(),
        );
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&fmt);
        bytes.extend_from_slice(b"JUNK\x01\x00\x00\x00\x42\x00data");
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&data);
        std::fs::write(f.0.join("valid.WAV"), bytes).unwrap();
        f.ok(&["music", "-m", "valid.WAV", "-a", "test.mavc"]);
        assert_eq!(
            f.open().index.tracks[0].metadata.bit_depth,
            Some(bits as u8)
        );
    }
}

#[test]
fn mp3_accepts_variable_bitrate_and_mpeg_versions() {
    for version in [3, 2, 0] {
        let f = Fixture::new();
        let mut bytes = vec![];
        for index in [9u8, 10, 11, 9] {
            let rate = match version {
                3 => 44100,
                2 => 22050,
                _ => 11025,
            };
            let bitrate = if version == 3 {
                match index {
                    9 => 128,
                    10 => 160,
                    _ => 192,
                }
            } else {
                match index {
                    9 => 80,
                    10 => 96,
                    _ => 112,
                }
            };
            let len = (if version == 3 { 144000 } else { 72000 }) * bitrate / rate;
            bytes.extend_from_slice(&[0xff, 0xe3 | (version << 3), index << 4, 0]);
            bytes.resize(bytes.len() + len - 4, 0);
        }
        std::fs::write(f.0.join("variable.mp3"), bytes).unwrap();
        f.ok(&["music", "-m", "variable.mp3", "-a", "test.mavc"]);
    }
}

#[test]
fn arrow_alias_is_no_longer_accepted() {
    let f = Fixture::new();
    f.archive();
    let original = std::fs::read(f.0.join("test.mavc")).unwrap();
    for args in [
        vec!["do", "-+", "song.mp3", "->", "test.mavc"],
        vec!["do", "-x", "0", "->", "test.mavc"],
        vec!["do", "-x", "0", "--from", "test.mavc", "->", "export"],
    ] {
        assert!(!f.run(&args).status.success());
        assert_eq!(std::fs::read(f.0.join("test.mavc")).unwrap(), original);
    }
}
