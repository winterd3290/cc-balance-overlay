# ARCHITECTURE

## 文件职责

- `src/main.rs`：程序入口。
- `src/app.rs`：组合设置、余额读取、窗口显示和更新检查。
- `src/overlay.rs`：Win32 余额显示窗口、悬浮提示、右键设置面板、全屏自动隐藏。
- `src/settings.rs`：读写用户设置。
- `src/ccswitch.rs`：只读读取 CC Switch 当前 provider 和余额脚本配置。
- `src/usage.rs`：调用 provider 余额接口并解析余额。
- `src/display.rs`：把余额格式化成两行显示文本。
- `src/updater.rs`：检查 GitHub Release 并执行更新。
- `README.md` / `README.zh-CN.md`：用户说明。
- `CONTEXT.md`：当前任务进度和关键决定。
- `lessons.md`：用户反馈后沉淀的经验。

## 调用关系

`main.rs` 启动 `app.rs`；`app.rs` 读取 `settings.rs`，通过 `ccswitch.rs` 找到 provider，再用 `usage.rs` 查询余额，交给 `display.rs` 格式化，最后通过 `overlay.rs` 显示。`updater.rs` 在启动阶段独立检查更新。

## 关键决定

- 余额窗口不是系统托盘图标，而是一个轻量 Win32 置顶窗口，方便显示双行余额文字。
- 设置面板使用自绘行布局，避免中文被系统控件裁切。
- 全屏自动隐藏通过检测前台窗口是否覆盖显示器实现，不嵌入任务栏，也不改变余额查询逻辑。
- 配置保存在本机 `%APPDATA%\cc-balance-overlay\settings.toml`，不上传任何 key、provider 信息或余额。
