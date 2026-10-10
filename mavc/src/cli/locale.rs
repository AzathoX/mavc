//! Help localization. Command names, flags, IDs and file paths stay unchanged.
use clap::Command;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Language {
    English,
    TraditionalChinese,
    SimplifiedChinese,
    Japanese,
    Spanish,
    German,
}
impl Language {
    fn text(
        self,
        en: &'static str,
        zh_hant: &'static str,
        zh_hans: &'static str,
        ja: &'static str,
        es: &'static str,
        de: &'static str,
    ) -> &'static str {
        match self {
            Self::English => en,
            Self::TraditionalChinese => zh_hant,
            Self::SimplifiedChinese => zh_hans,
            Self::Japanese => ja,
            Self::Spanish => es,
            Self::German => de,
        }
    }
    fn from_locale(value: &str) -> Self {
        let value = value.trim().to_ascii_lowercase();
        let code = value.split(['_', '-', '.', '@']).next().unwrap_or("");
        match code {
            "zh" => {
                let locale = value.split(['.', '@']).next().unwrap_or("");
                let parts = locale.split(['_', '-']).collect::<Vec<_>>();
                // Explicit script wins over region, e.g. zh-Hans-TW and zh-Hant-CN.
                if parts.contains(&"hant") || parts.contains(&"cht") {
                    Self::TraditionalChinese
                } else if parts.contains(&"hans") || parts.contains(&"chs") {
                    Self::SimplifiedChinese
                } else if parts.iter().any(|part| matches!(*part, "tw" | "hk" | "mo")) {
                    Self::TraditionalChinese
                } else {
                    Self::SimplifiedChinese
                }
            }
            "ja" => Self::Japanese,
            "es" => Self::Spanish,
            "de" => Self::German,
            _ => Self::English,
        }
    }
}

// LC_ALL is an explicit override, including C/POSIX. LANGUAGE can contain a
// colon-separated preference list. Only the first preference is selected.
fn environment_language(mut get: impl FnMut(&str) -> Option<String>) -> Option<Language> {
    let nonempty = |value: String| (!value.trim().is_empty()).then_some(value);
    if let Some(value) = get("LC_ALL").and_then(nonempty) {
        return Some(Language::from_locale(&value));
    }
    // C/POSIX suppress LANGUAGE, as in the standard gettext convention.
    let base = get("LC_MESSAGES")
        .and_then(nonempty)
        .or_else(|| get("LANG").and_then(nonempty));
    if base.as_deref().is_some_and(|v| {
        matches!(v.trim().to_ascii_lowercase().as_str(), "c" | "posix")
            || v.to_ascii_lowercase().starts_with("c.")
    }) {
        return Some(Language::English);
    }
    if let Some(value) = get("LANGUAGE").and_then(nonempty) {
        return Some(Language::from_locale(
            value
                .split(':')
                .find(|v| !v.trim().is_empty())
                .unwrap_or("en"),
        ));
    }
    base.map(|value| Language::from_locale(&value))
}

pub(super) fn detect() -> Language {
    environment_language(|name| std::env::var(name).ok())
        .or_else(system_language)
        .unwrap_or(Language::English)
}

fn system_language() -> Option<Language> {
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "-g", "AppleLanguages"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let text = String::from_utf8(output.stdout).ok()?;
        let first = text
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && *line != "(" && *line != ")")?;
        Some(Language::from_locale(first.trim_matches(['"', ',', ' '])))
    }
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "[System.Globalization.CultureInfo]::CurrentUICulture.Name",
            ])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        Some(Language::from_locale(
            String::from_utf8(output.stdout).ok()?.trim(),
        ))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    } // Unix terminals normally supply LANG or LC_MESSAGES.
}

pub(super) fn command(language: Language) -> Command {
    use clap::CommandFactory;
    let mut command = super::args::Cli::command();
    // Build first so clap's generated help flags/subcommands can also be translated.
    command.build();
    translate(command, language)
}

