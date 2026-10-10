# malbum：桌面 App 前後端程式碼學習指南

本文件取代原本的 Tauri/Dioxus 模板說明，按目前原始碼逐檔、逐函式與事件區塊解說。閱讀快照：2026-10-03。請先讀 [整體導讀與 review](../README.learning.zh-TW.md)，封存格式細節見 [mavc 核心指南](../mavc/README.md)。

## 2026-10-10 啟動與檔案開啟更新

- 中文介面名稱為簡體「专辑馆」、繁體「專輯館」，英文及日文介面名稱為 **malbum**；安裝包名稱為 malbum，MAVC 仍是歸檔格式名稱。
- 啟動時讀取系統語言，支援簡體中文、繁體中文、英文及日文；不支援或無法偵測時使用英文。瀏覽器預覽使用瀏覽器語言。
- 前端、桌面後端及安裝包版本為 `0.1.0`，本機 mavc 函式庫為 `0.5.0`，分別版本化。介面顯示實際版本，啟動時檢查前後端版本是否一致。
- 音訊選檔格式與 lib 共用 MP3、FLAC、AAC、WAV、OGG 清單；匯出僅提供已實作的 MAVC 格式。相容性測試涵蓋五種格式的建立、加入、移除及前後端 JSON 資料解析。
- 預設使用「經典真夜」主題，仍可手動切換 Apple。
- `index.html` 的金色懷錶啟動畫面使用內嵌 CSS／SVG，在 WASM 之前顯示。確認主樣式、初始檔案請求及畫面排版準備完成後，才顯示介面；15 秒後提供重試，減少動態效果設定下停止旋轉。
- 安裝包宣告 `.mavc` 檔案關聯。macOS 透過系統 Opened 事件接收檔案；Windows／Linux 接收啟動參數；單一實例插件將第二次啟動的檔案送給原視窗。
- 檔案請求先進入後端佇列，前端先訂閱事件再取出請求，避免冷啟動漏接。讀取歸檔在背景工作中執行，失敗顯示錯誤並保留原本已開啟的歸檔。
- 作業系統須註冊新安裝包，Finder／檔案總管的關聯才生效；已有其他預設程式時，依系統的「打開方式／預設應用程式」設定選擇 malbum。

```sh
cargo check -p mavc-app-ui --target wasm32-unknown-unknown --offline
cargo test -p mavc-app --lib --offline
cargo test -p mavc-app-ui --offline
dx bundle --release
cargo tauri build --bundles app --ci -- --offline
```

人工驗收：冷啟動看懷錶後進入經典真夜；切換四種語言確認名稱；應用關閉及已開啟兩種狀態雙擊含空白／中文路徑的有效 `.mavc`；再用損壞檔案確認保留原歸檔並顯示錯誤。Windows／Linux 檔案關聯需在各平台安裝包驗證。

## 1. 為什麼有兩個 Rust 入口？

![圖解：1. 為什麼有兩個 Rust 入口？](../docs/diagrams/app-01.svg)

```text
src/main.rs                   → Dioxus → WebAssembly → WebView 介面
src-tauri/src/main.rs          → Tauri → 本機 Rust → 檔案與系統 API
src-tauri/Cargo.toml 的 mavc   → ../../mavc → 核心函式庫
```

介面端也是 Rust，但編譯目標是瀏覽器的 WASM，操作 DOM/HTML audio；它不直接使用 std::fs 去打開使用者電腦上的音樂。本機端才執行檔案操作，再透過 Tauri IPC 回傳資料。

點擊播放時，不是前端去執行 `mavc play` 命令，也不是 Tauri 用 Rodio 出聲。桌面 app 先抽出音訊暫存檔，回傳 asset URL，再由 WebView 的 `<audio>` 播放。

## 2. Cargo 與建置設定

![圖解：2. Cargo 與建置設定](../docs/diagrams/app-02.svg)

### `Cargo.toml`

![圖解：Cargo.toml](../docs/diagrams/app-03.svg)

