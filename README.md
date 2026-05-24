# CC Balance Overlay

<p align="center">
  <strong>Keep your Claude and Codex relay balance visible beside the Windows 11 tray.</strong>
</p>

<p align="center">
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="https://github.com/wgd-12138/cc-balance-overlay/releases/latest">Download</a> ·
  <a href="#quick-start">Quick start</a> ·
  <a href="#usage-guide">Usage guide</a>
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

CC Balance Overlay is a tiny native Windows utility for people who use Claude, Codex, CC Switch, and relay providers every day. It follows the active CC Switch providers and renders a compact two-line balance overlay next to the system tray, so you can see your remaining balance without opening a browser, terminal, or provider dashboard.

## Why

When you switch between multiple relay providers, balance information is usually hidden in web consoles or provider panels. You often notice the problem only after a request fails.

CC Balance Overlay turns that hidden state into something glanceable. It is intentionally small: no dashboard, no account system, no cloud service, just the current Claude and Codex balances where you already look.

## Features

- **Follows CC Switch automatically**: reads the currently selected Claude and Codex providers.
- **Live balance display**: refreshes periodically and keeps the last successful value as fallback.
- **Taskbar-side overlay**: compact two-line text beside the Windows tray area.
- **Clock-like layout**: low-distraction presentation inspired by the native Windows time/date block.
- **Provider tooltip**: hover to see the current Claude and Codex provider names.
- **Right-click settings**: font size, text color, Claude prefix, Codex prefix, startup toggle, and quit.
- **Local-first**: settings stay on your machine.
- **Native Rust + Win32**: lightweight app with no console window on launch.

## Quick Start

1. Install and configure CC Switch.
2. Make sure your Claude and Codex providers in CC Switch include usable `usage_script` balance metadata.
3. Download `cc-balance-overlay-v0.1.1-windows-x64.zip` from the [latest release](https://github.com/wgd-12138/cc-balance-overlay/releases/latest).
4. Extract the zip to a stable folder, for example `C:\Tools\cc-balance-overlay`.
5. Run `cc-balance-overlay.exe`.
6. Look near the Windows tray area. You should see two compact balance lines:

```text
C $17.7
X $88.0
```

The default prefixes are:

- `C`: Claude
- `X`: Codex

You can change both prefixes from the right-click settings panel.

## Usage Guide

### Read The Overlay

The overlay shows one line per app:

```text
C $17.7
X $88.0
```

If a balance cannot be loaded yet, the app shows `--` for that line and keeps running. When a previous successful value exists, it can keep displaying the last known value while the next refresh is pending.

### Hover For Provider Names

Move your mouse over the overlay to see which CC Switch providers are currently active for Claude and Codex. This is useful when you switch providers often and want to confirm which relay is being charged.

### Open Settings

Right-click the overlay to open the settings panel.

Available settings:

- **Font size**: adjust the overlay text size.
- **Text color**: choose a custom display color.
- **Claude prefix**: customize the label used for Claude.
- **Codex prefix**: customize the label used for Codex.
- **Start with Windows**: enable or disable startup at login.
- **Quit**: close the app.

Most changes apply immediately.

### Switch Providers

Change the active Claude or Codex provider in CC Switch as usual. CC Balance Overlay reads CC Switch state and follows the currently selected providers, so you do not need to configure provider names manually in this app.

### Start With Windows

Right-click the overlay and enable **Start with Windows**. The app writes a normal current-user startup entry and can be disabled from the same settings panel.

Keep the executable in the same folder after enabling startup. If you move the app, open the settings panel, turn **Start with Windows** off, then turn it on again so Windows stores the new path.

### Exit The App

Right-click the overlay and choose **Quit**.

## Build From Source

Requirements:

- Windows 11
- Rust stable
- A working Windows Rust toolchain: MSVC or GNU MinGW-w64
- A configured CC Switch installation

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

## Configuration

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

Most users should use the right-click settings panel instead of editing this file manually.

## How It Works

CC Balance Overlay reads local CC Switch state in read-only mode:

- `~\.cc-switch\settings.json`
- `~\.cc-switch\cc-switch.db`

It uses:

- `currentProviderClaude`
- `currentProviderCodex`
- `usage_script` metadata from the provider table

Then it calls the configured provider balance endpoint and renders the result as a compact taskbar-side overlay.

## Troubleshooting

### The Overlay Shows `--`

Check that:

- CC Switch is installed and has selected Claude / Codex providers.
- The selected provider has enabled `usage_script` metadata.
- The provider balance endpoint is reachable.
- Your provider key or token is still valid.

### The Balance Looks Wrong

Different relay providers may expose different balance response shapes. Please open an issue with the provider type and a redacted response example. Do not include real keys or tokens.

### Windows Blocks The App

The project is open source but the executable is not code-signed yet. Windows SmartScreen may warn on first launch. You can build from source if you prefer.

### I Cannot See The Overlay

Try right-clicking near the tray area, checking whether the app process is running, and restarting the app. Multi-monitor, DPI, and unusual taskbar layouts may need additional compatibility work.

## Privacy & Security

- This project does not operate a relay, proxy, or cloud service.
- It does not upload your keys, provider details, or balances to any third-party server.
- Balance requests go only to the endpoint configured in your CC Switch provider metadata.
- CC Switch data and CC Balance Overlay settings remain local.
- Avoid posting secret-bearing `settings.json`, database files, or logs in public issues.

## Limitations

- Currently focused on Windows 11.
- Requires CC Switch providers with usable `usage_script` balance metadata.
- Provider balance APIs may differ and may need adapter updates.
- More real-device testing is needed for multi-monitor, DPI, and taskbar-position edge cases.

## Roadmap

- Add real taskbar screenshots and a demo GIF
- Provide an installer
- Improve multi-monitor, DPI, and taskbar-position handling
- Add more provider balance adapter rules
- Improve text rendering fidelity against the native Windows clock
- Add auto-update or version checking

## Contributing

Issues and pull requests are welcome, especially for:

- New provider balance adapters
- Windows taskbar, DPI, and multi-monitor compatibility reports
- UI comparison screenshots
- Packaging and release improvements
- Documentation polish

If this tiny tool saves you from one failed request at the worst possible moment, consider giving it a star so more Claude, Codex, and CC Switch users can find it.

## License

MIT
