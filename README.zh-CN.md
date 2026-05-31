# CC Balance Overlay

<p align="center">
  <strong>在 Windows 11 任务栏旁边常驻显示 Claude / Codex 中转站余额。</strong>
</p>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="https://github.com/wgd-12138/cc-balance-overlay/releases/latest">下载</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#使用教程">使用教程</a>
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
- **全屏自动隐藏**：看电影或玩游戏进入全屏时自动隐藏，退出全屏后自动恢复。
- **启动时检查更新**：启动时检查 GitHub 最新 Release；没有新版时完全无提示。
- **本地优先**：配置保存在本机，不依赖额外云服务。
- **Rust + Win32 原生实现**：轻量、低资源占用，启动后无命令行窗口。

## 快速开始

1. 安装并配置 CC Switch。
2. 确认 CC Switch 中 Claude 和 Codex 当前 provider 带有可用的 `usage_script` 余额查询配置。
3. 从 [最新 Release](https://github.com/wgd-12138/cc-balance-overlay/releases/latest) 下载 `cc-balance-overlay-v1.0.1-windows-x64.zip`。
4. 解压到一个固定目录，比如 `C:\Tools\cc-balance-overlay`。
5. 双击运行 `cc-balance-overlay.exe`。
6. 查看 Windows 系统托盘附近，正常会看到两行余额：

```text
C $17.7
X $88.0
```

默认前缀含义：

- `C`：Claude
- `X`：Codex

你可以在右键设置面板中自定义这两个前缀。

## 使用教程

### 查看余额

显示区域每行对应一个应用：

```text
C $17.7
X $88.0
```

如果某个余额暂时无法读取，会显示 `--`。如果之前成功读取过余额，程序会尽量保留最近一次成功结果作为兜底显示。

### 查看当前中转站

把鼠标移动到余额显示区域上方，会悬浮显示当前 Claude / Codex 正在使用的 provider 名称。频繁切换中转站时，可以用它确认当前扣费来源。

### 全屏看电影和玩游戏

当前台窗口覆盖整个显示器时，余额显示会自动隐藏，并关闭已经打开的悬浮提示或设置面板。退出全屏后会自动恢复显示。

### 打开设置

在余额显示区域上右键，可以打开设置面板。

当前支持：

- **字号**：调整显示区域文字大小。
- **文字颜色**：选择自定义显示颜色。
- **Claude 前缀**：自定义 Claude 行的标签。
- **Codex 前缀**：自定义 Codex 行的标签。
- **开机启动**：控制登录 Windows 后是否自动启动。
- **退出**：关闭程序。

大部分设置调整后会立刻生效。

### 切换中转站

正常在 CC Switch 中切换 Claude 或 Codex provider 即可。CC Balance Overlay 会读取 CC Switch 当前状态并自动跟随，不需要在本工具里手动填写 provider 名称。

### 设置开机启动

右键打开设置面板，启用 **开机启动**。程序会写入当前用户的普通启动项；如果不需要，也可以在同一个设置面板里关闭。

启用开机启动后，请尽量不要移动 exe 所在目录。如果移动了程序位置，重新打开设置面板，把 **开机启动** 关闭再打开一次，让 Windows 记录新的路径。

### 更新行为

程序启动时会在后台检查 GitHub 最新 Release。

- 如果没有新版本，不会有任何提示。
- 如果发现新版本，会弹出一个小窗口提醒。
- 选择 **自动更新** 后会保存偏好，以后检测到新版本会静默下载并更新。
- 选择 **不再提醒** 后会停止检查新版本，也不会自动更新。

如果以后想恢复提醒，可以编辑 `%APPDATA%\cc-balance-overlay\settings.toml`，把 `update_policy` 改回 `"prompt"`。

### 退出程序

右键打开设置面板，点击 **退出**。

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

## 配置文件

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
update_policy = "prompt"
```

多数情况下，建议直接使用右键设置面板，不需要手动编辑这个文件。

## 工作原理

CC Balance Overlay 会以只读方式读取本机 CC Switch 状态：

- `~\.cc-switch\settings.json`
- `~\.cc-switch\cc-switch.db`

它会使用：

- `currentProviderClaude`
- `currentProviderCodex`
- provider 表中的 `usage_script` 元数据

然后调用当前 provider 配置里的余额接口，并把结果渲染为任务栏旁的紧凑余额文本。

全屏隐藏通过 Win32 检测当前前台窗口是否覆盖所在显示器，不改变余额查询和配置格式。

## 搜索记录

- `skills.sh`：没有找到可直接复用的 Windows 任务栏 / 托盘覆盖层方案。
- GitHub / Win32 参考：常见做法是比较前台窗口矩形和显示器矩形。本项目采用这个简单机制，使用 `GetForegroundWindow`、`GetWindowRect`、`MonitorFromWindow`、`GetMonitorInfoW`。

## 常见问题

### 显示 `--` 怎么办？

请检查：

- CC Switch 是否已安装并选择了 Claude / Codex provider。
- 当前 provider 是否启用了 `usage_script` 元数据。
- provider 的余额接口是否能正常访问。
- provider 的 key 或 token 是否仍然有效。

### 余额不对怎么办？

不同中转站的余额接口返回结构可能不一样。可以提交 issue，并附上 provider 类型和脱敏后的返回示例。不要上传真实 key 或 token。

### Windows 提示风险怎么办？

项目是开源的，但当前 exe 还没有代码签名，Windows SmartScreen 可能会在首次运行时提示风险。如果介意，可以自行从源码构建。

### 看不到显示区域怎么办？

可以检查程序进程是否正在运行，或重新启动程序。多显示器、特殊 DPI、特殊任务栏位置等场景可能还需要继续做兼容。

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

- 添加真实任务栏截图和演示 GIF
- 提供安装器
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