package 名是 `mavc-app-ui`，edition 2024。Dioxus 的 web feature 建立 WebView 前端；dioxus-logger 記錄日誌；wasm-bindgen 提供 JavaScript 互通；web-sys 對應 DOM/audio API；serde-wasm-bindgen 在 JS value 與 Rust DTO 之間轉換。其他依賴包含 wasm-bindgen-futures、js-sys、serde 和 serde_json。

`[workspace] members = ["src-tauri"]` 讓本機 crate 成為同一 workspace 成員。核心 `mavc` 是路徑依賴，並不在這份 members 裡。app 的 Cargo.lock 被此目錄 .gitignore 忽略，因此不同環境重新解析依賴可能得到不同結果。

### `src-tauri/Cargo.toml`

![圖解：src-tauri/Cargo.toml](../docs/diagrams/app-04.svg)

本機 package 名是 `mavc-app`，library 名是 `mavc_app_lib`，crate-type 支援 staticlib/cdylib/rlib。Tauri 啟用 protocol-asset；dialog plugin 開原生檔案對話框，opener plugin 提供系統開啟能力；mavc 是核心依賴，lofty 用於封面，base64 用於圖片 data URL。

末尾 `[profile.release]` 指定 codegen-units、LTO、opt-level、panic=abort、strip，但它放在 workspace 子 package；實際 Cargo check 明確警告這組 profile 被忽略，應搬到 workspace 根的 Cargo.toml。不能以此處宣告推論打包結果已做所有優化。

### `Dioxus.toml`

![圖解：Dioxus.toml](../docs/diagrams/app-05.svg)

指定 web 為預設 platform、dist 為輸出目錄、assets 為靜態資產，頁面 title 為「專輯館 · MAVC」。watcher 監看 src/assets，開發時重新載入 HTML。

### `src-tauri/tauri.conf.json`

![圖解：src-tauri/tauri.conf.json](../docs/diagrams/app-06.svg)

`beforeDevCommand` 啟動 `dx serve --port 1420 --interactive false`；`devUrl` 指向 localhost:1420；release 前執行 `dx bundle --release`，讀取 `../dist/public`。

`withGlobalTauri = true` 將 Tauri API 放到 window，供 bridge 的 `window.__TAURI__.core.invoke` 使用。視窗預設 1280×900、最小 940×720。bundle 定義產品名稱、identifier 和圖示。

security 區塊目前 `csp: null`；assetProtocol 開啟，但 scope 初始為空。播放 helper 對產生的音訊路徑逐一 allow_file，才能經 asset protocol 讀取。

### `src-tauri/capabilities/default.json`

![圖解：src-tauri/capabilities/default.json](../docs/diagrams/app-07.svg)

capability 指定 main 視窗，包含 core:default 和 opener:default。後端已安裝 dialog plugin，且透過 Rust command 開對話框；這和前端直接呼叫 dialog plugin 的 JS API 是不同路徑。不要把「沒列 dialog permission」直接推論成後端 picker 必然不可用，也不要把 capability 當成所有自訂 command 內的路徑驗證。

### `src-tauri/build.rs`、圖示與忽略檔

![圖解：src-tauri/build.rs、圖示與忽略檔](../docs/diagrams/app-08.svg)

build.rs 只呼叫 `tauri_build::build()` 產生建置所需資源。icons 中 PNG/ICO/ICNS 是不同平台尺寸；SVG 是可編輯圖形來源。前端 assets 的 dioxus.png/tauri.svg 是模板資產，當前 frontend.rs 直接引用的是 styles.css，沒有直接引用這兩個圖片。

`.gitignore` 忽略 dist/target/Cargo.lock；src-tauri 的 .gitignore 忽略 target 和產生 schema；`.taurignore` 排除前端 src/assets/Cargo.toml 等打包監看項目。IDE extensions.json 是編輯器建議，不是執行期依賴。

## 3. 啟動流程

![圖解：3. 啟動流程](../docs/diagrams/app-09.svg)

`src/main.rs` 載入 app::App，以 INFO 初始化 logger，`launch(App)` 建立 Dioxus 介面；初始化日誌失敗時 expect 會中止。

`src/app.rs` 宣告 bridge、formatting、frontend、i18n、logic、models、playback，並將 frontend::App 重新公開。

