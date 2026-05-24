use anyhow::{Context, Result};
use rusqlite::{params, Connection, OpenFlags};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CcSwitchPaths {
    pub root: PathBuf,
    pub settings: PathBuf,
    pub database: PathBuf,
}

impl CcSwitchPaths {
    pub fn discover() -> Result<Self> {
        let home = dirs::home_dir().context("could not find user home directory")?;
        Self::from_root(home.join(".cc-switch"))
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        Ok(Self {
            settings: root.join("settings.json"),
            database: root.join("cc-switch.db"),
            root,
        })
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CcSwitchSettings {
    pub current_provider_claude: Option<String>,
    pub current_provider_codex: Option<String>,
}

impl CcSwitchSettings {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read CC Switch settings at {}", path.display()))?;
        let settings = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse CC Switch settings at {}", path.display()))?;
        Ok(settings)
    }

    pub fn current_provider_for(&self, app: AppKind) -> Option<&str> {
        match app {
            AppKind::Claude => self.current_provider_claude.as_deref(),
            AppKind::Codex => self.current_provider_codex.as_deref(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppKind {
    Claude,
    Codex,
}

impl AppKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AppKind::Claude => "claude",
            AppKind::Codex => "codex",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppKind::Claude => "C",
            AppKind::Codex => "X",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            AppKind::Claude => "Claude",
            AppKind::Codex => "Codex",
        }
    }

    pub fn clock_label(self) -> &'static str {
        match self {
            AppKind::Claude => "C",
            AppKind::Codex => "X",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRecord {
    pub id: String,
    pub app_type: String,
    pub name: String,
    pub website_url: Option<String>,
    pub meta: Value,
}

impl ProviderRecord {
    pub fn usage_script(&self) -> Option<crate::usage::UsageScript> {
        let script = self.meta.get("usage_script")?;
        serde_json::from_value(script.clone()).ok()
    }
}

pub fn load_current_providers(paths: &CcSwitchPaths) -> Result<Vec<(AppKind, ProviderRecord)>> {
    let settings = CcSwitchSettings::load(&paths.settings)?;
    let db = ProviderDb::open_read_only(&paths.database)?;
    let mut providers = Vec::new();
    for app in [AppKind::Claude, AppKind::Codex] {
        let Some(provider_id) = settings.current_provider_for(app) else {
            continue;
        };
        let provider = db
            .find_provider(provider_id, app)
            .with_context(|| format!("failed to load {} provider {}", app.as_str(), provider_id))?;
        providers.push((app, provider));
    }
    Ok(providers)
}

pub struct ProviderDb {
    conn: Connection,
}

impl ProviderDb {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .with_context(|| format!("failed to open CC Switch database at {}", path.display()))?;
        Ok(Self { conn })
    }

    pub fn open_for_tests(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(path)?,
        })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn find_provider(&self, provider_id_or_name: &str, app: AppKind) -> Result<ProviderRecord> {
        let mut stmt = self.conn.prepare(
            "select id, app_type, name, website_url, meta
             from providers
             where (id = ?1 or name = ?1) and app_type = ?2
             limit 1",
        )?;
        let record = stmt.query_row(params![provider_id_or_name, app.as_str()], |row| {
            let meta_text: Option<String> = row.get(4)?;
            let meta = meta_text
                .as_deref()
                .and_then(|raw| serde_json::from_str(raw).ok())
                .unwrap_or(Value::Null);
            Ok(ProviderRecord {
                id: row.get(0)?,
                app_type: row.get(1)?,
                name: row.get(2)?,
                website_url: row.get(3)?,
                meta,
            })
        })?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_current_provider_ids_from_settings() {
        let dir = tempdir().unwrap();
        let settings_path = dir.path().join("settings.json");
        std::fs::write(
            &settings_path,
            r#"{
              "currentProviderClaude": "claude-a",
              "currentProviderCodex": "codex-b",
              "language": "zh"
            }"#,
        )
        .unwrap();

        let settings = CcSwitchSettings::load(&settings_path).unwrap();

        assert_eq!(
            settings.current_provider_for(AppKind::Claude),
            Some("claude-a")
        );
        assert_eq!(settings.current_provider_for(AppKind::Codex), Some("codex-b"));
    }

    #[test]
    fn reads_provider_metadata_by_id_without_requiring_secret_logging() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("cc-switch.db");
        let db = ProviderDb::open_for_tests(&db_path).unwrap();
        db.connection()
            .execute_batch(
                r#"
                create table providers (
                    id text,
                    app_type text,
                    name text,
                    website_url text,
                    meta text
                );
                insert into providers values (
                    'codex-current',
                    'codex',
                    'Codex Provider',
                    'https://example.test',
                    '{"usage_script":{"enabled":true,"baseUrl":"https://example.test","apiKey":"TEST_SECRET","code":"({ request: { url: \"{{baseUrl}}/v1/usage\" } })"}}'
                );
                "#,
            )
            .unwrap();

        let provider = db.find_provider("codex-current", AppKind::Codex).unwrap();

        assert_eq!(provider.name, "Codex Provider");
        assert_eq!(
            provider.usage_script().unwrap().base_url.as_deref(),
            Some("https://example.test")
        );
    }
}
