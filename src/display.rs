use crate::ccswitch::AppKind;
use crate::settings::OverlaySettings;
use crate::usage::BalanceSnapshot;

#[derive(Debug, Clone, PartialEq)]
pub struct DisplayLine {
    pub app: AppKind,
    pub provider_name: String,
    pub text: String,
}

pub fn format_line(
    app: AppKind,
    provider_name: &str,
    balance: Option<&BalanceSnapshot>,
    settings: &OverlaySettings,
) -> DisplayLine {
    let value = balance
        .and_then(|snapshot| snapshot.remaining)
        .map(format_amount)
        .unwrap_or_else(|| "--".to_string());
    let unit = balance
        .map(|snapshot| normalize_unit(&snapshot.unit))
        .unwrap_or_default();
    let text = if unit.is_empty() {
        format!("{} {}", prefix_for(app, settings), value)
    } else {
        format!("{} {}{}", prefix_for(app, settings), unit, value)
    };

    DisplayLine {
        app,
        provider_name: provider_name.to_string(),
        text,
    }
}

fn prefix_for(app: AppKind, settings: &OverlaySettings) -> &str {
    match app {
        AppKind::Claude => &settings.claude_prefix,
        AppKind::Codex => &settings.codex_prefix,
    }
}

pub fn format_overlay_text(lines: &[DisplayLine]) -> String {
    lines
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_amount(value: f64) -> String {
    if value.abs() >= 1000.0 {
        format!("{value:.0}")
    } else if value.abs() >= 100.0 {
        format!("{value:.0}")
    } else if value.abs() >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

fn normalize_unit(unit: &str) -> String {
    match unit.to_ascii_uppercase().as_str() {
        "" => String::new(),
        "CNY" | "RMB" | "YUAN" => "¥".to_string(),
        "USD" => "$".to_string(),
        other => format!("{other} "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_two_compact_lines_for_taskbar() {
        let claude = BalanceSnapshot {
            remaining: Some(12.34),
            unit: "CNY".to_string(),
            valid: true,
            message: None,
        };
        let codex = BalanceSnapshot {
            remaining: Some(90.426),
            unit: "USD".to_string(),
            valid: true,
            message: None,
        };

        let lines = vec![
            format_line(
                AppKind::Claude,
                "Example Relay",
                Some(&claude),
                &OverlaySettings::default(),
            ),
            format_line(
                AppKind::Codex,
                "Codex++",
                Some(&codex),
                &OverlaySettings::default(),
            ),
        ];

        assert_eq!(format_overlay_text(&lines), "C ¥12.3\nX $90.4");
    }

    #[test]
    fn shows_placeholder_when_balance_is_unknown() {
        let line = format_line(
            AppKind::Claude,
            "Example Relay",
            None,
            &OverlaySettings::default(),
        );

        assert_eq!(line.text, "C --");
    }
}