`src-tauri/src/main.rs` 在 release Windows 隱藏額外 console window，然後呼叫 mavc_app_lib::run。`src-tauri/src/lib.rs` 的 run 建 Tauri Builder → 註冊 opener/dialog → generate_handler 註冊 19 個 command → generate_context 載入設定 → 啟動 event loop，失敗時 expect 中止。新增 command 只加函式不夠，還需加進 handler 清單。

## 4. `src/app/models.rs`：前端 DTO

![圖解：4. src/app/models.rs：前端 DTO](../docs/diagrams/app-10.svg)

DTO 是跨前後端傳送的資料物件，不等同封存磁碟上的模型。

- `ArchiveInfo` 有 path/title/creator/description/tracks，外層使用 camelCase 命名。
- 前端 `Track` 只保留畫面需要的 id/title/file_name/weight/cover_art_url/embedded_cover_data_url/metadata，未包含 codec、offset、length；Serde 默認可忽略額外欄位。
- `Metadata` 是核心 metadata 的顯示子集合，不含 track_number/comment。
- `PlaybackData` 有 archive_title、track、src，外層使用 camelCase，因此 JSON 是 archiveTitle。
- `MixedSource` 只存在前端狀態，保存一首合奏音訊的 id/title/src/position_seconds/duration_seconds，不經 IPC 反序列化。

**命名要逐層看**：Track 和 Metadata 沒有 rename_all，仍讀 snake_case。後端 TrackInfo flatten 核心 Track 時，原本 file_name 等保持 snake_case；只有新增封面欄位變成 embeddedCoverDataUrl。前端卻讀 embedded_cover_data_url，且有 default，所以會默默變 None。這已用實際前端 Track 型別重現，並不是整個 Track 都無法解碼。

建議只對前端封面欄位加明確 rename/alias，或使用專門 IPC DTO；不要為了解決一個欄位而直接修改所有封存欄位命名。

## 5. `src/app/bridge.rs`：前端如何呼叫後端

![圖解：5. src/app/bridge.rs：前端如何呼叫後端](../docs/diagrams/app-11.svg)

extern 宣告用 wasm-bindgen 綁定 `window.__TAURI__.core.invoke`，catch 讓 JS rejection 轉成 Result。

`command<T: DeserializeOwned>` 的流程：記錄 name/args → JSON value 轉成 JsValue → await invoke → 成功記日誌、失敗把 JS error 轉字串 → 回傳值反序列化成 T；解碼失敗也記 console error。

例如：

```rust,ignore
command::<ArchiveInfo>("archive_list", serde_json::json!({"path": path})).await
command::<PlaybackData>("archive_play_track", serde_json::json!({"path": path, "trackId": 0})).await
```

Tauri 參數名的 trackId 對應後端 track_id；回傳物件內每一層的欄位名由其 Serde attributes 決定，兩者不要混為一談。泛型 T 是呼叫端承諾的資料型別，型別檢查不能跨過 IPC 自動驗證後端 JSON schema。

在普通瀏覽器只開 dx 前端時，window.__TAURI__ 不存在；可看介面不代表後端命令能執行。console 目前會印完整 args，包含路徑和編輯的 metadata。

## 6. `src/app/formatting.rs`：三個小 helper

![圖解：6. src/app/formatting.rs：三個小 helper](../docs/diagrams/app-12.svg)

`display_title`：非空 metadata.title 優先，否則用 track.title。回傳借用字串，避免不必要複製。

`format_time_seconds`：NaN、無限、負值顯示 `--:--`；正常值 floor，再以分鐘和兩位秒數顯示。分鐘不限於 59，因此不是時分秒格式。

`optional`：trim 表單文字，空字串變 None，其他變 Some。清空 artist 等欄位時因此會存 null，而不是空文字。

## 7. `src/app/playback.rs`：操作 DOM audio

![圖解：7. src/app/playback.rs：操作 DOM audio](../docs/diagrams/app-13.svg)

`seek_combined_track` 依 `mavc-combined-audio-ID` 找元素，轉成 HtmlAudioElement，再把 currentTime 加偏移量。先限制不小於 0，如果 duration 有限再限制不超過 duration。

