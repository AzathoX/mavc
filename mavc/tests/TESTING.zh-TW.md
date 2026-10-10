# MAVC 非 UI 測試用例

## 執行自動化測試

在非 UI crate 目錄執行：

```sh
cd /Volumes/data/Project/mavc/mavc
cargo test --offline
```

執行個別用例或一組用例：

```sh
cargo test --offline --test cli remove_exports_before_deleting
cargo test --offline --test cli remove_
cargo test --offline --test cli help_language_covers_all_commands
cargo test --offline cli::locale::tests
```

`tests/cli.rs` 的每個測試使用獨立暫存目錄，結束後清理，不修改使用者的歌曲。
自動化測試使用 `tests/fixtures/` 中可解碼的 MP3、FLAC、AAC 短靜音音訊，檢查匯入驗證、打包及匯出完整性；實際有聲播放仍需人工驗證。
測試名稱可直接傳給 `cargo test` 作為篩選條件。

## 自動化用例：基本命令

1. **幫助 `help`**：執行 `mavc help`。預期退出碼為 0，顯示幫助。
2. **版本 `version`**：執行 `mavc --version`。預期包含 Cargo 套件版本。
3. **音訊打包 `package_music`**：`mavc package -m song.mp3 -a test.mavc`。預期歸檔可開啟，包含 1 首。
4. **清單打包 `package_manifest`**：`mavc package manifest.json test.mavc`。預期歸檔標題等於清單標題。
5. **音訊快捷命令 `music`**：`mavc music -m song.mp3 -a test.mavc`。預期包含 1 首。
6. **舊快捷入口 `music_shortcut`**：`mavc -m song.mp3 -a test.mavc`。預期包含 1 首。
7. **建立命令 `create`**：`mavc create manifest.json test.mavc`。預期包含清單中的曲目。
8. **列表 `list`**：`mavc list test.mavc`。預期輸出包含歌曲名稱。
9. **資訊 `inspect`**：分別執行 `inspect test.mavc`、`inspect test.mavc 0`、`inspect 0 test.mavc`。預期三種形式都成功。
10. **隨機選曲 `pick`**：只有一首正權重歌曲時執行 `pick test.mavc`。預期選中 ID 0。
11. **權重 `weight`**：`mavc weight test.mavc 0=0.25`。預期重新開啟歸檔後，該曲目權重為 0.25。
12. **單首匯出 `extract`**：`mavc extract test.mavc 0 out.mp3`。預期輸出位元組與來源完全一致。
13. **全部匯出 `extract_all`**：在沒有同名檔案的目錄執行 `mavc extract test.mavc -all`。預期以原檔名匯出且位元組一致。
14. **無效播放 ID `play_invalid_track`**：`mavc play -o 99 test.mavc`。預期退出碼非 0，顯示找不到 ID 99，不啟動播放。

## 自動化用例：新增、移除與容量

15. **新增選項 `do_add`**：分別測試 `-+`、`--add`、`add`，搭配 `--to`；同一歸檔重複指定兩次。預期原有 1 首變成 3 首。
16. **移除選項 `do_remove`**：測試 `-x`、`--remove`、`remove`，搭配 `--from`。移除唯一的 ID 0 後，歸檔可開啟且為空。
17. **舊入口 `legacy_add_remove`**：使用頂層 `--add`、`--remove`。預期仍可新增／移除，未移除曲目的 ID 保留。
18. **缺參數／型別錯誤 `do_requires_action_and_destination`**：測試 `do` 無動作、新增／移除無來源或目標，以及以歌曲路徑當作移除 ID。預期皆退出非 0。
19. **超過 5 首 `no_track_limit`**：從 1 首連續新增 6 首。預期成功且包含 7 首。
20. **超過 150 MB `no_payload_limit`**：新增含合法 padding 中繼資料、長度超過 150 MB 的有效 FLAC 測試檔。預期成功、長度正確，且仍可移除另一首。測試會實際複製約 150 MB 資料。
21. **多個歸檔 `do_multiple_archives`**：一次向兩個歸檔新增，再使用重複 `--from` 移除 ID 0、1。預期兩個歸檔皆為空。

## 自動化用例：先匯出再移除

