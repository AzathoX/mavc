# MAVC mutiple audio version collection

[English](README.md) | **简体中文**

您好！您是否也遇到过同一首歌有多个版本的情况？例如中英文版本、不同歌手
的翻唱，或不同编曲版本。每个版本可能需要几分钟才能听完，因此把所有版本
都听一遍需要不少时间。MAVC 可以将相关版本打包到同一个 `.mavc` 文件中，
并在播放时随机选择其中一首，让每次收听都有更多变化。

MAVC 是一个使用 Rust 编写的音乐归档库和命令行工具，支持 MP3、FLAC 和 AAC。
每个归档包含描述信息、MVLC 音乐索引和原始音频数据。曲目权重用于控制加权
随机播放时各曲目被选中的频率。MAVC 目前处于 1.0 版本完善階段。这也是一个
学习和研究的过程；未来会继续探索支持更多播放器的可能性。

## 命令行帮助

运行 `mavc --help`、`mavc -h` 或 `mavc help` 可显示以下命令列表。

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
  mavc [-+|--add] <音频文件>... --to <归档.mavc>...
  mavc [-x|--remove] <曲目编号>... --from <归档.mavc>...
  mavc list <archive.mavc>
  mavc play [-r|--random | -o|--one <track-id> | -c|--combine <track-id> <track-id>...] [--all] [-sp|--system-player | -bp|--browser-player] <archive.mavc>
  mavc inspect <archive.mavc> [track-id]
  mavc inspect <track-id> <archive.mavc>
  mavc pick <archive.mavc>
  mavc weight <archive.mavc> <track-id>=<weight>
  mavc extract <archive.mavc> <track-id> [output-file]
```

## 命令说明

- **`help`、`--help`、`-h`** — 显示命令用法。
- **`version`、`--version`、`-V`、`-v`** — 显示 MAVC 版本。
- **`package -m <文件>... [-a <输出.mavc>]`** — 将音频文件打包成归档，初始权重均分（每首为 `1 / 曲目数`）。省略 `-a` 时，单个文件生成 `<文件名>.mavc`，多个文件生成 `music.mavc`。
- **`package <manifest.json> <输出.mavc>`** — 按 JSON 清单中的曲目信息打包。
- **`music -m <文件>... [-a <输出.mavc>]`** — 打包音乐列表的快捷命令，初始权重均分（每首为 `1 / 曲目数`）。`-m` 和 `-a` 选项可以按任意顺序排列。
- **`create <manifest.json> <输出.mavc>`** — 使用清单打包的别名命令。
- **`--add` / `-+ <文件>... --to <归档>...`** — 向每个归档添加音频文件；更新后的归档最多包含 5 首曲目，音频数据总量不超过 150 MB。新曲目权重为 1。
- **`--remove` / `-x <曲目编号>... --from <归档>...`** — 从每个归档删除至少两个指定编号的曲目。

归档限制从当前工作目录的 `mavc` 读取。文件不存在时，默认最多 5 首、150 MB。配置示例：

```toml
[limits]
max_tracks = 5
max_size_mb = 150
```

`max_size_mb` 使用十进制 MB（1 MB = 1,000,000 字节），限制的是音频数据总量。
- **`list <归档.mavc>`** — 列出曲目编号、格式和权重。
- **`play [-r|--random] <归档.mavc>`** — 按曲目权重随机选择，并使用 MAVC 内置播放器播放。随机播放为默认模式。
- **`play -o|--one <曲目编号> <归档.mavc>`** — 播放指定编号的曲目。
- **`play --all <归档.mavc>`** — 使用 MAVC 内置播放器按归档顺序播放全部曲目。
- **`play -c|--combine <曲目编号>... <归档.mavc>`** — 同时混音播放至少两首指定曲目。添加 `-bp` 可在浏览器中混音播放；添加 `-sp` 会显示警告并改用 MAVC 内置播放器。
- **`play -sp|--system-player <归档.mavc>`** — 导出全部曲目到临时歌单，并用系统关联的播放器打开。与 `--combine` 同用时会显示警告并改用 MAVC 内置播放器。
- **`play -bp|--browser-player <归档.mavc>`** — 将归档中的曲目和 HTML 选择页面保存到临时文件夹，再用默认浏览器打开。浏览器可能会阻止自动播放，此时请点击播放按钮。
- **`inspect <归档.mavc> [曲目编号]`** 或 **`inspect <曲目编号> <归档.mavc>`** — 显示归档信息和可用曲目元数据，例如标题、歌手、专辑、流派、年份、时长、比特率、采样率、声道数和位深。缺失的标签不会显示。
- **`pick <归档.mavc>`** — 按权重随机选择一首曲目并显示信息，但不播放。
- **`weight <归档.mavc> <曲目编号>=<权重>`** — 将指定曲目的随机播放权重设为 0 到 1，再将剩余权重平均分给其他曲目。例如，4 首曲目中将一首设为 `0.4`，其他每首会设为 `0.2`。设为 `0` 时该曲目不会被随机选中；设为 `1` 时其他曲目权重均为 `0`。
- **`extract <归档.mavc> <曲目编号> [输出文件]`** — 提取曲目。若未指定输出文件名，MAVC 会使用曲目原有文件名和扩展名。

`-r`/`--random` 和 `-o`/`--one` 不能同时使用。浏览器播放器也兼容旧拼写
`--broswer-player`。如果旧归档的 MVLC 索引中没有缓存元数据，`inspect` 会从
音频数据中读取。

## 归档格式

版本 1 使用小端序整数。22 字节的文件头包含 `MAVC` 标记、归档版本（`u16`）、
描述信息长度（`u32`）、MVLC 索引长度（`u32`）和音频数据长度（`u64`）。文件头
之后依次为 JSON 格式的描述信息、索引以及拼接后的音频数据。索引中的曲目偏移量
以音频数据区起始位置为基准。目前归档格式和 MVLC 索引版本均为 `1`。

### 代码结构

```mermaid
flowchart TD
    CLI["src/cli<br/>参数、命令分派、输出和播放"] --> API["src/api<br/>高阶 Rust builder"]
    CLI --> CORE["src/core<br/>归档操作与格式实现"]
    API --> CORE
    CORE --> MODEL["数据模型 + 元数据"]
    CORE --> FORMAT["格式 + 归档读取器"]
    CORE --> CONFIG["配置 + 错误类型"]
