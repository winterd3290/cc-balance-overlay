use crate::settings::{OverlaySettings, UpdatePolicy};
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/wgd-12138/cc-balance-overlay/releases/latest";
const WINDOWS_ZIP_SUFFIX: &str = "windows-x64.zip";
const USER_AGENT: &str = "cc-balance-overlay";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub page_url: String,
    pub asset_name: String,
    pub asset_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatePromptChoice {
    Automatic,
    Disable,
    Later,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn check_for_updates_on_startup() {
    thread::spawn(|| {
        if let Err(err) = check_for_updates() {
            eprintln!("update check failed: {err:#}");
        }
    });
}

fn check_for_updates() -> Result<()> {
    let settings = OverlaySettings::load();
    if settings.update_policy == UpdatePolicy::Disabled {
        return Ok(());
    }

    let Some(release) = fetch_latest_release()? else {
        return Ok(());
    };

    if settings.update_policy == UpdatePolicy::Automatic {
        install_update_and_restart(&release)?;
        return Ok(());
    }

    match show_update_prompt(&release) {
        UpdatePromptChoice::Automatic => {
            let mut settings = OverlaySettings::load();
            settings.update_policy = UpdatePolicy::Automatic;
            settings.save()?;
            install_update_and_restart(&release)?;
        }
        UpdatePromptChoice::Disable => {
            let mut settings = OverlaySettings::load();
            settings.update_policy = UpdatePolicy::Disabled;
            settings.save()?;
        }
        UpdatePromptChoice::Later => {}
    }

    Ok(())
}

fn fetch_latest_release() -> Result<Option<ReleaseInfo>> {
    let release: GithubRelease = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?
        .get(LATEST_RELEASE_API)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?
        .error_for_status()?
        .json()?;

    let latest_version = normalize_version(&release.tag_name);
    if compare_versions(&latest_version, env!("CARGO_PKG_VERSION")) <= 0 {
        return Ok(None);
    }

    let Some(asset) = select_windows_asset(&release.assets) else {
        return Ok(None);
    };

    Ok(Some(ReleaseInfo {
        tag: release.tag_name,
        version: latest_version,
        page_url: release.html_url,
        asset_name: asset.name.clone(),
        asset_url: asset.browser_download_url.clone(),
    }))
}

fn select_windows_asset(assets: &[GithubAsset]) -> Option<&GithubAsset> {
    assets
        .iter()
        .find(|asset| asset.name.ends_with(WINDOWS_ZIP_SUFFIX))
}

fn normalize_version(value: &str) -> String {
    value.trim().trim_start_matches('v').to_string()
}

fn compare_versions(left: &str, right: &str) -> i32 {
    let left_parts = version_parts(&normalize_version(left));
    let right_parts = version_parts(&normalize_version(right));
    for index in 0..left_parts.len().max(right_parts.len()) {
        let left = *left_parts.get(index).unwrap_or(&0);
        let right = *right_parts.get(index).unwrap_or(&0);
        if left > right {
            return 1;
        }
        if left < right {
            return -1;
        }
    }
    0
}

fn version_parts(value: &str) -> Vec<u32> {
    value
        .split(|ch| ch == '.' || ch == '-' || ch == '+')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}

fn install_update_and_restart(release: &ReleaseInfo) -> Result<()> {
    let exe = std::env::current_exe()?;
    let exe_dir = exe
        .parent()
        .context("current executable has no parent directory")?
        .to_path_buf();
    let update_dir =
        std::env::temp_dir().join(format!("cc-balance-overlay-update-{}", release.tag));
    if update_dir.exists() {
        fs::remove_dir_all(&update_dir).ok();
    }
    fs::create_dir_all(&update_dir)?;

    let zip_path = update_dir.join(&release.asset_name);
    download_file(&release.asset_url, &zip_path)?;

    let script_path = update_dir.join("apply-update.ps1");
    write_update_script(&script_path, &zip_path, &exe, &exe_dir)?;
    launch_update_script(&script_path)?;
    std::process::exit(0);
}

fn download_file(url: &str, path: &Path) -> Result<()> {
    let mut response = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()?
        .error_for_status()?;
    let mut file = fs::File::create(path)?;
    response.copy_to(&mut file)?;
    Ok(())
}

fn write_update_script(
    script_path: &Path,
    zip_path: &Path,
    exe_path: &Path,
    exe_dir: &Path,
) -> Result<()> {
    let content = format!(
        r#"$ErrorActionPreference = 'Stop'
$zip = {zip}
$targetExe = {exe}
$targetDir = {dir}
$extractDir = Join-Path ([System.IO.Path]::GetTempPath()) ('cc-balance-overlay-extract-' + [Guid]::NewGuid().ToString('N'))
Start-Sleep -Seconds 2
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
Expand-Archive -LiteralPath $zip -DestinationPath $extractDir -Force
$newExe = Get-ChildItem -LiteralPath $extractDir -Filter 'cc-balance-overlay.exe' -Recurse | Select-Object -First 1
if (-not $newExe) {{ throw 'cc-balance-overlay.exe not found in update package' }}
Copy-Item -LiteralPath $newExe.FullName -Destination $targetExe -Force
Start-Process -FilePath $targetExe -WorkingDirectory $targetDir
"#,
        zip = powershell_quote(zip_path),
        exe = powershell_quote(exe_path),
        dir = powershell_quote(exe_dir),
    );
    fs::write(script_path, content)?;
    Ok(())
}

fn powershell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

fn launch_update_script(script_path: &Path) -> Result<()> {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(script_path)
        .spawn()
        .context("failed to launch updater script")?;
    Ok(())
}

#[cfg(windows)]
fn show_update_prompt(release: &ReleaseInfo) -> UpdatePromptChoice {
    use std::ptr::null_mut;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Controls::{
        TaskDialogIndirect, TASKDIALOGCONFIG, TASKDIALOG_BUTTON, TDF_ALLOW_DIALOG_CANCELLATION,
        TDF_USE_COMMAND_LINKS,
    };

    const BUTTON_AUTO: i32 = 1001;
    const BUTTON_DISABLE: i32 = 1002;

    let title = wide("CC Balance Overlay");
    let instruction = wide(&format!("发现新版本 {}", release.tag));
    let content = wide(&format!(
        "当前版本是 {}，最新版本是 {}。\n\n请选择后续更新方式：",
        env!("CARGO_PKG_VERSION"),
        release.version
    ));
    let auto = wide("自动更新\n以后检测到新版本时自动下载并重启应用");
    let disable = wide("不再提醒\n不再检测新版本，也不再自动更新");
    let buttons = [
        TASKDIALOG_BUTTON {
            nButtonID: BUTTON_AUTO,
            pszButtonText: PCWSTR(auto.as_ptr()),
        },
        TASKDIALOG_BUTTON {
            nButtonID: BUTTON_DISABLE,
            pszButtonText: PCWSTR(disable.as_ptr()),
        },
    ];
    let mut selected = 0;
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        hwndParent: HWND(null_mut()),
        dwFlags: TDF_USE_COMMAND_LINKS | TDF_ALLOW_DIALOG_CANCELLATION,
        pszWindowTitle: PCWSTR(title.as_ptr()),
        pszMainInstruction: PCWSTR(instruction.as_ptr()),
        pszContent: PCWSTR(content.as_ptr()),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: BUTTON_AUTO,
        ..Default::default()
    };

    let result = unsafe { TaskDialogIndirect(&config, Some(&mut selected), None, None) };
    if result.is_err() {
        return UpdatePromptChoice::Later;
    }

    match selected {
        BUTTON_AUTO => UpdatePromptChoice::Automatic,
        BUTTON_DISABLE => UpdatePromptChoice::Disable,
        _ => UpdatePromptChoice::Later,
    }
}

#[cfg(not(windows))]
fn show_update_prompt(_release: &ReleaseInfo) -> UpdatePromptChoice {
    UpdatePromptChoice::Later
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_semver_like_versions() {
        assert!(compare_versions("0.2.0", "0.1.9") > 0);
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert_eq!(compare_versions("v1.2.3", "1.2.3"), 0);
    }

    #[test]
    fn selects_windows_zip_asset() {
        let assets = vec![
            GithubAsset {
                name: "cc-balance-overlay-v0.2.0-windows-x64.zip.sha256".to_string(),
                browser_download_url: "sha".to_string(),
            },
            GithubAsset {
                name: "cc-balance-overlay-v0.2.0-windows-x64.zip".to_string(),
                browser_download_url: "zip".to_string(),
            },
        ];

        let asset = select_windows_asset(&assets).unwrap();
        assert_eq!(asset.browser_download_url, "zip");
    }

    #[test]
    fn powershell_quote_escapes_single_quotes() {
        let path = Path::new(r"C:\Tools\cc 'balance'\cc-balance-overlay.exe");
        assert_eq!(
            powershell_quote(path),
            r#"'C:\Tools\cc ''balance''\cc-balance-overlay.exe'"#
        );
    }
}