22. **長短參數組合 `remove_exports_before_deleting`**：來源用 `--from`，匯出目標用 `--to`。預期建立巢狀目錄、匯出內容正確、歌曲從歸檔移除。
23. **多個匯出目錄 `remove_exports_to_multiple_directories`**：`mavc do --remove 0 0 --from test.mavc --to one two --to three`。預期重複 ID 去重，三個目錄都有正確歌曲，歸檔為空。
24. **匯出失敗 `remove_export_failure_preserves_archive`**：分別測試目標已有同名歌曲、目標是檔案、移除清單包含不存在的 ID。預期退出非 0，歸檔位元組完全不變，已有歌曲不被覆寫。
25. **跨歸檔同名衝突 `remove_duplicate_export_names_preserves_archives`**：兩個歸檔含同名歌曲，匯出到同一目錄。預期拒絕操作，兩個歸檔的歌曲保留，不產生歌曲輸出。

## 自動化用例：幫助語言

26. **六個語言版本的幫助 `help_language_covers_all_commands`**：分別設定 `LC_ALL=en_US.UTF-8`、`zh_TW.UTF-8`、`zh_CN.UTF-8`、`ja_JP.UTF-8`、`es_ES.UTF-8`、`de_DE.UTF-8`。檢查各命令的 `--help`、主 `help`、不帶參數、`do --add --help`、`do --remove --help`、`help do remove`、`do help remove`。預期標題與說明使用所選語言；英語輸出不殘留中文或日文；詳細幫助包含來源與目標規則。
27. **英語回退 `unknown_and_c_locales_use_english_help`**：設定 `C`、`POSIX`、`C.UTF-8`、`fr_FR.UTF-8` 或未知語言。預期使用英語。
28. **地區格式 `cli::locale::tests::locale_variants`**：測試 `zh_CN`、`zh-TW`、`zh_HK`、`ZH-hans`、`ja_JP`、`ja-JP`、`es_ES`、`es-MX`、`de_DE`、`de-AT` 及不支援語言。預期選擇對應語言，不支援則英語。
29. **語言優先順序 `cli::locale::tests::environment_precedence`**：設定互相衝突的語言變數。預期 `LC_ALL` 優先；`C`／`POSIX` 壓過 `LANGUAGE`；其餘依 `LANGUAGE` 第一個偏好、`LC_MESSAGES`、`LANG` 選擇，空值忽略。

另有 2 個 Rust 文件範例編譯測試，驗證 `music()` 與 `play()` 的公開 API 用法可編譯。加上直接切換語言的測試，合計 53 個測試。

## 手動用例：真實播放

以下是待人工驗證項目，不代表已播放驗收通過。準備兩首短的、可正常播放的音訊檔案，命名為 `song-a.mp3` 與 `song-b.flac`，在獨立測試目錄操作。
文件中的 `mavc` 代表已編譯的執行檔；可使用絕對路徑 `/Volumes/data/Project/mavc/mavc/target/debug/mavc`。

先建立測試歸檔：

```sh
mavc music -m song-a.mp3 song-b.flac -a playback.mavc
mavc list playback.mavc
```

- **M01 預設播放**：`mavc play playback.mavc`。預期依權重選取一首，可聽到音訊，結束後正常退出。單次結果不驗證機率分布。
- **M02 指定曲目**：`mavc play -o 1 playback.mavc`。預期播放 ID 1。
- **M03 依序播放**：`mavc play --all playback.mavc`。預期先播放 ID 0，再播放 ID 1。
- **M04 混音**：`mavc play -c 0 1 playback.mavc`。預期兩首同時播放。
- **M05 系統播放器**：`mavc play -sp playback.mavc`。預期系統播放器開啟含兩首曲目的播放清單。
- **M06 瀏覽器播放**：`mavc play -bp playback.mavc`。預期瀏覽器開啟頁面，兩首可手動播放。
- **M07 瀏覽器混音**：`mavc play -c 0 1 -bp playback.mavc`。預期按下同時播放按鈕後兩首一起播放。

## 手動用例：系統語言回退

這些用例用於驗證平台行為，不在目前跨平台自動化測試的覆蓋範圍內。