`seek_audio_to` 接受任意 element ID 和絕對秒數，用相同方式定位與限制。

`set_audio_playing` 找元素，再執行 play 或 pause。找不到元素、轉型失敗都直接回傳；play 的 Promise 被忽略，前端也沒有等待 rejection。這使圖示先更新時可能與真實播放狀態不一致。

這三個 helper 沒有控制本機 Rodio，也沒有修改音訊檔，是對 WebView 的 DOM 操作。

## 8. `src/app/i18n.rs`：語言

![圖解：8. src/app/i18n.rs：語言](../docs/diagrams/app-14.svg)

`Language` 支援簡中、繁中、日文、英文；Copy 允許簡單複製 enum。`code()` 映射語言碼，`from_code()` 未識別時回繁中。

`tr(language, traditional)` 以繁中文字串作 key，繁中直接回傳，其餘各自 match 翻譯，沒命中的 key 仍回繁中。大量 match arm 是相同機制的翻譯資料，不是每行不同演算法。

目前仍有未經 tr 的固定文字和 format! 狀態訊息，例如編輯、表單 placeholder、曲目數、播放提示。切換語言只改 signal，不會回頭重翻已存成 String 的 status；不存在的翻譯和舊 status 都可能保留繁中。增加功能時宜使用穩定 message key 與參數化訊息，避免文案改字就找不到翻譯。

## 9. `src/app/logic.rs`：狀態與顯示模型

![圖解：9. src/app/logic.rs：狀態與顯示模型](../docs/diagrams/app-15.svg)

### `AppState`：持久、可變、會觸發畫面更新

![圖解：AppState：持久、可變、會觸發畫面更新](../docs/diagrams/app-16.svg)

Signal 是 Dioxus 的反應式狀態 handle；read 讀值，set 寫值。這里分為：

- 偏好：language、apple_theme。
- 封存：archive_path、archive_info。
- 選取：selected_track 是詳情焦點；combine_ids 是勾選集合；playback_selection 是開始播放時的勾選快照。
- 聲音來源：audio_src 是單首 URL；combined_sources 是多首來源；audio_nonce 用來強制單曲 audio 重新建立。
- 播放：is_paused、current_time、total_time、playback_mode。
- 訊息：status、status_error。
- 表單：editing_track 與 edit_title/artist/album/genre/year/cover/weight。

詳情選取與真正播放中的曲目目前共用 selected_track，是後面播放 all 問題的根源。

### `use_app_state`

![圖解：use_app_state](../docs/diagrams/app-17.svg)

依穩定順序建立所有 use_signal，再組成 AppState。預設繁中、Apple 主題、沒有封存與音訊，模式是 idle，表單空白。語言與主題未保存到磁碟；重新啟動會回預設。

hook 不是普通可以任意放在 if/loop 裡的初始化函式；保持呼叫順序讓重渲染能對應正確狀態。

### `AppViewModel`、`derive_view_model`

![圖解：AppViewModel、derive_view_model](../docs/diagrams/app-18.svg)

先取 lang/info/selected_id，從 archive tracks 找 selected；然後計算目前標題、artist fallback、album、genre/year，以及 bitrate/sample rate/channels/bit depth 的顯示文字。

接著算時間和 `[0,100]` 進度；total 非有限或非正時進度為 0。依來源存在與 pause 狀態决定按鈕標題和 `Ⅱ/▶`。最後回傳 AppViewModel，不做 IPC 或寫檔。

selected_for_edit 是 selected 的複本，供事件 closure 使用；selected_track_id 在無 selected 時缺省為 0，但編輯/移除按鈕在 selected 存在的區塊中才顯示。`detail_scrape` 名字雖叫 scrape，實際顯示已讀取的技術 properties，沒有網路刮削。

## 10. `src/app/frontend.rs`：按畫面區塊讀 755 行 App

![圖解：10. src/app/frontend.rs：按畫面區塊讀 755 行 App](../docs/diagrams/app-19.svg)

此檔只有一個大型 `App()` component。開頭引入 helper、用 asset! 取得 CSS，建立 AppState、計算 AppViewModel，再解構 signal handle。後面 rsx! 是 HTML 類型的宣告式畫面，事件 closure 可以讀写 signal，`spawn(async move {...})` 執行 IPC。