```

`mavc` 重新导出公开类型和操作，因此调用者可以使用
`mavc::Archive`、`mavc::create_archive` 或更高阶的 `mavc::music()` builder，
不必依赖内部模块路径。

### 归档布局

```mermaid
flowchart LR
    H["文件头 · 22 字节<br/>MAVC · 版本 · 描述长度<br/>索引长度 · 音频数据长度"]
    D["描述 JSON<br/>长度由文件头记录"]
    I["MVLC 索引 JSON<br/>长度由文件头记录"]
    P["音频数据区<br/>按曲目顺序拼接原始音频字节"]
    H --> D --> I --> P
```

索引中的每首曲目都有 `offset` 和 `length`。`offset` 相对于音频数据区起点，
因此曲目在文件中的绝对位置为：

```text
payload_start = 22 + description_length + index_length
track_position = payload_start + track.offset
```

索引还保存曲目编号、显示名称、原始文件名、编码格式、权重和缓存的音频元数据。
描述和索引都是 UTF-8 JSON；音频区保存原始编码后的音频字节，不做转码。这个字节
布局与编程语言无关，其他语言的实现只要遵循相同的字段长度、字节序、JSON 字段和
偏移量规则，就可以读写 MAVC 文件。

### 创建流程

```mermaid
flowchart TD
    C["CLI music 命令或 Rust music() 调用者"] --> B["music().from(files).output(path)"]
    B --> M["CreateManifest<br/>描述信息 + 曲目来源"]
    M --> V["校验输入<br/>分配编号、权重和偏移量"]
    V --> J["将描述信息与 MVLC 索引序列化为 JSON"]
    J --> W["写入 22 字节文件头和 JSON 区段"]
    W --> A["按曲目顺序追加原始音频字节"]
    A --> F[".mavc 归档"]
