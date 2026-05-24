# CC Balance Overlay

<p align="center">
  <strong>在 Windows 11 任务栏旁边常驻显示 Claude / Codex 中转站余额。</strong>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="https://github.com/wgd-12138/cc-balance-overlay/releases">Releases</a> ·
  <a href="#从源码构建">从源码构建</a>
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

CC Balance Overlay 是一个轻量的 Windows 原生小工具，适合每天使用 Claude、Codex、CC Switch 和多个中转站的人。它会自动跟随 CC Switch 当前选择的 Claude / Codex provider，并把余额以紧凑的双行文本显示在系统托盘旁边。

目标很简单：不用打开网页、终端或中转站后台，也能像看时间一样随时看到余额。

## 为什么需要它

多个中转站之间来回切换时，余额通常藏在网页控制台、面板或工具菜单里。很多时候，只有请求失败了才发现余额不足。

CC Balance Overlay 把这个隐藏状态变成一个可以随时扫一眼的信息块。它不会做复杂仪表盘，也不接管你的账号，只专注显示当前 Claude 和 Codex 的余额。

## 功能特性

- **自动跟随 CC Switch**：读取当前 Claude / Codex provider，切换中转站后自动使用新 provider。
- **实时余额显示**：按固定间隔刷新余额，并保留最近一次成功结果作为兜底。
- **任务栏旁显示**：不是普通托盘图标，而是在系统托盘区域旁显示紧凑双行文本。
- **接近时间日期样式**：低干扰、双行、适合长期常驻。
- **悬浮显示 provider**：鼠标悬浮时显示当前 Claude / Codex provider 名称。
- **右键快速设置**：支持字号、文字颜色、Claude 前缀、Codex 前缀、开机启动和退出。
- **本地优先**：配置保存在本机，不依赖额外云服务。
- **Rust + Win32 原生实现**：轻量、低资源占用，启动后无命令行窗口。

## 安装

正式发布后，可以从 [Releases](https://github.com/wgd-12138/cc-balance-overlay/releases) 下载 `cc-balance-overlay.exe` 直接运行。

当前如果还没有 Release，请先从源码构建。

## 从源码构建

需要：

- Windows 11
- Rust stable
- 可用的 Windows Rust toolchain：MSVC 或 GNU MinGW-w64
- 已配置好的 CC Switch

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

## 配置

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

## 工作原理

CC Balance Overlay 会以只读方式读取本机 CC Switch 状态：

- `~\.cc-switch\settings.json`
- `~\.cc-switch\cc-switch.db`

它会使用：

- `currentProviderClaude`
- `currentProviderCodex`
- provider 表中的 `usage_script` 元数据

然后调用当前 provider 配置里的余额接口，并把结果渲染为任务栏旁的紧凑余额文本。

## 隐私与安全

- 本项目不提供也不经过任何中转服务。
- 本项目不会把你的 key、provider 信息或余额上传到第三方服务器。
- 余额查询只会请求你在 CC Switch provider 中配置的接口。
- CC Switch 配置和本工具配置都保留在本机。
- 不要在公开 issue 中上传包含密钥的 `settings.json`、数据库文件或日志。

## 已知限制

- 当前主要面向 Windows 11。
- 需要 CC Switch provider 提供可用的 `usage_script` 余额查询配置。
- 不同中转站的余额接口可能存在差异，适配可能需要持续补充。
- 多显示器、不同 DPI、任务栏位置等场景仍需要更多真实设备验证。

## 路线图

- 发布正式 GitHub Release
- 添加真实任务栏截图和演示 GIF
- 提供安装器或便携版压缩包
- 增强多显示器、DPI、任务栏位置适配
- 增加更多 provider 余额接口兼容规则
- 改进文本渲染清晰度和系统时间样式一致性
- 添加自动更新或版本检查

## 贡献

欢迎提交 issue 和 pull request，尤其是：

- 新 provider 的余额接口适配
- Windows 任务栏、DPI、多显示器兼容性反馈
- UI 细节截图对比
- 安装、打包、发布流程改进
- 文档优化

如果这个小工具刚好解决了你的痛点，欢迎点一个 Star。它会帮助更多 Claude / Codex / CC Switch 用户发现这个项目。

## License

MIT