fn translate(mut command: Command, lang: Language) -> Command {
    let name = command.get_name().to_owned();
    let about = match name.as_str() {
        "mavc" => lang.text(
            "Weighted music archive tool",
            "加權音樂歸檔工具",
            "加权音乐归档工具",
            "重み付き音楽アーカイブツール",
            "Herramienta de archivos musicales con selección ponderada",
            "Musikarchiv-Werkzeug mit gewichteter Auswahl",
        ),
        "package" => lang.text(
            "Create an archive from audio files or a JSON manifest",
            "從音訊檔案或 JSON 清單建立歸檔",
            "从音频文件或 JSON 清单建立归档",
            "音声ファイルまたは JSON マニフェストからアーカイブを作成",
            "Crear un archivo a partir de audio o un manifiesto JSON",
            "Archiv aus Audiodateien oder einem JSON-Manifest erstellen",
        ),
        "music" => lang.text(
            "Create an archive from audio files",
            "從音訊檔案建立歸檔",
            "从音频文件建立归档",
            "音声ファイルからアーカイブを作成",
            "Crear un archivo a partir de archivos de audio",
            "Archiv aus Audiodateien erstellen",
        ),
        "music-shortcut" => lang.text(
            "Legacy shortcut: mavc -m file.mp3",
            "舊版快捷用法：mavc -m file.mp3",
            "旧版快捷用法：mavc -m file.mp3",
            "従来の短縮形式：mavc -m file.mp3",
            "Atajo anterior: mavc -m file.mp3",
            "Bisherige Kurzform: mavc -m file.mp3",
        ),
        "create" => lang.text(
            "Create an archive from a JSON manifest",
            "從 JSON 清單建立歸檔",
            "从 JSON 清单建立归档",
            "JSON マニフェストからアーカイブを作成",
            "Crear un archivo a partir de un manifiesto JSON",
            "Archiv aus einem JSON-Manifest erstellen",
        ),
        "do" => lang.text(
            "Add audio files or remove tracks, optionally exporting before removal",
            "新增歌曲或移除曲目，可先匯出再移除",
            "新增歌曲或移除曲目，可先导出再移除",
            "曲の追加や削除。削除前の書き出しも可能",
            "Añadir o eliminar pistas, con exportación opcional antes de eliminarlas",
            "Titel hinzufügen oder entfernen, optional vorher exportieren",
        ),
        "add" => lang.text(
            "Add audio files to one or more existing archives",
            "將歌曲新增到一個或多個既有歸檔",
            "将歌曲新增到一个或多个既有归档",
            "既存のアーカイブに音声ファイルを追加",
            "Añadir audio a uno o varios archivos existentes",
            "Audiodateien zu einem oder mehreren vorhandenen Archiven hinzufügen",
        ),
        "remove" => lang.text(
            "Remove track IDs, optionally exporting songs first",
            "依曲目 ID 移除，可先匯出歌曲",
            "依曲目 ID 移除，可先导出歌曲",
            "トラック ID で削除。事前の書き出しも可能",
            "Eliminar pistas por ID, con exportación previa opcional",
            "Titel anhand ihrer IDs entfernen, optional vorher exportieren",
        ),
        "list" => lang.text(
            "List track IDs, formats and weights",
            "列出曲目 ID、格式與權重",
            "列出曲目 ID、格式与权重",
            "トラック ID、形式、重みを一覧表示",
            "Mostrar los ID, formatos y pesos de las pistas",
            "Titel-IDs, Formate und Gewichtungen auflisten",
        ),
        "play" => lang.text(
            "Play tracks from an archive",
            "播放歸檔中的曲目",
            "播放归档中的曲目",
            "アーカイブのトラックを再生",
            "Reproducir pistas de un archivo",
            "Titel aus einem Archiv abspielen",
        ),
        "inspect" => lang.text(
            "Show archive or track metadata",
            "顯示歸檔或曲目資訊",
            "显示归档或曲目资讯",
            "アーカイブまたはトラックのメタデータを表示",
            "Mostrar metadatos del archivo o de una pista",
            "Metadaten des Archivs oder eines Titels anzeigen",
        ),
        "pick" => lang.text(
            "Pick a track using its configured weight",
            "依權重選取一首曲目",
            "依权重选取一首曲目",
            "設定した重みに従ってトラックを選択",
            "Elegir una pista según su peso",
            "Titel anhand seiner Gewichtung auswählen",
        ),
        "weight" => lang.text(
            "Update a track's selection weight",
            "更新曲目的選取權重",
            "更新曲目的选取权重",
            "トラックの選択重みを更新",
            "Actualizar el peso de selección de una pista",
            "Auswahlgewichtung eines Titels ändern",
        ),
        "extract" => lang.text(
            "Extract one track or all tracks",
            "匯出單一或全部曲目",
            "导出单一或全部曲目",
            "1 曲またはすべての曲を書き出す",
            "Extraer una pista o todas las pistas",
            "Einen Titel oder alle Titel extrahieren",
        ),
        "help" => lang.text(
            "Print help for a command",
            "顯示命令幫助",
            "显示命令帮助",
            "コマンドのヘルプを表示",
            "Mostrar la ayuda de un comando",
            "Hilfe zu einem Befehl anzeigen",
        ),
        _ => "",
    };
    command = command.about(about);
    let details = match name.as_str() {
        "mavc" => Some(lang.text(
            include_str!("help/main.en.txt"),
            include_str!("help/main.zh-Hant.txt"),
            include_str!("help/main.zh-Hans.txt"),
            include_str!("help/main.ja.txt"),
            include_str!("help/main.es.txt"),
            include_str!("help/main.de.txt"),
        )),
        "do" | "add" | "remove" => Some(lang.text(
            include_str!("help/do.en.txt"),
            include_str!("help/do.zh-Hant.txt"),
            include_str!("help/do.zh-Hans.txt"),
            include_str!("help/do.ja.txt"),
            include_str!("help/do.es.txt"),
            include_str!("help/do.de.txt"),
        )),
        _ => None,
    };
    if let Some(details) = details {
        command = command.after_help(details);
    }
    // A template controls clap's headings as well as our own prose.
    let mut template = String::from("{about-with-newline}\n");
    template.push_str(lang.text(
        "Usage: ",
        "用法：",
        "用法：",
        "使い方：",
        "Uso: ",
        "Aufruf: ",
    ));
    template.push_str("{usage}\n");
    if command.get_subcommands().any(|sub| !sub.is_hide_set()) {
        template.push_str(lang.text(
            "\nCommands:\n",
            "\n命令：\n",
            "\n命令：\n",
            "\nコマンド：\n",
            "\nComandos:\n",
            "\nBefehle:\n",
        ));
        template.push_str("{subcommands}");
    }
    if command.get_positionals().any(|arg| !arg.is_hide_set()) {
        template.push_str(lang.text(
            "\nArguments:\n",
            "\n參數：\n",
            "\n参数：\n",
            "\n引数：\n",
            "\nArgumentos:\n",
            "\nArgumente:\n",
        ));
        template.push_str("{positionals}");
    }
    if command
        .get_arguments()
        .any(|arg| !arg.is_positional() && !arg.is_hide_set())
    {
        template.push_str(lang.text(
            "\nOptions:\n",
            "\n選項：\n",
            "\n选项：\n",
            "\nオプション：\n",
            "\nOpciones:\n",
            "\nOptionen:\n",
        ));
        template.push_str("{options}");
    }
    template.push_str("{after-help}\n");
    command = command.help_template(template);
    command = command.mut_args(|arg| {
        let help = match arg.get_id().as_str() {
            "help" => lang.text(
                "Print help",
                "顯示幫助",
                "显示帮助",
                "ヘルプを表示",
                "Mostrar ayuda",
                "Hilfe anzeigen",
            ),
            "version" => lang.text(
                "Print version information",
                "顯示版本資訊",
                "显示版本资讯",
                "バージョン情報を表示",
                "Mostrar la versión",
                "Versionsinformationen anzeigen",
            ),
            "files" => lang.text(
                "Audio files to add or package (one or more)",
                "新增或打包的音訊檔案（至少一個）",
                "新增或打包的音频文件（至少一个）",
                "追加または格納する音声ファイル（1 つ以上）",
                "Archivos de audio que se añadirán o empaquetarán (uno o más)",
                "Audiodateien zum Hinzufügen oder Verpacken (mindestens eine)",
            ),
            "archives" if name == "remove" => lang.text(
                "Source archives; accepts multiple paths or repeated --from",
                "來源歸檔；支援多個路徑或重複 --from",
                "来源归档；支持多个路径或重复 --from",
                "入力元アーカイブ。複数パスや --from の繰り返しが可能",
                "Archivos de origen; admite varias rutas o repetir --from",
                "Quellarchive; mehrere Pfade oder wiederholtes --from möglich",
            ),
            "archives" => lang.text(
                "Existing destination archives; accepts multiple paths or repeated --to",
                "既有目標歸檔；支援多個路徑或重複 --to",
                "既有目标归档；支持多个路径或重复 --to",
                "既存の出力先アーカイブ。複数パスや --to の繰り返しが可能",
                "Archivos de destino existentes; admite varias rutas o repetir --to",
                "Vorhandene Zielarchive; mehrere Pfade oder wiederholtes --to möglich",
            ),
            "ids" => lang.text(
                "Track IDs from mavc list, not audio file paths",
                "mavc list 顯示的曲目 ID，不是音訊檔案路徑",
                "mavc list 显示的曲目 ID，不是音频文件路径",
                "mavc list のトラック ID。音声ファイルのパスではありません",
                "ID de pistas de mavc list, no rutas de audio",
                "Titel-IDs aus mavc list, keine Audiodateipfade",
            ),
            "outputs" => lang.text(
                "Optional export directories; all exports must succeed before removal",
                "可選匯出目錄；全部匯出成功後才移除",
                "可选导出目录；全部导出成功后才移除",
                "任意の書き出し先。すべての書き出しが成功してから削除",
                "Directorios de exportación opcionales; se exporta todo antes de eliminar",
                "Optionale Exportverzeichnisse; erst nach vollständigem Export entfernen",
            ),
            "output" if name == "extract" => lang.text(
                "Output file; defaults to the track's original filename",
                "輸出檔案；預設使用曲目原檔名",
                "输出文件；默认使用曲目原文件名",
                "出力ファイル。既定は元のファイル名",
                "Archivo de salida; por defecto se usa el nombre original de la pista",
                "Ausgabedatei; standardmäßig der ursprüngliche Dateiname des Titels",
            ),
            "output" => lang.text(
                "Output archive; music mode defaults to <filename>.mavc or music.mavc",
                "輸出歸檔；音訊模式預設為 <filename>.mavc 或 music.mavc",
                "输出归档；音频模式默认为 <filename>.mavc 或 music.mavc",
                "出力アーカイブ。音声モードの既定は <filename>.mavc または music.mavc",
                "Archivo de salida; el modo de audio usa <filename>.mavc o music.mavc por defecto",
                "Ausgabearchiv; im Audiomodus standardmäßig <filename>.mavc oder music.mavc",
            ),
            "manifest" => lang.text(
                "JSON manifest path",
                "JSON 清單路徑",
                "JSON 清单路径",
                "JSON マニフェストのパス",
                "Ruta del manifiesto JSON",
                "Pfad zum JSON-Manifest",
            ),
            "manifest_output" => lang.text(
                "Output archive path for manifest mode",
                "清單模式的輸出歸檔路徑",
                "清单模式的输出归档路径",
                "マニフェストモードの出力アーカイブパス",
                "Ruta del archivo de salida para el modo de manifiesto",
                "Pfad zum Ausgabearchiv im Manifestmodus",
            ),
            "archive" => lang.text(
                "Archive path",
                "歸檔路徑",
                "归档路径",
                "アーカイブのパス",
                "Ruta del archivo",
                "Archivpfad",
            ),
            "random" => lang.text(
                "Weighted random playback (the default)",
                "依權重隨機播放（預設）",
                "依权重随机播放（默认）",
                "重みに従うランダム再生（既定）",
                "Reproducción aleatoria ponderada (predeterminada)",
                "Gewichtete Zufallswiedergabe (Standard)",
            ),
            "one" => lang.text(
                "Play the specified track ID",
                "播放指定曲目 ID",
                "播放指定曲目 ID",
                "指定したトラック ID を再生",
                "Reproducir la pista con el ID indicado",
                "Titel mit der angegebenen ID abspielen",
            ),
            "all" if name == "extract" => lang.text(
                "Extract all tracks into the current directory (-all also accepted)",
                "將全部曲目匯出到目前目錄（也接受 -all）",
                "将全部曲目导出到目前目录（也接受 -all）",
                "すべての曲を現在のディレクトリに書き出す（-all も可）",
                "Extraer todas las pistas al directorio actual (también acepta -all)",
                "Alle Titel ins aktuelle Verzeichnis extrahieren (auch -all möglich)",
            ),
            "all" => lang.text(
                "Play all tracks in archive order",
                "按歸檔順序播放全部曲目",
                "按归档顺序播放全部曲目",
                "すべての曲をアーカイブの順に再生",
                "Reproducir todas las pistas en el orden del archivo",
                "Alle Titel in Archivreihenfolge abspielen",
            ),
            "combine" => lang.text(
                "Mix two or more track IDs simultaneously",
                "同時混音兩個或更多曲目 ID",
                "同时混音两个或更多曲目 ID",
                "2 つ以上のトラック ID を同時にミックス",
                "Mezclar dos o más pistas simultáneamente",
                "Zwei oder mehr Titel gleichzeitig mischen",
            ),
            "system_player" => lang.text(
                "Open a playlist in the system player (-sp also accepted)",
                "以系統播放器開啟播放清單（也接受 -sp）",
                "以系统播放器开启播放清单（也接受 -sp）",
                "システムプレーヤーでプレイリストを開く（-sp も可）",
                "Abrir una lista en el reproductor del sistema (también acepta -sp)",
                "Wiedergabeliste im Systemplayer öffnen (auch -sp möglich)",
            ),
            "browser_player" => lang.text(
                "Open a browser player (-bp also accepted)",
                "開啟瀏覽器播放器（也接受 -bp）",
                "开启浏览器播放器（也接受 -bp）",
                "ブラウザープレーヤーを開く（-bp も可）",
                "Abrir el reproductor del navegador (también acepta -bp)",
                "Browserplayer öffnen (auch -bp möglich)",
            ),
            "archive_or_id" => lang.text(
                "Archive path or track ID; accepts either order",
                "歸檔路徑或曲目 ID，可交換順序",
                "归档路径或曲目 ID，可交换顺序",
                "アーカイブのパスまたはトラック ID。順序は交換可能",
                "Ruta del archivo o ID de pista; admite ambos órdenes",
                "Archivpfad oder Titel-ID; beide Reihenfolgen möglich",
            ),
            "second" => lang.text(
                "Optional track ID, or archive path if the ID comes first",
                "可選曲目 ID；若 ID 在前，這裡填歸檔路徑",
                "可选曲目 ID；若 ID 在前，这里填归档路径",
                "任意のトラック ID。ID が先の場合はアーカイブのパス",
                "ID opcional, o ruta del archivo si el ID se indica primero",
                "Optionale Titel-ID oder Archivpfad, wenn die ID zuerst steht",
            ),
            "assignment" => lang.text(
                "TRACK_ID=WEIGHT; weight must be between 0 and 1",
                "TRACK_ID=WEIGHT；權重須介於 0 與 1",
                "TRACK_ID=WEIGHT；权重须介于 0 与 1",
                "TRACK_ID=WEIGHT。重みは 0 から 1 の範囲",
                "TRACK_ID=WEIGHT; el peso debe estar entre 0 y 1",
                "TRACK_ID=WEIGHT; die Gewichtung muss zwischen 0 und 1 liegen",
            ),
            "track_id" => lang.text(
                "Track ID to extract (unless --all is specified)",
                "要匯出的曲目 ID（指定 --all 時可省略）",
                "要导出的曲目 ID（指定 --all 时可省略）",
                "書き出すトラック ID（--all 指定時は省略）",
                "ID de la pista que se extraerá (salvo si se usa --all)",
                "ID des zu extrahierenden Titels (außer bei --all)",
            ),
            "subcommand" => lang.text(
                "Command to show help for",
                "要顯示幫助的命令",
                "要显示帮助的命令",
                "ヘルプを表示するコマンド",
                "Comando cuya ayuda se mostrará",
                "Befehl, dessen Hilfe angezeigt werden soll",
            ),
            _ => return arg,
        };
        arg.help(help).long_help(help)
    });
    command.mut_subcommands(|subcommand| translate(subcommand, lang))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn select(vars: &[(&str, &str)]) -> Option<Language> {
        environment_language(|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_string())
        })
    }
    // Select a language directly without reading or changing process-wide locale settings.
    fn help_in_language(language: Language, subcommands: &[&str]) -> String {
        let mut root = command(language);
        let mut selected = &mut root;
        for name in subcommands {
            selected = selected
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("unknown help command: {name}"));
        }
        selected.render_help().to_string()
    }

    #[test]
    fn help_language_can_be_switched_directly() {
        for (language, usage, options, description) in [
            (Language::English, "Usage:", "Options:", "Remove track IDs"),
            (
                Language::TraditionalChinese,
                "用法：",
                "選項：",
                "依曲目 ID 移除",
            ),
            (
                Language::SimplifiedChinese,
                "用法：",
                "选项：",
                "依曲目 ID 移除",
            ),
            (
                Language::Japanese,
                "使い方：",
                "オプション：",
                "トラック ID で削除",
            ),
            (
                Language::Spanish,
                "Uso:",
                "Opciones:",
                "Eliminar pistas por ID",
            ),
            (
                Language::German,
                "Aufruf:",
                "Optionen:",
                "Titel anhand ihrer IDs entfernen",
            ),
            // Switching back must not retain text from the previous language.
            (Language::English, "Usage:", "Options:", "Remove track IDs"),
        ] {
            for path in [&[][..], &["do"][..], &["do", "remove"][..]] {
                let text = help_in_language(language, path);
                assert!(text.contains(usage), "{language:?} {path:?}: {text}");
                assert!(text.contains(options), "{language:?} {path:?}: {text}");
                assert!(!text.contains("->"), "{language:?} {path:?}: {text}");
                if path == ["do", "remove"] {
                    println!(
                        "\n========== {language:?} | direct language selection ==========\n{text}"
                    );
                    assert!(text.contains(description), "{text}");
                }
                if language == Language::English {
                    assert!(text.is_ascii(), "{text}");
                } else {
                    assert!(
                        !text.contains("Usage:") && !text.contains("Print help"),
                        "{text}"
                    );
                }
            }
        }
    }

    #[test]
    fn locale_variants() {
        for value in ["zh-TW", "zh_HK", "zh_MO", "zh-Hant", "zh-Hant-CN", "zh_CHT"] {
            assert_eq!(Language::from_locale(value), Language::TraditionalChinese);
        }
        for value in [
            "zh_CN.UTF-8",
            "zh-SG",
            "ZH-hans",
            "zh-Hans-TW",
            "zh_CHS",
            "zh",
        ] {
            assert_eq!(Language::from_locale(value), Language::SimplifiedChinese);
        }
        for value in ["ja_JP.UTF-8", "ja-JP"] {
            assert_eq!(Language::from_locale(value), Language::Japanese);
        }
        for value in ["es_ES.UTF-8", "es-MX", "ES-ar"] {
            assert_eq!(Language::from_locale(value), Language::Spanish);
        }
        for value in ["de_DE.UTF-8", "de-AT", "de_CH"] {
            assert_eq!(Language::from_locale(value), Language::German);
        }
        for value in ["C", "POSIX", "en_US", "fr_FR", "unknown"] {
            assert_eq!(Language::from_locale(value), Language::English);
        }
    }
    #[test]
    fn environment_precedence() {
        assert_eq!(select(&[]), None);
        assert_eq!(
            select(&[("LC_ALL", "ja_JP"), ("LANGUAGE", "zh"), ("LANG", "en")]),
            Some(Language::Japanese)
        );
        assert_eq!(
            select(&[("LC_ALL", "C"), ("LANGUAGE", "zh")]),
            Some(Language::English)
        );
        assert_eq!(
            select(&[("LANG", "C.UTF-8"), ("LANGUAGE", "zh")]),
            Some(Language::English)
        );
        assert_eq!(
            select(&[("LANGUAGE", "ja:zh:en"), ("LANG", "zh_CN")]),
            Some(Language::Japanese)
        );
        assert_eq!(
            select(&[("LC_MESSAGES", "zh_TW"), ("LANG", "ja")]),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            select(&[("LC_ALL", ""), ("LANG", "ja")]),
            Some(Language::Japanese)
        );
        assert_eq!(select(&[("LANGUAGE", "de:zh")]), Some(Language::German));
    }
}
