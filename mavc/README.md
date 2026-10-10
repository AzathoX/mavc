# mavc：核心函式庫與 CLI 程式碼學習指南

本文件按目前原始碼逐檔、逐函式及重要邏輯區塊解說。閱讀快照：2026-10-03。函式庫版本 `0.5.0`，封存容器版本 `1`；兩者不是同一個版本號。整體架構、review 結果及驗證紀錄見 [整體學習導讀](../README.learning.zh-TW.md)，桌面介面見 [mavc-app 指南](../mavc-app/README.md)。

## 1. 先理解這個 crate

![圖解：1. 先理解這個 crate](../docs/diagrams/core-01.svg)

此目錄是一個 Cargo package，同時產出 library 和命令列 binary：`src/lib.rs` 是其他 Rust 程式使用的入口；`src/main.rs` 是終端機執行的入口。桌面 app 依賴 library，不會執行 CLI。

```text
src/lib.rs → api → core
src/main.rs → cli → mavc library
```

`Cargo.toml` 的 `edition = "2024"` 指 Rust 語言 edition，不代表最低 Rust 編譯器版本已完整宣告。`clap` 處理命令列，`serde` 與 `serde_json` 處理資料結構和 JSON，`lofty` 讀音訊標籤，`rand` 抽選曲目，`rodio` 負責 CLI 的音訊解碼與輸出。`rodio` 關閉預設 feature，明確啟用播放與所需解碼器；`symphonia` 用於匯入前完整解碼。支援 MP3、FLAC、AAC（ADTS）、WAV（PCM／IEEE 浮點）和 OGG（Vorbis）。

`Cargo.lock` 固定解析後的依賴版本；`target/` 是編譯產物，不是手寫程式碼。`mavc.toml` 是限制範例，但目前讀取位置有問題，見第 7 節。

## 2. Rust 語法速讀

![圖解：2. Rust 語法速讀](../docs/diagrams/core-02.svg)

- `Result<T, Error>`：成功是 `Ok(T)`，失敗是 `Err(Error)`；`?` 遇到錯誤提早回傳，可透過 `From` 轉型。
- `Option<T>`：資料可能存在，`Some(T)` 表示有值，`None` 表示沒有；不是錯誤。
- `impl AsRef<Path>`：接受多種能借用成路徑的參數；`impl Into<PathBuf>` 則取得可持有的路徑值。
- `&Track` 是借用，`Track` 是持有；`.cloned()` 複製借用的物件，使回傳值不依賴原本容器的生命週期。
- `pub` 是對外 API，`pub(super)` 限定父模組能使用。`mod core` 雖然私有，但 `pub use` 可以選擇性重新公開裡面的項目。
- `#[derive(...)]` 產生 trait 實作，`#[serde(default)]` 讓 JSON 缺少欄位時補預設值。
- `checked_add`、`checked_mul` 偵測整數溢位；`as` 轉型本身不是完整的資料有效性驗證。
- `Read`、`Write`、`Seek` 是不同的 I/O 能力。`.take(n)` 限制讀取量，`io::copy` 搬運位元組，不進行音訊轉碼。

## 3. `src/lib.rs`、`src/core/mod.rs`、`src/main.rs`

![圖解：3. src/lib.rs、src/core/mod.rs、src/main.rs](../docs/diagrams/core-03.svg)

`lib.rs` 的第一段公開 `api`，第二段載入私有 `core`，最後重新公開 builder、資料模型、`Archive` 及操作函式。使用者因此可以寫 `mavc::Archive`，不需要知道內部 `core/archive.rs` 的位置。

`core/mod.rs` 宣告七個子模組，再集中轉出必要項目。這是一個把實作檔拆開、保持外部名稱簡單的例子。

`main.rs` 載入 `cli`，呼叫 `cli::run()`。成功回傳 `ExitCode::SUCCESS`；失敗把訊息送到 stderr，再回傳失敗退出碼。命令流程不應在 library 裡直接 `exit`，由最外層決定退出碼較容易重用。

## 4. `src/core/model.rs`：磁碟資料模型

![圖解：4. src/core/model.rs：磁碟資料模型](../docs/diagrams/core-04.svg)

### `Description`

![圖解：Description](../docs/diagrams/core-05.svg)

`title` 必填，`creator` 和 `description` 可缺省。它是封存整體的介紹，不是每首歌的資訊。

### `TrackSource`、`default_weight`、`CreateManifest`

