use crate::vcs::VcsPreference;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub work: WorkConfig,
    #[serde(default)]
    pub vcs: VcsConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub editor: EditorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkConfig {
    #[serde(default)]
    pub skip_permissions: bool,
    #[serde(default)]
    pub auto_implement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VcsConfig {
    #[serde(default)]
    pub prefer: VcsPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_true")]
    pub colors: bool,
}

fn default_true() -> bool {
    true
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { colors: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorConfig {
    pub command: Option<String>,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        if path.exists() {
            let content = tokio::fs::read_to_string(path)
                .await
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        tokio::fs::write(path, content)
            .await
            .context("Failed to write config file")?;
        
        Ok(())
    }

    pub fn get_editor(&self) -> String {
        // Check config first
        if let Some(ref cmd) = self.editor.command {
            return cmd.clone();
        }
        
        // Check EDITOR environment variable
        if let Ok(editor) = std::env::var("EDITOR") {
            return editor;
        }
        
        // Platform-specific defaults
        #[cfg(target_os = "windows")]
        return "notepad".to_string();
        
        #[cfg(not(target_os = "windows"))]
        return "vim".to_string();
    }
}

pub fn find_pebbles_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()
        .context("Failed to get current directory")?;
    
    loop {
        let pebbles_dir = current.join(".pebbles");
        if pebbles_dir.exists() {
            return Ok(current);
        }
        
        if !current.pop() {
            break;
        }
    }
    
    Err(anyhow::anyhow!("Not in a pebbles repository. Run 'pebbles init' first."))
}

pub fn get_pebbles_dir() -> Result<PathBuf> {
    find_pebbles_root().map(|root| root.join(".pebbles"))
}

pub fn get_db_path() -> Result<PathBuf> {
    get_pebbles_dir().map(|dir| dir.join("db.json"))
}

pub fn get_config_path() -> Result<PathBuf> {
    get_pebbles_dir().map(|dir| dir.join("config.toml"))
}
