#![cfg_attr(windows, windows_subsystem = "windows")]

use anyhow::Result;
use cc_balance_overlay::app::{AppConfig, BalanceApp};
use cc_balance_overlay::overlay::OverlayWindow;
use cc_balance_overlay::settings::OverlaySettings;

fn main() -> Result<()> {
    let config = AppConfig::discover()?;
    let settings = OverlaySettings::load();
    let mut app = BalanceApp::new(config, settings.clone());
    let mut overlay = OverlayWindow::new(settings)?;
    let initial = app.refresh();
    overlay.set_text(initial.text);
    overlay.set_tooltip(initial.tooltip);
    overlay.run_message_loop(move |window| {
        let settings = OverlaySettings::load();
        app.update_settings(settings.clone());
        window.apply_settings(settings);
        let state = app.refresh();
        window.set_text(state.text);
        window.set_tooltip(state.tooltip);
    })?;
    Ok(())
}