![圖解：TrackSource、default_weight、CreateManifest](../docs/diagrams/core-06.svg)

`TrackSource` 是打包之前的輸入：來源檔路徑、顯示標題和權重。manifest 沒寫 `weight` 時，`default_weight()` 回傳 `1.0`。`CreateManifest` 把一份 `Description` 和一組 `TrackSource` 包在一起。

`MusicBuilder` 的初始權重是 `1/N`；JSON manifest 的缺省權重是 `1`。兩者在等權重抽選時機率相同，但儲存數字不同。

### `Track`

![圖解：Track](../docs/diagrams/core-07.svg)

打包之後的索引記錄包含 `id`、`title`、`file_name`、`codec`、`offset`、`length`、`weight`、可選 `cover_art_url` 和 `metadata`。`offset` 相對於音訊 payload 起點，不是相對於檔案起點。`id` 是識別碼，不保證等於向量位置：移除曲目後其餘 ID 不會全部重新編號。

### `AudioMetadata`

![圖解：AudioMetadata](../docs/diagrams/core-08.svg)

文字標籤：title、artist、album、genre、year、comment。其他欄位：track_number、duration_ms、bitrate_kbps、sample_rate_hz、channels、bit_depth。每個欄位都是 `Option`，未知值不應解釋成零。`Default` 使整組欄位預設為 `None`，支援缺少新欄位的舊索引。

### `MusicIndex`

![圖解：MusicIndex](../docs/diagrams/core-09.svg)

包含 `format = "MVLC"`、`version` 和 tracks。MVLC 是封存內的 JSON 索引識別值，不是另一個獨立音訊編碼。

這裡的結構沒有 `rename_all = "camelCase"`，寫出的多字欄位是 `file_name`、`duration_ms` 等 snake_case。桌面 IPC 若要另一種命名，應另外定義轉換層，不能直接改這裡而忽略既有封存相容性。

## 5. `src/core/format.rs`：位元組排列與檔案替換

![圖解：5. src/core/format.rs：位元組排列與檔案替換](../docs/diagrams/core-10.svg)

### 格式常數

![圖解：格式常數](../docs/diagrams/core-11.svg)

`MAGIC = b"MAVC"` 是前四個位元組，`VERSION = 1`，`HEADER_LEN = 22`。所有數字使用小端序。

```text
位元組  0..4   MAGIC，4 bytes
位元組  4..6   version，u16，2 bytes
位元組  6..10  description JSON 長度，u32，4 bytes
位元組 10..14  index JSON 長度，u32，4 bytes
位元組 14..22  payload 長度，u64，8 bytes
接下來：description JSON → index JSON → 所有原始音訊依序拼接
```

例如 description 長 80、index 長 300，則 `payload_start = 22 + 80 + 300 = 402`。某曲目 offset 是 1000、length 是 200，實際資料區間是 `[1402, 1602)`。

### `sibling_temp_path`

![圖解：sibling_temp_path](../docs/diagrams/core-12.svg)

取得目的檔所在目錄和檔名，組成 `.<name>.mavc-<pid>-<timestamp>.tmp`。把暫存檔放同一目錄，有助於最後以 rename 替換。特殊的 parent 缺省值目前是 `../../..`，不要把它解讀成通常的目前目錄；一般有檔名的相對路徑 parent 也可能是空路徑。

### 兩個 `replace_file`

![圖解：兩個 replace_file](../docs/diagrams/core-13.svg)

`#[cfg(not(windows))]` 版本直接 rename；Windows 版本先把目的檔移到備份，再移入暫存檔，失敗時嘗試還原，成功後刪備份。Windows 這段是多步驟操作，不應宣稱與單一 rename 有完全相同的原子性。這些 helper 用於替換已存在的封存，不用於首次建立。

## 6. `src/core/error.rs`：錯誤傳播

![圖解：6. src/core/error.rs：錯誤傳播](../docs/diagrams/core-14.svg)

`Error` 分成 `Io`、`Json`、`Invalid`。`Display` 提供可讀訊息，`std::error::Error` 讓它能作為一般 Rust 錯誤。兩個 `From` 實作讓 I/O 和 JSON 函式後面可直接用 `?`。

例如 `File::open(path)?` 會把 `io::Error` 包成 `Error::Io`。手動驗證權重失敗則用 `Error::Invalid`。目前 `Invalid` 的顯示前綴是「invalid MAVC archive」，因此輸入參數錯誤也可能帶這個前綴。