### A. 外框與 sidebar，約第 65–202 行

![圖解：A. 外框與 sidebar，約第 65–202 行](../docs/diagrams/app-20.svg)

main class 依主題切 theme-apple/classic。品牌與 nav 是靜態 UI，href 指頁內 player/library。

載入封存後遍歷 tracks：複製 track_id 給事件 closure；勾選按鈕增刪 combine_ids，UI 最多五首；曲目按鈕只設定 selected_track 和關閉編輯，**不直接播放**。

清除選取會清 combined_sources 與 combine_ids。載入新曲目流程是 pick_audio → archive_add → archive_list → 更新資訊和 status。此流程使用點擊時捕获的 archive_path。

重新載入：若已載入就使用 info.path，否則開 picker；讀 archive_list 後更新資訊並清詳情焦點與勾選，但沒有統一清掉播放來源。取消 picker 直接 return，也可能留下「正在重新載入」訊息。

退出按鈕完整清封存、選取、音訊、時間、模式、表單並增加 nonce。其他重設按鈕沒有共用此流程，狀態清理程度不同。

### B. topbar、狀態訊息與裝飾，約第 204–285 行

![圖解：B. topbar、狀態訊息與裝飾，約第 204–285 行](../docs/diagrams/app-21.svg)

語言下拉修改 Language；外觀下拉修改 apple_theme。status 非空時顯示 banner，status_error 決定 error class。

watch-stage/chain/case/rim/face 是懷錶造型，0..60 loop 每格用 `--turn: mark*6deg` 產刻度，每五格 major。SVG 描繪五線譜和音符，配合 CSS，不是使用者音訊的頻譜或真正進度表。more-button 只清 archive_info/audio_src，沒有清合奏和其他狀態，需注意舊來源殘留。

### C. 單曲進度與 audio，約第 286–372 行

![圖解：C. 單曲進度與 audio，約第 286–372 行](../docs/diagrams/app-22.svg)

不在合奏時顯示 range slider；輸入百分比乘 total_time，seek 主 audio 並更新 current_time。

有 audio_src 才建 audio，key 用 audio_nonce，換 key 讓新來源重新掛載。autoplay 開啟、原生 controls 關閉：

- timeupdate 從 DOM audio 讀 currentTime，變化至少 0.1 秒才更新 signal。
- loadedmetadata 讀真實 duration 和位置。
- error 設解碼失敗訊息。
- play/pause 更新 is_paused。
- ended：all 模式從 selected_track 在 tracks 的位置找下一首，再 IPC 取得新 URL；到尾清來源。單曲模式直接清來源。

all 下一首使用的是**詳情焦點 selected_track**，不是獨立的正在播放 ID。播放中點另一首看資訊，當目前音訊結束就可能跳錯下一首；這個結論來自事件邏輯，未做 GUI 操作重現。

### D. 四個 transport 按鈕，約第 373–522 行

![圖解：D. 四個 transport 按鈕，約第 373–522 行](../docs/diagrams/app-23.svg)

隨機按鈕清合奏、設定 single，呼叫 archive_play_track，trackId=null；成功後更新 selected ID、時間、src、nonce、status。

主播放按鈕讀 checked_ids：

1. 超過五首就報錯。
2. 已有聲音來源且 checked_ids 等於 playback_selection：依 is_paused 對單首或全部 audio 暫停/繼續。
3. 選取改變時停止舊來源，保存新 playback_selection。
4. 恰一首 → 指定 trackId 播單曲。
5. 至少兩首 → archive_play_combine，將回傳 PlaybackData 轉成 MixedSource，再清單首來源。
6. 沒勾選 → 隨機單曲；詳情中高亮的曲目不决定這條路徑。

依序全部按鈕取第一首 ID，設 all 再播放，由 ended 串接後續。系統播放器按鈕清 WebView 音訊，再呼叫 archive_system_playlist。

目前沒有 operation/request token。很快點兩次播放或退出時，較早 IPC 的晚到結果仍可能寫回 signal，覆蓋較新的操作。建議集中播放狀態並加入請求世代檢查。

