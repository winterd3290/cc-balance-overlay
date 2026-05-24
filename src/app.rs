use crate::ccswitch::{load_current_providers, AppKind, CcSwitchPaths};
use crate::display::{format_line, format_overlay_text};
use crate::settings::OverlaySettings;
use crate::usage::{query_balance, BalanceSnapshot};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub cc_switch_paths: CcSwitchPaths,
    pub refresh_interval: Duration,
}

impl AppConfig {
    pub fn discover() -> Result<Self> {
        Ok(Self {
            cc_switch_paths: CcSwitchPaths::discover()?,
            refresh_interval: Duration::from_secs(30),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BalanceState {
    pub text: String,
    pub tooltip: String,
    pub last_success: Option<Instant>,
    pub last_error: Option<String>,
}

pub struct BalanceApp {
    config: AppConfig,
    settings: OverlaySettings,
    last_known: HashMap<AppKind, BalanceSnapshot>,
    last_providers: HashMap<AppKind, String>,
}

impl BalanceApp {
    pub fn new(config: AppConfig, settings: OverlaySettings) -> Self {
        Self {
            config,
            settings,
            last_known: HashMap::new(),
            last_providers: HashMap::new(),
        }
    }

    pub fn update_settings(&mut self, settings: OverlaySettings) {
        self.settings = settings;
    }

    pub fn refresh(&mut self) -> BalanceState {
        match self.try_refresh() {
            Ok(state) => state,
            Err(err) => {
                let fallback = self.fallback_text();
                BalanceState {
                    text: if fallback.is_empty() {
                        "C --\nX --".to_string()
                    } else {
                        fallback
                    },
                    tooltip: self.tooltip_text(),
                    last_success: None,
                    last_error: Some(err.to_string()),
                }
            }
        }
    }

    fn try_refresh(&mut self) -> Result<BalanceState> {
        let providers = load_current_providers(&self.config.cc_switch_paths)?;
        let mut lines = Vec::new();
        let mut last_error = None;
        let mut had_success = false;

        for (app, provider) in providers {
            self.last_providers.insert(app, provider.name.clone());
            let balance = match provider.usage_script() {
                Some(script) if script.enabled => match query_balance(&script) {
                    Ok(snapshot) => {
                        if snapshot.remaining.is_some() {
                            self.last_known.insert(app, snapshot.clone());
                            had_success = true;
                        }
                        Some(snapshot)
                    }
                    Err(err) => {
                        last_error = Some(err.to_string());
                        self.last_known.get(&app).cloned()
                    }
                },
                _ => None,
            };
            lines.push(format_line(
                app,
                &provider.name,
                balance.as_ref(),
                &self.settings,
            ));
        }

        Ok(BalanceState {
            text: format_overlay_text(&lines),
            tooltip: self.tooltip_text(),
            last_success: had_success.then(Instant::now),
            last_error,
        })
    }

    fn fallback_text(&self) -> String {
        let mut lines = Vec::new();
        for app in [AppKind::Claude, AppKind::Codex] {
            lines.push(format_line(
                app,
                "",
                self.last_known.get(&app),
                &self.settings,
            ));
        }
        format_overlay_text(&lines)
    }

    fn tooltip_text(&self) -> String {
        let mut lines = Vec::new();
        for app in [AppKind::Claude, AppKind::Codex] {
            let name = self
                .last_providers
                .get(&app)
                .map(String::as_str)
                .unwrap_or("--");
            lines.push(format!("{}: {}", app.short_name(), name));
        }
        lines.join("\n")
    }

    pub fn refresh_interval(&self) -> Duration {
        self.config.refresh_interval
    }
}