## 7. 容量限制已移除

新增與移除不再讀取 `mavc.toml`，也不再限制曲目數或音訊容量；仍保留整數溢位與格式欄位大小檢查。

使用 `mavc do -+ song.mp3 --to archive.mavc` 新增，或 `mavc do -x 0 --from archive.mavc` 移除。

## 8. `src/core/metadata.rs`：格式識別與標籤

![圖解：8. src/core/metadata.rs：格式識別與標籤](../docs/diagrams/core-16.svg)

### `codec_for`

![圖解：codec_for](../docs/diagrams/core-17.svg)

取得副檔名、轉小寫，接受 `.mp3`、`.flac`、`.aac`、`.wav`、`.ogg`。`validation.rs` 會在寫入前完整解碼，並檢查內容與副檔名是否一致。MP3／AAC 檢查幀邊界，FLAC 檢查樣本數及可用 MD5，`containers.rs` 檢查 WAV 區塊與 OGG 頁面 CRC、順序及結束標記。OGG 支援單一 Vorbis 音訊流；不支援 Opus、多路或串接流。

### `read_audio_metadata`

![圖解：read_audio_metadata](../docs/diagrams/core-18.svg)

先 `Probe::open`，再 `.read()`。任一步失敗就回傳空 metadata，故「無法讀標籤」不會阻止打包。這種容錯能處理沒有標籤的檔案，但也會掩蓋損壞音訊；學習時應分清楚「保留原始檔案」和「證明它可播放」。

### `metadata_from_tagged_file`

![圖解：metadata_from_tagged_file](../docs/diagrams/core-19.svg)

從技術 properties 取得時長、位元率等；從 primary tag 或 first tag 取得文字。`.and_then()` 串接可能不存在的 tag；`.map(|s| s.into_owned())` 把借用字串轉成持有字串。時長先取毫秒，再限制到 u64 可表示的範圍。year 使用 tag 的 date 字串，內容可能包含完整日期，不一定只是四位年份。

## 9. `src/core/archive.rs`：讀取、檢驗和抽選

![圖解：9. src/core/archive.rs：讀取、檢驗和抽選](../docs/diagrams/core-20.svg)

### `Archive`

![圖解：Archive](../docs/diagrams/core-21.svg)

保存公開的 description、index，以及內部的 payload 起點。它沒有保存開啟的 `File`；之後每次讀曲目會重新開啟傳入路徑。`Clone` 複製資料，不複製整個封存音訊。

### `Archive::open`，依程式區塊閱讀

![圖解：Archive::open，依程式區塊閱讀](../docs/diagrams/core-22.svg)

1. 開檔並 `read_exact` 固定 22 bytes，拒絕太短的 header。
2. 檢查 MAGIC，解析 version，拒絕不支援版本。這裡的 `try_into().unwrap()` 用於固定長度切片，切片長度由程式控制，與任意輸入上的 unwrap 不同。
3. 解析 description、index、payload 長度，配置兩個 JSON buffer，再讀取、反序列化。
4. 檢查 index format 和 version；計算 payload 起點與預期檔案長度，要求實際長度完全相等。
5. 每首歌驗證 weight 有限且非負，使用 checked_add 算 offset+length，要求曲目不超出 payload。
6. 排序曲目區間，透過 `windows(2)` 比較相鄰區間，拒絕重疊。
7. 建立 `Archive` 回傳。

目前尚未檢查 ID 唯一性、codec 白名單，也沒有在 JSON buffer 配置前把 header 宣告長度與檔案大小比較。`open` 成功表示通過這些結構檢查，並不代表音訊一定可播放。

### `pick_random`

![圖解：pick_random](../docs/diagrams/core-23.svg)

加總權重，若總和非正或不是有限值就回傳 `None`。在 `[0,total)` 取隨機數，依序減去每首權重，落在哪個區間就選哪首。機率是 `weight_i / sum(weights)`。

```text
權重 [1, 2, 1] → total=4 → 機率 [25%, 50%, 25%]
抽到 2.4：越過第一區間 [0,1)，落在第二區間 [1,3)
```

weight 不必總和等於 1；零權重不會被選中。單首權重有限，也不代表很多首相加一定有限，因此仍需檢查 total。

### `extract_track`

![圖解：extract_track](../docs/diagrams/core-24.svg)