### E. 合奏音訊，約第 523–588 行

![圖解：E. 合奏音訊，約第 523–588 行](../docs/diagrams/app-24.svg)

遍歷 MixedSource，顯示獨立進度、range slider 和 ±0.5 秒按鈕。每首有一個 `mavc-combined-audio-ID` audio，autoplay=true；timeupdate/loadedmetadata 以 ID 找到 state 的來源並更新。

這是多個 audio 元素各自啟動，不是共用取樣時鐘的精準同步。每首都未加 ended handler，結束後 combined_sources 仍非空，主按鈕仍可能呈現可暫停的播放狀態。

### F. 初始製作與導入，約第 589–648 行

![圖解：F. 初始製作與導入，約第 589–648 行](../docs/diagrams/app-25.svg)

製作：呼叫 archive_create_music_from_picker，成功載入回傳的 ArchiveInfo；取消不做事；特定「至少兩首」英文錯誤轉成目前語言。

導入：archive_pick_existing → archive_list → 設 path/info、清詳情焦點、顯示成功。初始畫面沒有文字路徑輸入建立流程；後面的 path 欄位也是 readonly。

### G. 導出與曲目詳情，約第 650–755 行

![圖解：G. 導出與曲目詳情，約第 650–755 行](../docs/diagrams/app-26.svg)

導出按鈕把 archive_path 作為 source，呼叫 export picker，成功回報路徑。沒有封存時 disable。後端接受 .mvsc，但只是複製同樣位元組。

selected 存在才顯示詳情。圖片來源優先 embedded_cover_data_url，再用 cover_art_url。點編輯把現值複製到表單；input 寫各自 signal；儲存把 weight parse 成 f64，optional 清理文字，呼叫 details 更新，再 reload list。

前端這里只檢查是否 parse 成功，有限值與 `[0,1]` 由核心拒絕。儲存修改的是索引，不是音訊 tags。取消只關閉表單。

移除：以 selected ID 呼叫 archive_remove，reload，再清 selected；沒有同步移除 combine_ids/playback_selection 中的舊 ID，因此刪掉已勾選曲目後仍可能傳出不存在 ID。

footer 是固定版本文案，並未呼叫 app_version。`archive_inspect`、manifest 建立、weight、extract 等已註冊 commands 在當前 UI 沒有專屬入口；command 存在不代表使用者能在畫面操作到。

## 11. `src-tauri/src/commands.rs`：本機端逐函式

![圖解：11. src-tauri/src/commands.rs：本機端逐函式](../docs/diagrams/app-27.svg)

### 回傳型別與共用 helper，第 20–91 行

![圖解：回傳型別與共用 helper，第 20–91 行](../docs/diagrams/app-28.svg)

`ArchiveInfo` 是封存概況，tracks 使用 `TrackInfo`。TrackInfo flatten 核心 Track，再加 embedded cover；PlaybackData 則直接含核心 Track 和 URL。flatten 會攤平欄位，但不遞迴改變內部 Track 的 snake_case。

`paths()` 把 Vec<String> 轉 Vec<PathBuf>。

`load_archive()` 開封存、複製每首，透過 track_metadata 取快取/補讀。只有 codec=flac 才另外用 lofty 讀圖片，取第一個 picture，取得 MIME，base64 編碼成 data URL。MP3/AAC 的嵌入封面目前未實作。這個 helper 每次 reload 都遍歷，沒有封面快取。

### 查詢與建立，第 94–159 行

![圖解：查詢與建立，第 94–159 行](../docs/diagrams/app-29.svg)

- `archive_list(path)`：回 load_archive 的 info。
- `archive_inspect(path,track_id)`：先讀全部，若有 ID 再 retain 該曲，找不到回傳錯誤。
- `archive_create_music(files,output)`：拒绝空清單，忽略空白 output，走 music builder，回訊息字串。
- `archive_create_music_from_picker(app)`：開多選 picker，取消回 Ok(None)，要求至少兩個，將選中路徑轉換，builder 使用默認 `music.mavc`，再讀新 archive 回 Some(info)。沒有另問輸出位置，也沒有檔名避碰。

