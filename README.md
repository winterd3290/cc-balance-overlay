# CC Balance Overlay

<p align="center">
  <strong>Windows 11 taskbar-side balance monitor for Claude and Codex users on CC Switch.</strong>
</p>

<p align="center">
  <a href="#中文">中文</a> ·
  <a href="#english">English</a>
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white">
  <img alt="Windows 11" src="https://img.shields.io/badge/Windows-11-0078D4?logo=windows11&logoColor=white">
  <img alt="CC Switch" src="https://img.shields.io/badge/CC%20Switch-supported-4B7BEC">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
</p>

<p align="center">
  <code>C $17.7</code><br>
  <code>X $88.0</code>
</p>

> A tiny native Rust utility that keeps your current Claude and Codex relay balance visible next to the Windows tray, without opening a terminal or browser tab.

---

## 中文

**CC Balance Overlay** 是一个 Windows 11 桌面小工具，用来在任务栏系统托盘旁边实时显示当前 CC Switch 正在使用的 Claude / Codex 中转站余额。

它的目标很简单：像看右下角时间一样，随时看到余额。

### 为什么做它

如果你经常在 Claude Code、Codex、CC Switch 和多个中转站之间切换，余额通常藏在网页、面板或工具菜单里。等到请求失败才发现余额不足，体验很糟。

这个工具会自动读取 CC Switch 当前选择的 provider，并把余额以一个极轻量的双行文本显示在任务栏托盘区域旁边。

### 功能特性

- **自动跟随 CC Switch**：读取当前 Claude / Codex provider，切换中转站后自动使用新的 provider。
- **实时余额显示**：按固定间隔刷新余额，并保留最近一次成功结果作为兜底显示。
- **任务栏旁显示**：不是普通托盘图标，而是在系统托盘区域左侧放置紧凑双行文本。
- **接近系统时间样式**：双行、紧凑、低干扰，适合长期常驻。
- **鼠标悬浮提示**：悬浮时显示当前 Claude / Codex provider 名称。
- **右键快速设置**：支持字号、文字颜色、Claude 前缀、Codex 前缀、开机启动和退出。
- **本地优先**：配置保存在本机，不依赖额外云服务。
- **Rust + Win32 原生实现**：轻量、低资源占用，启动后无命令行窗口。

### 预览

当前仓库建议在第一次 GitHub Release 前添加真实截图：

```text
Windows taskbar
┌──────────────────────────────────────────────────────────────┐
│                                             C $17.7  20:45    │
│                                             X $88.0  2026/5/24│
└──────────────────────────────────────────────────────────────┘
```

建议截图路径：

```text
docs/assets/taskbar-preview.png
```

添加截图后，可以把下面这一行放回 README 顶部预览区：

```md
![CC Balance Overlay taskbar preview](docs/assets/taskbar-preview.png)
```

### 快速开始

#### 方式一：下载 Release

项目发布后，从 GitHub Releases 下载 `cc-balance-overlay.exe`，双击运行即可。

> 当前仓库如果还没有 Release，请先使用源码构建方式。

#### 方式二：从源码构建

准备环境：

- Windows 11
- Rust stable
- 可用的 Windows Rust toolchain：MSVC 或 GNU MinGW-w64
- 已配置好的 CC Switch

构建：

```powershell
git clone https://github.com/wgd-12138/cc-balance-overlay.git
cd cc-balance-overlay
cargo test
cargo build --release
.\target\release\cc-balance-overlay.exe
```

