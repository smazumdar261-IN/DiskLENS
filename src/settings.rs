use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymlinkPolicy {
    DontFollow,
    FollowAll,
    FollowWithinRoot,
}

impl Default for SymlinkPolicy {
    fn default() -> Self {
        Self::DontFollow
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreference {
    Dark,
    Light,
    System,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub symlink_policy: SymlinkPolicy,
    pub include_hidden_files: bool,
    pub cross_filesystem_boundaries: bool,
    pub exclusions: Vec<String>,
    pub confirm_before_trash: bool,
    pub confirm_before_permanent_delete: bool,
    pub worker_threads: usize,
    pub enable_sparse_detection: bool,
    pub enable_debug_logging: bool,
    pub theme_preference: ThemePreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(8)
            .clamp(1, 16);

        Self {
            symlink_policy: SymlinkPolicy::DontFollow,
            include_hidden_files: true,
            cross_filesystem_boundaries: false,
            exclusions: vec![
                "/proc".to_string(),
                "/sys".to_string(),
                "/dev".to_string(),
                "/run".to_string(),
                "~/.cache".to_string(),
            ],
            confirm_before_trash: true,
            confirm_before_permanent_delete: true,
            worker_threads: threads,
            enable_sparse_detection: false,
            enable_debug_logging: false,
            theme_preference: ThemePreference::Dark,
        }
    }
}

impl AppSettings {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("disklens")
            .join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(settings) = serde_json::from_str::<AppSettings>(&contents) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let mut file =
            File::create(&path).map_err(|e| format!("Failed to write settings file: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to save settings: {}", e))?;

        Ok(())
    }

    pub fn reset_defaults(&mut self) {
        *self = Self::default();
    }
}