- **M08 macOS**：在系統偏好語言為英語、中文、日文的環境，各自清除終端語言變數後執行以下命令。預期依 `AppleLanguages` 的第一個語言顯示幫助。
- **M09 Windows**：在英語、中文、日文系統分別清除 `LC_ALL`、`LANGUAGE`、`LC_MESSAGES`、`LANG` 後執行 `mavc --help`。預期依系統 UI 語言顯示幫助。

macOS／Linux 清除語言變數的單次命令：

```sh
env -u LC_ALL -u LANGUAGE -u LC_MESSAGES -u LANG mavc --help
```

Linux 未設定任何上述變數時，預期回退英語；目前沒有額外讀取桌面環境偏好。

## 安全 review 後的待修回歸用例

以下記錄先前 review 發現的風險，尚未修正，也未標為通過：

- **S01 M3U 換行注入**：建立標題含換行與測試網址的歸檔。修正後應確保產生的 M3U 不會多出來源曲目以外的播放項目。驗證產生的文字即可，不需要實際連線外部網址。
- **S02 不可信標頭配置記憶體**：建立很小、但標頭宣告巨大 JSON 區段的歸檔。修正後應在大量配置記憶體前拒絕；未修正前不要直接執行此惡意輸入，以免耗盡記憶體。


## 直接呼叫方法切換幫助語言

在 `src/cli/locale.rs` 的測試模組中使用 `help_in_language(language, subcommands)`，
不需要更改電腦或程序的語言環境變數：

```rust
let english = help_in_language(Language::English, &[]);
let chinese = help_in_language(Language::TraditionalChinese, &["do"]);
let simplified = help_in_language(Language::SimplifiedChinese, &["do", "remove"]);
let japanese = help_in_language(Language::Japanese, &["do", "remove"]);
let spanish = help_in_language(Language::Spanish, &["do", "remove"]);
let german = help_in_language(Language::German, &["do", "remove"]);
```

此方法只供測試使用。每次直接傳入語言，回傳該命令的幫助文字。
空的命令列表代表主幫助；`["do", "remove"]` 代表移除命令的幫助。
`help_language_can_be_switched_directly` 依序切換六個語言版本，最後切回英語，
驗證標題、選項、移除說明及來源與目標用法，不會殘留上一種語言。
原有自動化測試仍會另外驗證環境變數選擇語言的流程。

```sh
cargo test --offline help_language_can_be_switched_directly
```


## 查看語言切換的實際輸出

語言測試會印出各語言的移除命令幫助，並以語言標籤分隔。
Rust 測試預設會收集輸出；使用 `--nocapture` 可以直接看到文字：

```sh
cargo test --offline --test cli help_language_covers_all_commands -- --nocapture
cargo test --offline help_language_can_be_switched_directly -- --nocapture
```

若 IDE 使用 JSON 測試輸出與 `--show-output`，成功測試的輸出可能顯示在
`stdout` JSON 欄位或 IDE 的測試輸出面板。需要直接閱讀時，請在終端執行上述命令。


`chinese_help_follows_script_and_region` 驗證簡繁體依地區與文字標籤切換：
CN／SG／Hans 使用簡體，TW／HK／MO／Hant 使用繁體；明確文字標籤優先於地區，
單獨 `zh` 預設簡體。測試會檢查「选项／選項」、「导出／匯出」、「文件／檔案」沒有混用。

```sh
cargo test --offline --test cli chinese_help_follows_script_and_region
```


## 目標歸檔在前的新增用例

```sh
mavc do add --to a.mavc b.mavc song.mp3 other.flac
mavc do --add --to a.mavc song.mp3
mavc do -+ --to a.mavc -- song.mp3 other.flac
```

`do_add_accepts_archives_before_files` 驗證三種新增入口與兩種目標選項的組合，
兩個目標歸檔都會收到全部音訊；同時檢查大寫副檔名與匯出資料完整性。
`do_add_archives_first_supports_repeated_destinations_and_spaces` 驗證重複目標選項、
`--to=...` 以及含空白的路徑。
`do_add_archives_first_explicit_separator` 驗證使用 `--` 明確分隔，支援非 `.mavc` 的歸檔檔名。
未加 `--` 時，以第一個 MP3／FLAC／AAC／WAV／OGG 路徑作為音訊檔案清單起點；
歸檔檔名若也使用這些副檔名，請加 `--` 明確分隔。