如果你的本机使用 GNU toolchain：

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu test --lib
cargo +stable-x86_64-pc-windows-gnu build --release
.\target\release\cc-balance-overlay.exe
```

### 配置

用户设置会保存在：

```text
%APPDATA%\cc-balance-overlay\settings.toml
```

示例：

```toml
font_size = 13
color = "#FFFFFF"
claude_prefix = "C"
codex_prefix = "X"
```

多数设置可以通过右键设置面板即时修改，无需手动编辑配置文件。

### 它如何工作

CC Balance Overlay 会只读访问本机 CC Switch 数据：

- `~\.cc-switch\settings.json`
- `~\.cc-switch\cc-switch.db`

它会读取：

- `currentProviderClaude`
- `currentProviderCodex`
- provider metadata 中的 `usage_script`

然后调用当前 provider 配置里的余额接口，格式化为任务栏旁的两行余额文本。

### 隐私与安全

- 本项目不提供也不经过任何中转服务。
- 本项目不会把你的 key、provider 信息或余额上传到第三方服务器。
- 余额查询只会请求你在 CC Switch provider 中配置的接口。
- CC Switch 配置和本工具配置都保留在本机。
- 调试时请避免把包含密钥的 `settings.json`、数据库或日志上传到公开 issue。

### 已知限制

- 当前主要面向 Windows 11。
- 需要 CC Switch provider 提供可用的 `usage_script` 余额查询配置。
- 不同中转站的余额接口可能存在差异，适配可能需要持续补充。
- 多显示器、不同 DPI、任务栏位置等场景仍需要更多真实设备验证。

### 路线图

- [ ] 发布正式 GitHub Release
- [ ] 添加真实任务栏截图和演示 GIF
- [ ] 提供安装器或便携版压缩包
- [ ] 增强多显示器 / DPI / 任务栏位置适配
- [ ] 增加更多 provider 余额接口兼容规则
- [ ] 改进文本渲染清晰度和系统时间样式一致性
- [ ] 添加自动更新或版本检查

### 贡献

欢迎提交 issue 和 pull request，尤其是：

- 新 provider 的余额接口适配
- Windows 任务栏 / DPI / 多显示器兼容性反馈
- UI 细节截图对比
- 安装、打包、发布流程改进
- README 翻译和文档优化

如果这个工具正好解决了你的痛点，欢迎点一个 Star。它会帮助更多 Claude / Codex / CC Switch 用户发现这个项目。

---

## English

**CC Balance Overlay** is a small Windows 11 desktop utility that shows the current Claude and Codex relay balance selected by CC Switch, right next to the system tray.

The goal is intentionally simple: check your AI relay balance as casually as you check the clock.

### Why

If you use Claude Code, Codex, CC Switch, and multiple relay providers, balance information is often hidden in dashboards, web pages, or provider panels. Discovering that your balance is empty only after a request fails is frustrating.

This app follows the active CC Switch providers and renders a compact two-line balance overlay beside the Windows tray.

### Features

- **Follows CC Switch automatically**: reads the currently selected Claude and Codex providers.
- **Live balance display**: refreshes periodically and keeps the last successful value as fallback.
- **Taskbar-side overlay**: compact text beside the tray area instead of a terminal window.
- **Clock-like layout**: two-line, low-distraction display inspired by the native Windows time/date block.
- **Hover tooltip**: shows the current Claude and Codex provider names.
- **Right-click settings**: font size, text color, Claude prefix, Codex prefix, startup toggle, and quit.
- **Local-first**: settings stay on your machine.
- **Native Rust + Win32**: lightweight desktop app with no console window on launch.

### Preview

Add a real screenshot before the first public GitHub release:

```text
Windows taskbar
┌──────────────────────────────────────────────────────────────┐
│                                             C $17.7  20:45    │
│                                             X $88.0  2026/5/24│
└──────────────────────────────────────────────────────────────┘
```

Suggested screenshot path:

```text
docs/assets/taskbar-preview.png
```

After adding the screenshot, place this near the top of the README:

```md
![CC Balance Overlay taskbar preview](docs/assets/taskbar-preview.png)
```

### Quick Start

#### Option 1: Download a Release

Once published, download `cc-balance-overlay.exe` from GitHub Releases and run it.

> If this repository has no Release yet, build from source.

#### Option 2: Build from Source

Requirements:

- Windows 11
- Rust stable
- A working Windows Rust toolchain: MSVC or GNU MinGW-w64
- A configured CC Switch installation

Build:

```powershell
git clone https://github.com/wgd-12138/cc-balance-overlay.git
cd cc-balance-overlay
cargo test
cargo build --release
.\target\release\cc-balance-overlay.exe
```

For GNU toolchain users:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
cargo +stable-x86_64-pc-windows-gnu test --lib
cargo +stable-x86_64-pc-windows-gnu build --release
.\target\release\cc-balance-overlay.exe
```

### Configuration

User settings are stored at:

```text
%APPDATA%\cc-balance-overlay\settings.toml
```

Example:

```toml
font_size = 13
color = "#FFFFFF"
claude_prefix = "C"
codex_prefix = "X"
```

Most settings can be changed immediately from the right-click settings panel.

### How It Works

CC Balance Overlay reads local CC Switch state in read-only mode:

- `~\.cc-switch\settings.json`
- `~\.cc-switch\cc-switch.db`

It uses:

- `currentProviderClaude`
- `currentProviderCodex`
- `usage_script` metadata from the provider table

Then it calls the configured provider balance endpoint and renders the result as a compact two-line taskbar-side overlay.

### Privacy & Security

- This project does not operate a relay, proxy, or cloud service.
- It does not upload your keys, provider details, or balances to any third-party server.
- Balance requests go only to the endpoint configured in your CC Switch provider metadata.
- CC Switch data and CC Balance Overlay settings remain local.
- Please avoid posting secret-bearing `settings.json`, database files, or logs in public issues.

### Limitations

- Currently focused on Windows 11.
- Requires CC Switch providers with usable `usage_script` balance metadata.
- Provider balance APIs may differ and may need adapter updates.
- More real-device testing is needed for multi-monitor, DPI, and taskbar-position edge cases.

### Roadmap

- [ ] Publish the first GitHub Release
- [ ] Add real taskbar screenshots and a demo GIF
- [ ] Provide an installer or portable zip package
- [ ] Improve multi-monitor / DPI / taskbar-position handling
- [ ] Add more provider balance adapter rules
- [ ] Improve text rendering fidelity against the native Windows clock
- [ ] Add auto-update or version checking

### Contributing

Issues and pull requests are welcome, especially for:

- New provider balance adapters
- Windows taskbar / DPI / multi-monitor compatibility reports
- UI comparison screenshots
- Packaging and release improvements
- README translation and documentation polish

If this tiny tool saves you from one failed request at the worst possible moment, consider giving it a Star so more Claude / Codex / CC Switch users can find it.

## License

MIT