picker filter 列 mp3/flac/wav/m4a/aac/ogg/opus；核心只收前三者中的 mp3/flac/aac，因此其餘格式可被選中卻打包失敗。

### picker 與編輯/導出，第 161–255 行

![圖解：picker 與編輯/導出，第 161–255 行](../docs/diagrams/app-30.svg)

- `archive_pick_existing` 只選 mavc 檔，回路徑或取消。
- `archive_pick_audio` 選一個音訊，同樣有過寬 filter。
- `archive_update_track_details` 只是把參數交核心，再把 Error 轉 String；沒有另改音訊標籤。
- `archive_export_picker` 設預設檔名和 mavc/mvsc filter，驗證副檔名，再 std::fs::copy。mvsc 沒有另一套轉換格式；應另外防止 source/destination 是同一路徑或相同實體檔。

這幾個 async picker 內部用 blocking_pick_*，以及部分後續同步 I/O；async 關鍵字不會自動把阻塞工作搬到背景執行緒。

### 建立、增刪、抽選、權重、抽取，第 257–367 行

![圖解：建立、增刪、抽選、權重、抽取，第 257–367 行](../docs/diagrams/app-31.svg)

- `archive_create_manifest` 讀 JSON、以 manifest parent 重定位相對來源、呼叫 create_archive，回建立數量。
- `archive_add` 將字串清單轉路徑，再批次 add；回音訊與封存數。
- `archive_remove` 批次 remove，回請求 ID 數（不是去重後實際刪除總數）。
- `archive_pick` 開 archive、加權抽選並 cloned，沒有正權重回傳錯誤。
- `archive_weight` 使用會重分配其他權重的 update_track_weight；目前編輯表單沒有使用它。
- `archive_extract_one` 按 ID 抽到指定 output，沒有防覆寫檢查。
- `archive_extract_all` 拒絕空封存，先檢查 basename 重複和目的檔存在，再 create_dir_all，逐首抽取。前置檢查避免一般誤覆寫，但不提供整批回滾，也不是 create_new 的競態保證。

### 播放和 asset URL，第 371–487 行

![圖解：播放和 asset URL，第 371–487 行](../docs/diagrams/app-32.svg)

`archive_play_track` 使用 spawn_blocking：選指定或隨機 reader → 再開 archive 取 title → playback_asset_url 寫暫存音訊 → 回傳 PlaybackData。await join 的錯誤轉 String。

`archive_play_combine` 也 spawn_blocking：取 archive title → combine.streams() → 每首依次寫暫存檔、組 PlaybackData → collect。至少兩首的限制由核心處理，最多五首只由 UI 檢查。

`playback_asset_url`：建 temp_dir/mavc-playback → 用 PID/ID/timestamp/codec 命名 → File::create 和 io::copy → 關檔 → allow_file → 取得 main window → convert_file_src → 回傳可播放 URL。這裡的 scope 動態授權的是個別檔案，不是整個使用者磁碟。

目前無暫存音訊清理、額度或失敗清除；每次播放都可能再寫一份。長期使用應建立生命週期管理，而不是在 audio 還可能 seek 時提早刪檔。

`archive_system_playlist`：開 archive → 拒絕空 tracks → 建 PID/timestamp 暫存目錄 → extract 每首 → 寫 M3U8 → 用 OS command 開啟。和 CLI 類似，也沒有後續清理。

### 版本與平台，第 489–521 行

![圖解：版本與平台，第 489–521 行](../docs/diagrams/app-33.svg)

`app_version` 回本機 crate `0.1.0`，不是核心 `0.5.0`；timestamp 取 Unix epoch 的奈秒，時鐘錯誤時回 0。`open_with_system_player` 用 cfg 按平台選 open/explorer.exe/xdg-open，其他平台回 Unsupported。沒有 shell 字串拼接，但 spawn 成功不表示播放器支持該清單或 codec。

## 12. `assets/styles.css`：按樣式層閱讀

![圖解：12. assets/styles.css：按樣式層閱讀](../docs/diagrams/app-34.svg)