依 ID 找曲目 → 再開來源檔 → seek 到 payload_start+offset → take(length) → 建立輸出 → copy。`File::create` 會覆寫既有檔案；library 本身沒有 CLI `--all` 的防覆寫前置檢查，也沒有驗證實際 copy 數量等於 length。

### `open_track`

![圖解：open_track](../docs/diagrams/core-25.svg)

同樣定位，但回傳 `io::Take<File>`，不建立輸出檔。限制讀取量可避免讀進下一首歌。這不是一個完整的獨立「子檔案」抽象：底層仍是整個 archive 的 `File`，`Take` 限制讀取，不把所有 seek/位置語意重新定位成曲目起點。不能只依註解認定它完全等同零起點的獨立音訊檔。

### `track_metadata`

![圖解：track_metadata](../docs/diagrams/core-26.svg)

若索引已有 duration/title/artist/album 任一欄位，直接回傳快取。否則依 codec 指定 lofty 的 FileType，從曲目 reader 解析；失敗回傳既有 metadata。這是整份快取判斷，不是逐欄補缺：已有 title 但沒有 artist 時，並不一定再讀音訊補 artist。

## 10. `src/core/operations.rs`：建立、增刪、更新

![圖解：10. src/core/operations.rs：建立、增刪、更新](../docs/diagrams/core-27.svg)

### `create_archive`

![圖解：create_archive](../docs/diagrams/core-28.svg)

第一段拒絕空 manifest。接著 enumerate 來源，從 0 指派 ID；檢查有限非負 weight、支援的副檔名、來源大小和 UTF-8 檔名；讀取 metadata，累加 payload 長度，建立 Track。

第二段序列化 description 和 MusicIndex，以 `u32::try_from` 確認兩份 JSON 長度能放進 header。額外保存每首預期長度，供後續檢查。

第三段用 `File::create(output)`，依序寫 header、JSON、所有音訊。每次 copy 後比對長度，偵測来源在打包期間改變。此函式直接寫目的檔，失敗可能留下半成品；輸出若是任何來源檔的同一路徑或同一實體檔，會先截斷來源，造成資料損失。這與後面的暫存檔替換流程不同。

### `add_tracks` 和 `remove_tracks`

![圖解：add_tracks 和 remove_tracks](../docs/diagrams/core-29.svg)

`add_tracks` 拒絕空 archive 清單和空 audio 清單，再逐個 archive 呼叫 rewrite。`remove_tracks` 拒絕空 archive 清單，把 ID 放進 HashSet 去重，再逐個 rewrite；目前 library 的 removals 可空，CLI 透過 clap 要求至少一個。

一批 archive 是逐個更新：前面成功、後面失敗時，前面的更新不會回滾。這不是跨檔案交易。

### `rewrite_archive`，重要區塊

![圖解：rewrite_archive，重要區塊](../docs/diagrams/core-30.svg)

1. 載入限制，將 MB 轉 bytes，開舊 archive，複製 tracks。
2. 檢查所有待刪 ID 存在，再 retain 未刪曲目。
3. 算最終曲目數，拒絕超限。
4. 處理新增檔：codec、大小、檔名、metadata；以最小未佔用 ID 指派，weight 固定 `1.0`。
5. 重算每首 offset，形成連續 payload；產生新 description/index。
6. 以 `create_new(true)` 建立相鄰暫存檔，避免直接覆寫同名暫存檔。
7. 寫新 header/JSON，從舊檔逐首 seek+take+copy 保留曲目，接著 copy 新檔。
8. `sync_all()`，寫入失敗則嘗試清暫存；成功呼叫 replace_file。

新增檔和保留曲目的 copy 沒有比對預期長度，存在來源在檢查後改變而產生無法重新開啟封存的風險。暫存流程避免寫入中途直接破壞原檔，但目前沒有替換前重新 `Archive::open(temp)`。

### `safe_relative_path`

![圖解：safe_relative_path](../docs/diagrams/core-31.svg)

把字串變成 Path，拒絕空路徑或任何非 Normal component，例如 root、parent (`..`) 等。允許普通多層相對路徑。它目前沒有被抽取流程呼叫，也不負責防止輸出目錄內既有 symlink 導向別處。

### `Archive::update_track_details`

![圖解：Archive::update_track_details](../docs/diagrams/core-32.svg)