```sh
cargo test --offline --test cli do_add_
```

## 自動化用例：損壞音訊拒絕匯入

- `imports_accept_valid_mp3_flac_and_aac`：三種有效音訊可打包，匯出與原檔完全一致。
- `create_commands_reject_damaged_audio_without_touching_output`：music、package 的音訊與 manifest 入口、create 及舊快捷入口拒絕偽音訊，不建立輸出，也不覆寫既有輸出。
- `add_rejects_damaged_audio_before_updating_any_archive`：三種新增入口遇到混合有效與損壞來源時，兩個目標歸檔皆保持不變。
- `rejects_empty_header_only_and_truncated_audio`：三種格式的空檔、僅有檔頭及尾端截斷都被拒絕。
- `rejects_corrupt_frames_after_valid_audio`：拒絕 MP3 後續幀檔頭損壞、AAC 後續幀解碼錯誤及 FLAC MD5 校驗失敗。
- `rejects_audio_with_misleading_extension`：拒絕把 FLAC 改名成 MP3。

```sh
cargo test --offline --test cli rejects_
cargo test --offline --test cli damaged_audio
```

匯入前會讀取整首音訊並完整解碼，MP3／AAC 另檢查幀邊界，FLAC 檢查宣告樣本數及可用校驗碼。此流程能拒絕可偵測的格式、截斷及解碼錯誤；沒有校驗碼的格式，無法保證偵測仍可合法解碼的所有位元變動。

### FLAC 尾端額外資料回歸

- `flac_trailing_data_preserves_complete_audio`：完整音訊後附加 15 bytes，music、package、add 都能匯入，匯出仍保留原檔全部位元組。
- `flac_trailing_data_does_not_hide_audio_damage`：相同尾端資料不會掩蓋截斷幀或 MD5 錯誤。
- `flac_trailing_data_cannot_bypass_declared_sample_count`：即使沒有 MD5，缺少宣告樣本仍拒絕。

解碼器可能把尾端額外資料併入最後一幀，因 CRC 不符而丟棄整幀。只在首次驗證失敗時，嘗試以最後幀的有效 CRC 邊界限制解碼範圍；仍須通過完整樣本數與可用 MD5 校驗。原檔不作修改。

## WAV、OGG 與 MP3／AAC 相容性

支援 WAV 的 PCM 8／16／24／32-bit、IEEE float 32／64-bit 及 WAVEFORMATEXTENSIBLE；OGG 支援單一 Vorbis 音訊流。OGG Opus、多路或串接流目前不支援。

- `wav_and_ogg_import_through_all_entry_points`：music、package（音訊與 manifest）、目標在前的 add，以及匯出、移除前匯出；位元組及索引格式一致，中繼資料可讀。
- `wav_and_ogg_damage_preserves_existing_archives`：損壞來源不覆寫既有歸檔，批次新增不插入有效來源。
- `wav_rejects_invalid_chunk_lengths_and_sample_alignment`：錯誤 RIFF 長度、區塊長度或樣本對齊會被拒絕。
- `wav_accepts_pcm_float_extensible_and_unknown_chunks`：驗證上述位元深度、浮點／擴充格式、大寫副檔名，以及合法未知區塊與奇數長度 padding。
- `ogg_rejects_checksum_sequence_and_missing_end_marker`：拒絕錯誤頁面 CRC、跳號、缺少 EOS、缺少整個最後頁面，以及 CRC 正確但無法解碼的音訊封包。
- `mp3_and_aac_accept_tags_and_reject_later_frame_damage`：帶 ID3v2／ID3v1 標籤可匯入；後續幀損壞不修改歸檔。空 ID3v2 標籤不再觸發解碼器誤判。
- `mp3_accepts_variable_bitrate_and_mpeg_versions`：MP3 的 VBR 與 MPEG-1／2／2.5 可匯入。

原本的空檔、僅檔頭、截斷測試也已擴充到 WAV 和 OGG。WAV 原始 PCM 通常沒有校驗碼，因此仍可合法解碼的樣本值變動無法判定為損壞。

`arrow_alias_is_no_longer_accepted` 驗證舊箭頭別名被拒絕，原歸檔保持不變。
