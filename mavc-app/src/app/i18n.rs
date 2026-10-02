//! UI language selection and translations.

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Language {
    SimplifiedChinese,
    TraditionalChinese,
    Japanese,
    English,
}

impl Language {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::SimplifiedChinese => "zh-CN",
            Self::TraditionalChinese => "zh-TW",
            Self::Japanese => "ja",
            Self::English => "en",
        }
    }

    pub(crate) fn from_code(code: &str) -> Self {
        match code {
            "zh-CN" => Self::SimplifiedChinese,
            "ja" => Self::Japanese,
            "en" => Self::English,
            _ => Self::TraditionalChinese,
        }
    }
}

pub(crate) fn tr(language: Language, traditional: &'static str) -> &'static str {
    match language {
        Language::TraditionalChinese => traditional,
        Language::SimplifiedChinese => match traditional {
            "專輯館" => "专辑馆",
            "外觀" => "外观",
            "經典真夜" => "经典真夜",
            "工作區" => "工作区",
            "正在聆聽" => "正在聆听",
            "隨機播放 / 合奏" => "随机播放 / 合奏",
            "暫停播放" => "暂停播放",
            "繼續播放" => "继续播放",
            "專輯收藏" => "专辑收藏",
            "重新載入" => "重新加载",
            "退出，回到主界面" => "退出，返回主界面",
            "加權音樂封存" => "加权音乐归档",
            "尚未載入封存" => "尚未加载归档",
            "讓旋律慢一點" => "让旋律慢一点",
            "製作音樂館" => "制作音乐馆",
            "導入音樂館" => "导入音乐馆",
            "選擇至少兩個音訊檔" => "请选择至少两个音频文件",
            "已建立 MAVC 封存：" => "已创建 MAVC 归档：",
            "已載入 MAVC 封存：" => "已加载 MAVC 归档：",
            "合奏不能多於五首歌曲。" => "合奏不能多于五首歌曲。",
            "NOW PLAYING" => "正在播放",
            "尚未選擇歌曲" => "尚未选择歌曲",
            "載入封存後顯示歌曲資訊" => "加载归档后显示歌曲信息",
            "載入一個 .mavc 封存即可開始聆聽。" => "加载一个 .mavc 归档即可开始聆听。",
            "播放清單已播放完畢。" => "播放列表已播放完毕。",
            "請先載入一個封存。" => "请先加载一个归档。",
            "未標記演出者" => "未标记演出者",
            "未標記" => "未标记",
            "曲目清單 · LIST" => "曲目列表 · LIST",
            "選取作為合奏曲目" => "选择作为合奏曲目",
            "這個封存目前沒有曲目。" => "此归档目前没有曲目。",
            "載入封存後，歌曲會列在這裡。" => "加载归档后，歌曲会显示在这里。",
            "可直接輸入多個曲目 ID（例如 1, 1, 3）" => {
                "可输入多个曲目 ID（例如 1, 1, 3）"
            }
            "合奏至少需要兩個曲目 ID；可在清單勾選或直接輸入 ID。" => {
                "合奏至少需要两个曲目 ID；可在列表勾选或直接输入 ID。"
            }
            "正在同步播放所選曲目。" => "正在同步播放所选曲目。",
            "合奏所選歌曲" => "合奏所选歌曲",
            "清除選取" => "清除选择",
            "音訊檔路徑（每行一個）" => "音频文件路径（每行一个）",
            "輸出封存路徑（可留空自動命名）" => "输出归档路径（可留空自动命名）",
            "或使用 JSON manifest" => "或使用 JSON manifest",
            "Manifest 建立需要 JSON 路徑和輸出封存路徑。" => {
                "Manifest 创建需要 JSON 路径和输出归档路径。"
            }
            "加入音訊檔（每行一個）" => "添加音频文件（每行一个）",
            "移除曲目 ID（逗號分隔）" => "移除曲目 ID（逗号分隔）",
            "請輸入有效的曲目 ID 和 0 到 1 之間的權重。" => {
                "请输入有效的曲目 ID 和 0 到 1 之间的权重。"
            }
            "請輸入有效的曲目 ID。" => "请输入有效的曲目 ID。",
            "找不到該曲目；請先載入封存。" => "找不到该曲目；请先加载归档。",
            "請輸入整批抽取的輸出目錄。" => "请输入批量提取的输出目录。",
            "載入封存會列出全部曲目與權重。點選曲目可在上方播放器播放，使用右側勾選框可加入合奏。" => {
                "加载归档会列出所有曲目和权重。点击曲目可在播放器播放，勾选曲目可加入合奏。"
            }
            "開啟 MAVC 封存" => "打开 MAVC 归档",
            "Explore · 導出封存" => "Explore · 导出归档",
            "導出" => "导出",
            "已導出封存：" => "已导出归档：",
            "請先載入 MAVC 封存。" => "请先加载 MAVC 归档。",
            "目前的播放器無法解碼這種音訊格式。" => {
                "当前播放器无法解码这种音频格式。"
            }
            "載入" => "加载",
            "輸入封存檔案路徑以載入曲目。" => "输入归档文件路径以加载曲目。",
            "點選一首播放；勾選多首可使用 --combine。" => {
                "点击歌曲播放；勾选多首可使用 --combine。"
            }
            "所有封存操作皆由本機 mavc 函式庫執行" => {
                "所有归档操作均由本地 mavc 库执行"
            }
            "建立封存" => "创建归档",
            "管理曲目" => "管理曲目",
            "抽取歌曲" => "提取歌曲",
            "檢視與抽選" => "查看与抽选",
            "命令工具箱" => "命令工具箱",
            "將 MAVC CLI 的建立、管理、檢視與抽取功能集中在這裡。" => {
                "在这里集中使用 MAVC CLI 的创建、管理、查看和提取功能。"
            }
            "讀取 manifest 並建立　↗" => "读取 manifest 并创建　↗",
            "目標封存路徑（每行一個）" => "目标归档路径（每行一个）",
            "預設使用上方已載入封存" => "默认使用上方已加载的归档",
            "加入封存　＋" => "添加到归档　＋",
            "移除目標封存（留空使用已載入封存）" => {
                "要移除曲目的归档（留空使用已加载归档）"
            }
            "移除指定曲目　−" => "移除指定曲目　−",
            "曲目 ID" => "曲目 ID",
            "權重 0–1" => "权重 0–1",
            "更新權重" => "更新权重",
            "抽取單曲" => "提取单曲",
            "輸出檔路徑（可留空使用原始檔名）" => {
                "输出文件路径（留空则使用原始文件名）"
            }
            "抽取此歌曲　↓" => "提取此歌曲　↓",
            "或抽取整個封存" => "或提取整个归档",
            "專輯：" => "专辑：",
            "曲風：" => "曲风：",
            "年份：" => "年份：",
            "格式：" => "格式：",
            "取樣率：" => "采样率：",
            "由音訊檔建立　→" => "从音频文件创建　→",
            "抽取全部歌曲　⇩" => "提取全部歌曲　⇩",
            "檢視封存資訊　⌕" => "查看归档信息　⌕",
            "依權重抽選歌曲　✦" => "按权重抽选歌曲　✦",
            "隨機播放" => "随机播放",
            "播放選中歌曲" => "播放选中歌曲",
            "依序播放全部" => "按顺序播放全部",
            "系統播放器播放清單" => "系统播放器播放列表",
            "抽取整個封存" => "提取整个归档",
            "抽取整批輸出目錄" => "批量提取输出目录",
            "權重" => "权重",
            "MAVC PLAYER" => "MAVC 播放器",
            "載入封存並選擇歌曲後，播放器會顯示在這裡。" => {
                "加载归档并选择歌曲后，播放器会显示在这里。"
            }
            _ => traditional,
        },
        Language::Japanese => match traditional {
            "專輯館" => "ミュージックルーム",
            "外觀" => "外観",
            "經典真夜" => "クラシック・ミッドナイト",
            "工作區" => "ワークスペース",
            "正在聆聽" => "再生中",
            "隨機播放 / 合奏" => "シャッフル / 合奏",
            "暫停播放" => "一時停止",
            "繼續播放" => "再開",
            "專輯收藏" => "ライブラリ",
            "命令工具箱" => "アーカイブツール",
            "重新載入" => "再読み込み",
            "退出，回到主界面" => "終了してホームに戻る",
            "加權音樂封存" => "重み付き音楽アーカイブ",
            "尚未載入封存" => "アーカイブ未読み込み",
            "讓旋律慢一點" => "音楽をゆっくり楽しむ",
            "製作音樂館" => "MAVCを作成",
            "導入音樂館" => "アーカイブを読み込む",
            "選擇至少兩個音訊檔" => "音声ファイルを2つ以上選択してください",
            "已建立 MAVC 封存：" => "MAVCアーカイブを作成しました：",
            "已載入 MAVC 封存：" => "MAVCアーカイブを読み込みました：",
            "合奏不能多於五首歌曲。" => "合奏再生は5曲までです。",
            "NOW PLAYING" => "再生中",
            "尚未選擇歌曲" => "曲が選択されていません",
            "載入封存後顯示歌曲資訊" => "アーカイブを読み込むと曲の情報が表示されます",
            "載入一個 .mavc 封存即可開始聆聽。" => {
                ".mavcアーカイブを読み込んで再生を始めましょう。"
            }
            "播放清單已播放完畢。" => "プレイリストの再生が終了しました。",
            "請先載入一個封存。" => "先にアーカイブを読み込んでください。",
            "未標記演出者" => "アーティスト情報なし",
            "未標記" => "情報なし",
            "曲目清單 · LIST" => "トラックリスト · LIST",
            "選取作為合奏曲目" => "合奏する曲として選択",
            "這個封存目前沒有曲目。" => "このアーカイブには曲がありません。",
            "載入封存後，歌曲會列在這裡。" => {
                "アーカイブを読み込むと曲がここに表示されます。"
            }
            "可直接輸入多個曲目 ID（例如 1, 1, 3）" => {
                "複数のトラックIDを入力できます（例：1, 1, 3）"
            }
            "合奏至少需要兩個曲目 ID；可在清單勾選或直接輸入 ID。" => {
                "合奏には2つ以上のトラックIDが必要です。リストから選択するか、IDを入力してください。"
            }
            "正在同步播放所選曲目。" => "選択した曲を同期再生しています。",
            "合奏所選歌曲" => "選択した曲を合奏再生",
            "清除選取" => "選択を解除",
            "音訊檔路徑（每行一個）" => "音声ファイルのパス（1行に1つ）",
            "輸出封存路徑（可留空自動命名）" => {
                "出力アーカイブのパス（空欄で自動命名）"
            }
            "或使用 JSON manifest" => "またはJSONマニフェストを使用",
            "Manifest 建立需要 JSON 路徑和輸出封存路徑。" => {
                "マニフェストの作成にはJSONパスと出力アーカイブのパスが必要です。"
            }
            "加入音訊檔（每行一個）" => "追加する音声ファイル（1行に1つ）",
            "移除曲目 ID（逗號分隔）" => "削除するトラックID（カンマ区切り）",
            "請輸入有效的曲目 ID 和 0 到 1 之間的權重。" => {
                "有効なトラックIDと0〜1の重みを入力してください。"
            }
            "請輸入有效的曲目 ID。" => "有効なトラックIDを入力してください。",
            "找不到該曲目；請先載入封存。" => {
                "トラックが見つかりません。先にアーカイブを読み込んでください。"
            }
            "請輸入整批抽取的輸出目錄。" => {
                "一括抽出先のフォルダーを入力してください。"
            }
            "載入封存會列出全部曲目與權重。點選曲目可在上方播放器播放，使用右側勾選框可加入合奏。" => {
                "アーカイブを読み込むと曲と重みが表示されます。曲をクリックすると再生し、チェックボックスで合奏に追加できます。"
            }
            "開啟 MAVC 封存" => "MAVCアーカイブを開く",
            "Explore · 導出封存" => "探索 · アーカイブを書き出す",
            "導出" => "書き出す",
            "已導出封存：" => "アーカイブを書き出しました：",
            "請先載入 MAVC 封存。" => "先にMAVCアーカイブを読み込んでください。",
            "目前的播放器無法解碼這種音訊格式。" => {
                "この音声形式は現在のプレーヤーでデコードできません。"
            }
            "載入" => "読み込む",
            "輸入封存檔案路徑以載入曲目。" => {
                "アーカイブのパスを入力して曲を読み込みます。"
            }
            "點選一首播放；勾選多首可使用 --combine。" => {
                "曲を選択して再生します。複数選択すると --combine を使用できます。"
            }
            "所有封存操作皆由本機 mavc 函式庫執行" => {
                "すべてのアーカイブ操作はローカルのmavcライブラリで実行されます"
            }
            "建立封存" => "アーカイブを作成",
            "管理曲目" => "トラックを管理",
            "抽取歌曲" => "曲を抽出",
            "檢視與抽選" => "確認と抽選",
            "將 MAVC CLI 的建立、管理、檢視與抽取功能集中在這裡。" => {
                "MAVC CLIの作成、管理、確認、抽出機能をまとめて利用できます。"
            }
            "讀取 manifest 並建立　↗" => "マニフェストを読み込んで作成　↗",
            "目標封存路徑（每行一個）" => "対象アーカイブのパス（1行に1つ）",
            "預設使用上方已載入封存" => "上で読み込んだアーカイブを使用",
            "加入封存　＋" => "アーカイブに追加　＋",
            "移除目標封存（留空使用已載入封存）" => {
                "削除対象のアーカイブ（空欄で読み込み済みを使用）"
            }
            "移除指定曲目　−" => "選択した曲を削除　−",
            "曲目 ID" => "トラックID",
            "權重 0–1" => "重み 0–1",
            "更新權重" => "重みを更新",
            "抽取單曲" => "1曲を抽出",
            "輸出檔路徑（可留空使用原始檔名）" => {
                "出力ファイルのパス（空欄で元のファイル名を使用）"
            }
            "抽取此歌曲　↓" => "この曲を抽出　↓",
            "或抽取整個封存" => "またはアーカイブ全体を抽出",
            "專輯：" => "アルバム：",
            "曲風：" => "ジャンル：",
            "年份：" => "年：",
            "格式：" => "形式：",
            "取樣率：" => "サンプルレート：",
            "由音訊檔建立　→" => "音声ファイルから作成　→",
            "抽取全部歌曲　⇩" => "すべての曲を抽出　⇩",
            "檢視封存資訊　⌕" => "アーカイブ情報を確認　⌕",
            "依權重抽選歌曲　✦" => "重みに基づいて曲を抽選　✦",
            "隨機播放" => "シャッフル再生",
            "播放選中歌曲" => "選択した曲を再生",
            "依序播放全部" => "すべて順番に再生",
            "系統播放器播放清單" => "システムプレーヤーでプレイリストを開く",
            "抽取整個封存" => "アーカイブ全体を抽出",
            "抽取整批輸出目錄" => "一括抽出先フォルダー",
            "權重" => "重み",
            "MAVC PLAYER" => "MAVCプレーヤー",
            "載入封存並選擇歌曲後，播放器會顯示在這裡。" => {
                "アーカイブを読み込み、曲を選択するとプレーヤーが表示されます。"
            }
            "載入新曲目" => "新しい曲を読み込む",
            "正在重新載入封存…" => "アーカイブを再読み込みしています…",
            "封存已重新載入" => "アーカイブを再読み込みしました",
            "歌手" => "アーティスト",
            "專輯" => "アルバム",
            "曲風 / 年份" => "ジャンル / 年",
            "刮削資訊" => "メタデータ情報",
            "編輯" => "編集",
            "曲名" => "曲名",
            "封面圖片 URL" => "カバー画像のURL",
            "權重需介於 0 和 1" => "重みは0〜1の範囲で入力してください",
            "曲目資訊已更新" => "曲の情報を更新しました",
            "儲存" => "保存",
            "取消" => "キャンセル",
            "移除曲目" => "曲を削除",
            _ => traditional,
        },
        Language::English => match traditional {
            "專輯館" => "Music Room",
            "外觀" => "Appearance",
            "經典真夜" => "Classic Midnight",
            "工作區" => "Workspace",
            "正在聆聽" => "Now Playing",
            "隨機播放 / 合奏" => "Shuffle / Combine",
            "暫停播放" => "Pause",
            "繼續播放" => "Resume",
            "專輯收藏" => "Library",
            "命令工具箱" => "Archive Tools",
            "重新載入" => "Reload",
            "退出，回到主界面" => "Exit to Home",
            "加權音樂封存" => "Weighted music archive",
            "尚未載入封存" => "No archive loaded",
            "讓旋律慢一點" => "Let the music linger",
            "製作音樂館" => "Make as MAVC",
            "導入音樂館" => "Import Music Room",
            "選擇至少兩個音訊檔" => "Select at least two audio files",
            "已建立 MAVC 封存：" => "Created MAVC archive: ",
            "已載入 MAVC 封存：" => "Loaded MAVC archive: ",
            "合奏不能多於五首歌曲。" => {
                "Combined playback cannot include more than five tracks."
            }
            "NOW PLAYING" => "NOW PLAYING",
            "尚未選擇歌曲" => "No track selected",
            "載入封存後顯示歌曲資訊" => "Track details appear after loading an archive",
            "載入一個 .mavc 封存即可開始聆聽。" => {
                "Load a .mavc archive to start listening."
            }
            "播放清單已播放完畢。" => "The playlist has finished.",
            "請先載入一個封存。" => "Load an archive first.",
            "未標記演出者" => "Artist not listed",
            "未標記" => "Not listed",
            "可直接輸入多個曲目 ID（例如 1, 1, 3）" => {
                "Enter track IDs (for example: 1, 1, 3)"
            }
            "合奏至少需要兩個曲目 ID；可在清單勾選或直接輸入 ID。" => {
                "Combined playback needs at least two track IDs. Select tracks or enter IDs."
            }
            "正在同步播放所選曲目。" => "Playing selected tracks together.",
            "Manifest 建立需要 JSON 路徑和輸出封存路徑。" => {
                "Manifest creation requires a JSON path and an output archive path."
            }
            "請輸入有效的曲目 ID 和 0 到 1 之間的權重。" => {
                "Enter a valid track ID and a weight between 0 and 1."
            }
            "請輸入有效的曲目 ID。" => "Enter a valid track ID.",
            "找不到該曲目；請先載入封存。" => {
                "Track not found. Load an archive first."
            }
            "請輸入整批抽取的輸出目錄。" => {
                "Enter an output directory for batch extraction."
            }
            "載入封存會列出全部曲目與權重。點選曲目可在上方播放器播放，使用右側勾選框可加入合奏。" => {
                "Loading an archive lists its tracks and weights. Select a track to play it, or check tracks for combined playback."
            }
            "開啟 MAVC 封存" => "Open MAVC archive",
            "Explore · 導出封存" => "Explore · Export archive",
            "導出" => "Export",
            "已導出封存：" => "Exported archive: ",
            "請先載入 MAVC 封存。" => "Load a MAVC archive first.",
            "目前的播放器無法解碼這種音訊格式。" => {
                "This audio format cannot be decoded by the current player."
            }
            "載入" => "Load",
            "輸入封存檔案路徑以載入曲目。" => {
                "Enter an archive path to load its tracks."
            }
            "點選一首播放；勾選多首可使用 --combine。" => {
                "Select a track to play; select several to use --combine."
            }
            "所有封存操作皆由本機 mavc 函式庫執行" => {
                "All archive operations run through the local mavc library"
            }
            "建立封存" => "Create archive",
            "管理曲目" => "Manage tracks",
            "抽取歌曲" => "Extract tracks",
            "檢視與抽選" => "Inspect and pick",
            "音訊檔路徑（每行一個）" => "Audio file paths (one per line)",
            "輸出封存路徑（可留空自動命名）" => {
                "Output archive path (leave blank to name automatically)"
            }
            "曲目清單 · LIST" => "Track list · LIST",
            "這個封存目前沒有曲目。" => "This archive has no tracks yet.",
            "載入封存後，歌曲會列在這裡。" => {
                "Tracks will appear here after loading an archive."
            }
            "合奏所選歌曲" => "Play selected tracks together",
            "清除選取" => "Clear selection",
            "將 MAVC CLI 的建立、管理、檢視與抽取功能集中在這裡。" => {
                "Create, manage, inspect, and extract MAVC archives here."
            }
            "或使用 JSON manifest" => "Or use a JSON manifest",
            "讀取 manifest 並建立　↗" => "Load manifest and create　↗",
            "目標封存路徑（每行一個）" => "Target archive paths (one per line)",
            "預設使用上方已載入封存" => "Defaults to the archive loaded above",
            "加入音訊檔（每行一個）" => "Audio files to add (one per line)",
            "加入封存　＋" => "Add to archive　＋",
            "移除曲目 ID（逗號分隔）" => "Track IDs to remove (comma separated)",
            "移除目標封存（留空使用已載入封存）" => {
                "Target archive (blank uses the loaded archive)"
            }
            "移除指定曲目　−" => "Remove selected tracks　−",
            "曲目 ID" => "Track ID",
            "權重 0–1" => "Weight 0–1",
            "更新權重" => "Update weight",
            "抽取單曲" => "Extract one track",
            "輸出檔路徑（可留空使用原始檔名）" => {
                "Output file path (blank uses the original name)"
            }
            "抽取此歌曲　↓" => "Extract this track　↓",
            "或抽取整個封存" => "Or extract the whole archive",
            "選取作為合奏曲目" => "Select for combined playback",
            "專輯：" => "Album: ",
            "曲風：" => "Genre: ",
            "年份：" => "Year: ",
            "格式：" => "Format: ",
            "取樣率：" => "Sample rate: ",
            "由音訊檔建立　→" => "Create from audio files　→",
            "抽取全部歌曲　⇩" => "Extract all tracks　⇩",
            "檢視封存資訊　⌕" => "Inspect archive　⌕",
            "依權重抽選歌曲　✦" => "Pick a weighted track　✦",
            "隨機播放" => "Shuffle",
            "播放選中歌曲" => "Play selected track",
            "依序播放全部" => "Play all in order",
            "系統播放器播放清單" => "Open system playlist",
            "抽取整個封存" => "Extract the entire archive",
            "抽取整批輸出目錄" => "Output directory for batch extraction",
            "權重" => "Weight",
            "MAVC PLAYER" => "MAVC PLAYER",
            "載入封存並選擇歌曲後，播放器會顯示在這裡。" => {
                "Load an archive and select a track to start playback."
            }
            _ => traditional,
        },
    }
}