參數包含 title、artist、album、genre、year、cover URL、weight。檢查 weight 在 `[0,1]` 且有限 → 找 ID → 同步設定 track.title 和 metadata.title → 更新其他欄位 → 重新序列化索引 → 暫存檔寫新 header/JSON → 原封不動複製整份 payload → sync 與 replace。

修改的是 MAVC 索引，**沒有寫回 MP3/FLAC/AAC 內部 tags**。抽出音訊後，播放器仍可能讀到原本 tag。此方法只更新這首 weight，不重分配其他曲目。

### `Archive::update_track_weight`

![圖解：Archive::update_track_weight](../docs/diagrams/core-33.svg)

同樣驗證 weight。記錄舊值，設定指定曲目，再把 `1-weight` 平均分給其他 `N-1` 首；`saturating_sub(1)` 避免 usize 下溢。只有一首時不做分配，其實際抽選機率仍是 100%（只要 weight > 0）。

然後與 details 更新一樣重寫 JSON、保留 payload、替換檔案，回傳 `(old_weight,new_weight)`。CLI `weight` 使用此方法；桌面編輯表單使用 details 方法，因此兩個操作對其他曲目權重的影響不同。

## 11. `src/api/mod.rs`：builder API

![圖解：11. src/api/mod.rs：builder API](../docs/diagrams/core-34.svg)

### `music()`、`MusicBuilder` 與各方法

![圖解：music()、MusicBuilder 與各方法](../docs/diagrams/core-35.svg)

`music()` 回傳空 builder。`from<I,P>` 接受任何可迭代且元素可轉成 PathBuf 的集合，收集成 files；`output` 指定目的路徑。兩者拿走 `self` 再回傳 `self`，因此能鏈式呼叫。

`create()`：拒絕空 files → 取得指定或預設輸出 → 用輸出 stem 決定封存 title → 算 `1/N` → 用每個來源 stem 決定曲目 title → 形成 manifest → 呼叫 create_archive → 回傳輸出路徑。

`default_output_path()`：單一來源用 `<stem>.mavc`，多個來源用 `music.mavc`。回傳相對路徑，落在目前工作目錄，不一定是音訊來源所在目錄。也不會自動避免覆寫。

### `play()`、`PlayBuilder`、`PlaybackTrack`

![圖解：play()、PlayBuilder、PlaybackTrack](../docs/diagrams/core-36.svg)

`play()` 只建立選曲器，不發出聲音。`PlaybackTrack` 包含一份 Track 和 bytes reader。

- `archive` 設封存路徑。
- `random` 清除單選和多選，回到加權抽選。
- `one(id)` 設單選並清除多選。
- `combine(ids)` 清除單選並保存多選。
- `stream()` 要求 archive，開封存，再按 one 或 random 找到 ID、複製 Track、回傳 open_track reader。
- `streams()` 要求 archive 和至少兩個 ID，開一次封存，再為每個 ID 建獨立 reader，collect 成向量；任一錯誤就整體回傳 Err。

`combine(...).stream()` 不使用多選清單，仍會走隨機選取；多選要配 `.streams()`。API 未去重多選 ID、未限制最多五首；UI 限制不是 library 保證。

```rust,no_run
let selected = mavc::play().archive("music.mavc").one(0).stream()?;
// 此時只得到 reader，還需要 decoder/player 才會有聲音。
let streams = mavc::play().archive("music.mavc").combine([0, 1]).streams()?;
# Ok::<(), mavc::Error>(())
```

## 12. `src/cli/mod.rs`：參數前處理

![圖解：12. src/cli/mod.rs：參數前處理](../docs/diagrams/core-37.svg)

`run()` 先收集 `args_os`，保留非 UTF-8 路徑的可能性。將舊拼寫 `-all`、`-bp`、`-sp` 轉成 canonical 長參數，再處理 combine archive 位置，最後由 clap 解析。

有 version flag 就印 `CARGO_PKG_VERSION`；有 subcommand 就分派；沒有則印 help。**目前 `mavc version` 沒有對應 subcommand**，只能依實作使用 `--version`、`-V` 或 `-v`；舊 README 的 `version` 說法已過時。

`normalize_combine_archive_position()` 找到 `play` 和 `-c/--combine`，在後面的值找第一個「不是 option、也不是 usize」的項目，移到 play 後當 archive。這是避開 variadic IDs 吞掉 archive 路徑的相容處理，但採啟發式辨識，純數字檔名可能無法判斷。學習及腳本建議用 archive 在前的清晰語法。