```

公开的 `music()` builder 会先准备清单，再调用与 CLI 相同的核心归档写入器。写入器
先记录相对于音频区起点的曲目偏移量，再将各来源文件流式写入音频区。

## Windows

请先使用 rustup 安装 Rust，然后在项目目录的 PowerShell 中运行：

```powershell
cargo build --release
.\target\release\mavc.exe --help
.\target\release\mavc.exe package -m .\song.mp3 .\song.flac -a .\music.mavc
.\target\release\mavc.exe play -r .\music.mavc
```

内置播放器通过 Windows 默认音频设备播放；`-sp` 和 `-bp` 会通过 Windows 文件
关联打开音频文件或浏览器页面。

## 桌面应用前端（`mavc-app`）

`mavc-app` 是 MAVC 的桌面应用。用户界面使用 Rust 和 Dioxus 编写，并运行在
Tauri 2 WebView 中。前端通过带类型的桥接层调用 Tauri 命令；桌面命令层再将归档
操作交给本地 `mavc` Rust 库。

### 功能介绍

- **打开与管理归档：** 加载 `.mavc` 归档、重新载入、返回主界面，并向当前归档
  添加音频文件。
- **浏览与查看曲目：** 在曲目清单中选择曲目，查看封面、标题、歌手、专辑、流派、
  年份、权重及可用的刮削信息。
- **编辑曲目信息：** 修改标题、歌手、专辑、流派、年份、封面图片 URL 和播放权重，
  也可以从归档中移除曲目。
- **加权播放：** 按权重随机播放、按归档顺序播放，或从指定曲目开始播放。曲目权重
  会影响随机选择概率。
- **合奏播放：** 勾选曲目清单中的曲目并同时播放，每个音源曲目都有独立的进度控制。
- **播放控制：** 暂停或继续播放、拖动进度条定位，并让合奏中的单首曲目前进或后退
  0.5 秒。
- **导出归档：** 从音频文件创建归档，或将当前归档导出到指定位置。
- **切换外观与语言：** 可选择 Apple 或经典真夜主题，并使用繁体中文、简体中文、
  英语或日语界面。

归档和曲目的所有操作都由本机 MAVC 函式库处理。

### 前端目录结构

```text
mavc-app/
├── src/
│   ├── app.rs                 # App 模块连接
│   ├── app/
│   │   ├── frontend.rs        # Dioxus 视图与界面事件绑定
│   │   ├── logic.rs           # 响应式应用状态与派生视图数据
│   │   ├── bridge.rs          # 从 WebView 调用 Tauri 的类型化接口
│   │   ├── models.rs          # 前端数据传输类型
│   │   ├── i18n.rs            # 繁体中文、简体中文、英语和日语
│   │   ├── formatting.rs      # 曲目信息与显示格式处理
│   │   └── playback.rs        # WebView 音频控制
│   └── main.rs               # Dioxus 应用入口
├── assets/styles.css         # 主题与界面样式
└── src-tauri/
    └── src/commands.rs       # 使用 mavc 库的桌面命令
```

`frontend.rs` 负责播放器、归档列表、曲目信息和归档工具的界面。`logic.rs` 管理
共享的响应式状态，并计算视图所需的数据。`bridge.rs` 将带类型的前端请求转换为
Tauri 命令调用；`src-tauri/src/commands.rs` 处理桌面访问并调用 MAVC 库。界面提供
Apple 和经典真夜主题，以及繁体中文、简体中文、英语和日语。

### 开发与打包桌面应用

请安装 Rust、Dioxus CLI（`dx`）、Tauri CLI（`cargo tauri`）以及当前操作系统所需的
原生构建依赖。在 `mavc-app` 目录运行：

```sh
cargo tauri dev
```

要为当前操作系统生成发布包，运行：

```sh
cargo tauri build
```

Tauri 配置会通过 `dx serve --port 1420 --interactive false` 启动 Dioxus 开发服务，
并使用 `dx bundle --release` 构建前端。

## 许可证

MAVC 使用 Apache License 2.0 授权，详见 [LICENSE](LICENSE)。