CSS 共 846 行，主要是反覆覆蓋的樣式，不是 846 個獨立功能。讀它的關鍵是 cascade：後面的同等 specificity 規則覆蓋前面；`theme-apple` 和 `:has(...)` 的較高 specificity 也影響勝出結果。

- 第 1–64 行：root variables、reset、sidebar/workspace/topbar、grid 版面基礎。
- 第 66–91 行：懷錶結構，absolute 定位、z-index、圓形邊框；tick 的 rotate 使用前端 --turn，transform-origin 決定旋轉中心。
- 第 93–149 行：歌曲資訊、舊進度樣式、transport、卡片、曲目列和 footer；部分 class 已沒有對應現有 markup。
- 第 150–179 行：1000/760/520px 響應式斷點。
- 第 181–298 行：深色 midnight palette 覆蓋 root 和各元件，偽元素星點背景在 prefers-reduced-motion 條件中啟用。
- 第 300–338 行：酒紅與古金色覆蓋懷錶和按鈕。
- 第 340–410 行：曲目詳情、封面、表單、合奏和工具類樣式，有些是舊 UI 的殘留。
- 第 412–545 行：Apple 主題，以 `:root:has(.app-shell.theme-apple)` 將主題 class 的影響傳到 root/body，改配色、字體、卡片和圖形。
- 第 547–581 行：Apple 小畫面補充、language/theme picker。
- 第 583–656 行：classic 主題的 Gothic 覆蓋。
- 第 658–712 行：隱藏原生 audio，改用 range slider；WebKit/Firefox thumb 分開設定；--progress 控制滑軌著色；focus-visible 支援鍵盤焦點。
- 第 714–784 行：topbar、初始製作/導入按鈕。
- 第 786–846 行：sidebar 新曲目清單、滾動/省略、grid/status 與末尾響應式覆蓋。

例如 `.audio-player` 前段設 height/width，後段 `display:none` 才決定目前不顯示原生 controls。Apple 對 `.content-grid` 有較高 specificity，不能只看末尾通用 `.content-grid` 就認為它一律覆蓋 Apple 版面。

CSS 含模板/舊介面 class，如部分 command-grid、quote-card 等；刪除前應查對 frontend.rs 的實際引用。圖示 SVG 和 class 決定視覺，沒有把真正音訊進度自動綁到懷錶旋轉。

## 13. 一次完整操作的呼叫鏈

![圖解：13. 一次完整操作的呼叫鏈](../docs/diagrams/app-35.svg)

```text
新增音樂：onclick
 → bridge command("archive_pick_audio") → 後端 dialog
 → command("archive_add") → mavc::add_tracks → rewrite_archive
 → command("archive_list") → load_archive → JSON → 前端 ArchiveInfo → signal.set

播放：onclick
 → command("archive_play_track") → spawn_blocking
 → mavc::play().archive(...).one(...).stream()
 → playback_asset_url → 暫存檔 + asset URL
 → 前端 audio_src.set → <audio src=...> → WebView 解碼/出聲
 → timeupdate/loadedmetadata → signal → 進度與時間文字
```

## 14. 建置、驗證與練習

![圖解：14. 建置、驗證與練習](../docs/diagrams/app-36.svg)

從 repository 根執行：

```sh
cargo check --offline --manifest-path mavc-app/Cargo.toml --target wasm32-unknown-unknown -p mavc-app-ui
cargo check --offline --manifest-path mavc-app/src-tauri/Cargo.toml
```

本次兩個 check 都成功，並出現 release profile 被忽略的警告。這不等於 GUI、播放、打包全部測過。依赖需先快取，WASM target 需已安裝。

開發時從 mavc-app 目錄執行 `cargo tauri dev`（需要已安裝 Tauri CLI 和 dx）；Tauri 會依設定啟動 dx。單独 `dx serve` 只提供前端，不提供桌面 IPC。本次沒有啟動 GUI 或重新安裝工具。

練習順序：先追導入從 picker 到 signal；再比較 selected_track/combine_ids/playback_selection；在 bridge 檢查 IPC payload 欄位；最後讀 audio events 和後端 asset URL，理解「Rust 寫檔 → WebView 播放」跨界流程。review 的修正建議留在整體導讀，本次文件工作未修改產品邏輯。