## 13. `src/cli/args.rs`：宣告命令列介面

![圖解：13. src/cli/args.rs：宣告命令列介面](../docs/diagrams/core-38.svg)

`Cli` derive Parser，包含 version flag 和可選 Commands；`Commands` derive Subcommand，讓 clap 產生解析和 help。每個 `Args` struct 描述該命令的值。

- `PackageArgs` 同時容納音訊清單模式和 manifest 模式；模式衝突由 commands.rs 再檢查。
- `MusicArgs` 要求 `-m` 至少一個檔，`-a` 可選。`LegacyMusicArgs` 供頂層 `-m` 快捷入口。
- `ManifestArgs` 固定 manifest 和 output 兩個位置參數。
- `AddArgs`：來源檔至少一個，`--to` archive 至少一個；隱藏入口也支援 `--add`、`-+`。
- `RemoveArgs`：ID 至少一個，`--from` archive 至少一個；不是舊文件說的至少兩個。
- `ArchiveArgs`：只有 archive 路徑，供 list 和 pick 重用。
- `PlayArgs`：random 與 one 互斥；all 與 random/one 互斥；combine 至少兩個 ID，與 random/one/all 互斥；system 與 browser 互斥。browser 另拒絕 random/one，並兼容拼錯的 `broswer-player`。
- `InspectArgs`：第一個字串必填、第二個可選，以支援 archive/id 任一順序。
- `WeightArgs`：archive 加 `ID=WEIGHT` 字串，數值解析由 commands 處理。
- `ExtractArgs`：有 all 就不能有 track_id/output，沒有 all 就必須有 track_id。

clap 的約束不代表後續分派一定使用每個值。例如 system playlist 分支先於單選，`--system-player --one 0` 雖可解析，仍導出全部曲目。

## 14. `src/cli/commands.rs`：每個命令如何執行

![圖解：14. src/cli/commands.rs：每個命令如何執行](../docs/diagrams/core-39.svg)

`run` 的 match 把命令對接 library：

- Package 選擇清單或 manifest，拒絕混用及不完整的 manifest 參數。
- Music 與 MusicShortcut 共用 `create_from_music_list`；Create 使用 `create_from_manifest`。
- Add/Remove 呼叫批次函式，成功後逐個印 Updated。
- List 開封存後呼叫 output 模組；Inspect 組一至兩個字串交給 inspect_archive。
- Pick 只印抽中的 ID/title/file_name；不播放。
- Weight 用 `split_once('=')` 分成 ID 和 weight，再解析為 usize/f64，更新並印舊新值。
- Extract all 先檢查所有 basename、忽略大小寫的檔名重複和既有目的檔，再逐個抽取；單首省略 output 時使用來源 basename。單首沒有防覆寫檢查。
- Play 把整份 PlayArgs 交給 player。

`create_from_manifest()` 讀 JSON，將相對 track.path 接在 manifest 所在目錄下，再打包。CLI 這層才做路徑重定位，直接呼叫 library 不會自動找 manifest 目錄。

`create_from_music_list()` 建 builder，只在有 output 時設定 output，建立後印來源數。

## 15. `src/cli/output.rs`：格式化終端輸出

![圖解：15. src/cli/output.rs：格式化終端輸出](../docs/diagrams/core-40.svg)

`print_track_list` 印封存名稱和曲目數，再印 ID/title/codec/weight。`inspect_archive` 若只有一個參數則列全部；有兩個時，第一個能解析為 usize 就當 ID，否則第二個當 ID。

`print_track_details` 印基本資訊與 optional metadata；duration 從毫秒轉成分秒，技術欄位附上單位。`print_optional` 只過濾 `None`，沒有統一過濾空字串。

## 16. `src/cli/player.rs`：真正的聲音與外部播放器

![圖解：16. src/cli/player.rs：真正的聲音與外部播放器](../docs/diagrams/core-41.svg)

### `play_random` 的分支優先順序

![圖解：play_random 的分支優先順序](../docs/diagrams/core-42.svg)

雖然函式名叫 random，它負責所有 CLI 播放模式：browser → combine → system → all → 指定或隨機單曲。讀懂順序就能知道參數組合的實際效果。

單曲時用 builder 取得 bytes，Rodio Decoder 接上 data、byte_len、codec hint 與 gapless 設定，再開預設 sink、連 Player、append、sleep_until_end。sink 必須在播放期間存活，否則音訊輸出也會失去生命週期。

### `play_combined` 與 `play_all_in_order`

![圖解：play_combined 與 play_all_in_order](../docs/diagrams/core-43.svg)

combine 共用一個 output mixer，為每首建立 Player，先全部 append 並保存，再逐個等待。等待迴圈是依序等待已在播放的 Player，不是依序才開始播放；但建立和啟動有時間差，不保證取樣級同步。

all 則每次建一首 Player、播放並等結束，再做下一首，因此是順序播放，不是整份預載的無縫 playlist。

### 資訊 helper

![圖解：資訊 helper](../docs/diagrams/core-44.svg)

`print_current_track` 顯示封存與歌曲資訊，metadata.title 優先；`print_optional` 過濾空字串，`format_duration` 毫秒轉分秒。

### `import_system_playlist`

![圖解：import_system_playlist](../docs/diagrams/core-45.svg)

拒絕空封存 → 建暫存目錄 → 每首抽成 `track-ID.codec` → 建立 M3U8 清單 → 呼叫系統開啟指令。這條路徑確實產生音訊副本，和直接 reader 播放不同。

### `open_browser_playlist`

![圖解：open_browser_playlist](../docs/diagrams/core-46.svg)

同樣抽取全部，為每首產生含標籤、`<audio controls>` 的 HTML section，組成 player.html，再交給預設瀏覽器。瀏覽器是否支援 codec 由其環境決定，不由 Rodio 決定。

### `open_browser_combine`

![圖解：open_browser_combine](../docs/diagrams/core-47.svg)

抽出指定曲目，產生 Play together 按鈕。JavaScript 建 AudioContext，把每個 media element 接到 destination；按鈕先 resume context、將時間歸零，再 `Promise.all(audio.play())`。這是瀏覽器多來源播放，不是將多首重新編碼成單一檔。

### HTML 和平台 helper

![圖解：HTML 和平台 helper](../docs/diagrams/core-48.svg)

`track_info_html` 組標籤列表及播放器；`track_file_name` 用 ID/codec 產生名字。`create_temp_dir` 使用 PID+時間建目錄；`escape_html` 處理 `&<>"'`，避免標題直接被當 HTML markup。

`open_with_system_player` 與 `open_in_default_browser` 依 cfg 在 macOS 用 `open`、Windows 用 `explorer.exe`、其他 Unix 用 `xdg-open`，其他平台回傳 Unsupported。使用 Command 直接傳參數，沒有 shell 字串串接；`.spawn()` 成功只代表子程序啟動，不保證外部播放器播放成功。暫存目錄目前沒有清理機制。

## 17. 動手學習

![圖解：17. 動手學習](../docs/diagrams/core-49.svg)

在此目錄執行，請使用自己準備的真正音訊，並使用新的輸出檔名：

```sh
cargo test --offline
cargo run --offline -- --help
cargo run --offline -- --version
cargo run --offline -- package -m /path/to/a.mp3 /path/to/b.flac -a lesson.mavc
cargo run --offline -- list lesson.mavc
cargo run --offline -- inspect lesson.mavc 0
cargo run --offline -- pick lesson.mavc
cargo run --offline -- weight lesson.mavc 0=0.7
cargo run --offline -- play lesson.mavc --one 0
cargo run --offline -- play lesson.mavc --combine 0 1
```

`--offline` 要求依賴已快取；初次未快取的環境需先正常取得依賴。播放需要音訊輸出設備。增刪、weight 會修改封存，練習請使用副本。

建議練習：先用程式讀前 22 bytes，驗算 JSON 長度和 payload 起點；再用 `[1,2,1]` 權重重複 pick，觀察長期比例；比較 details 和 weight 兩種更新；最後追蹤 CLI 一次 extract 從參數到 io::copy 的完整路徑。


### Remove and export / 移除並匯出

```sh
mavc do -x 0 1 --from archive.mavc --to ./export
mavc do --remove 0 --from a.mavc b.mavc --to ./export ./backup
```

`--to` is optional. Songs are exported using their original filenames before removal.
Export directories are created as needed. Existing files and duplicate output names cause an error,
leaving the archives unchanged. All exports must succeed before removal begins.
Use `--from` for source archives and `--to` for export directories.

`--to` 可省略；指定後會先以原檔名匯出歌曲，全部匯出成功後才移除。
自動建立目錄；遇到已有檔案、輸出檔名衝突或匯出失敗時，保留歸檔中的歌曲。
