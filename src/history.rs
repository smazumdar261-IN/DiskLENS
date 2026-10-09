use crate::models::{format_size, FileNode};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaType {
    Increase,
    Decrease,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSnapshotRecord {
    pub id: String,
    pub timestamp: String,
    pub root_path: String,
    pub total_size_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub duration_seconds: f64,
    pub top_subdirs: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct ScanHistoryItem {
    pub id: String,
    pub date: String,
    pub root_path: String,
    pub total_size_bytes: u64,
    pub total_size_formatted: String,
    pub delta_bytes: i64,
    pub delta_formatted: String,
    pub delta_type: DeltaType,
    pub duration_seconds: f64,
    pub file_count: u64,
}

#[derive(Debug, Clone)]
pub struct GrowthInsightItem {
    pub id: String,
    pub path: String,
    pub change_formatted: String,
    pub is_growth: bool,
    pub icon: String,
}

pub struct StorageHistoryManager;

impl StorageHistoryManager {
    pub fn history_file_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("disklens")
            .join("history.json")
    }

    pub fn load_snapshots() -> Vec<ScanSnapshotRecord> {
        let path = Self::history_file_path();
        if path.exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(records) = serde_json::from_str::<Vec<ScanSnapshotRecord>>(&contents)
                    {
                        return records;
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn save_snapshots(snapshots: &[ScanSnapshotRecord]) -> Result<(), String> {
        let path = Self::history_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create history dir: {}", e))?;
        }

        let json = serde_json::to_string_pretty(snapshots)
            .map_err(|e| format!("Serialization error: {}", e))?;

        let mut file =
            File::create(&path).map_err(|e| format!("Failed to write history file: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to save history: {}", e))?;

        Ok(())
    }

    pub fn record_scan(root: &FileNode, elapsed_secs: f64) -> Result<(), String> {
        let mut snapshots = Self::load_snapshots();
        let now: DateTime<Local> = Local::now();
        let timestamp_str = now.format("%Y-%m-%d %H:%M").to_string();

        let mut top_subdirs = Vec::new();
        for child in &root.children {
            if child.is_dir {
                top_subdirs.push((child.name.clone(), child.size));
            }
        }
        top_subdirs.sort_by(|a, b| b.1.cmp(&a.1));
        top_subdirs.truncate(10);

        let new_record = ScanSnapshotRecord {
            id: format!("hist-{}", now.timestamp()),
            timestamp: timestamp_str,
            root_path: root.path.display().to_string(),
            total_size_bytes: root.size,
            file_count: root.file_count,
            dir_count: root.dir_count,
            duration_seconds: elapsed_secs,
            top_subdirs,
        };

        snapshots.insert(0, new_record);
        if snapshots.len() > 100 {
            snapshots.truncate(100);
        }

        Self::save_snapshots(&snapshots)
    }

    pub fn get_history_items(target_root: Option<&Path>) -> Vec<ScanHistoryItem> {
        let snapshots = Self::load_snapshots();
        let target_str = target_root.map(|p| p.display().to_string());

        let matching: Vec<&ScanSnapshotRecord> = snapshots
            .iter()
            .filter(|s| {
                if let Some(target) = &target_str {
                    &s.root_path == target || s.root_path.starts_with(target)
                } else {
                    true
                }
            })
            .collect();

        let mut items = Vec::new();
        for (i, current) in matching.iter().enumerate() {
            let previous = matching.get(i + 1);
            let (delta_bytes, delta_type, delta_formatted) = match previous {
                Some(prev) => {
                    let diff = current.total_size_bytes as i64 - prev.total_size_bytes as i64;
                    if diff > 0 {
                        (
                            diff,
                            DeltaType::Increase,
                            format!("+{}", format_size(diff as u64)),
                        )
                    } else if diff < 0 {
                        (
                            diff,
                            DeltaType::Decrease,
                            format!("-{}", format_size((-diff) as u64)),
                        )
                    } else {
                        (0, DeltaType::Neutral, "0 B".to_string())
                    }
                }
                None => (0, DeltaType::Neutral, "First Scan".to_string()),
            };

            items.push(ScanHistoryItem {
                id: current.id.clone(),
                date: current.timestamp.clone(),
                root_path: current.root_path.clone(),
                total_size_bytes: current.total_size_bytes,
                total_size_formatted: format_size(current.total_size_bytes),
                delta_bytes,
                delta_formatted,
                delta_type,
                duration_seconds: current.duration_seconds,
                file_count: current.file_count,
            });
        }

        items
    }

    pub fn compute_growth_insights(
        target_root: Option<&Path>,
    ) -> (Vec<GrowthInsightItem>, Vec<GrowthInsightItem>) {
        let snapshots = Self::load_snapshots();
        let target_str = target_root.map(|p| p.display().to_string());

        let matching: Vec<&ScanSnapshotRecord> = snapshots
            .iter()
            .filter(|s| {
                if let Some(target) = &target_str {
                    &s.root_path == target
                } else {
                    true
                }
            })
            .collect();

        if matching.len() < 2 {
            return (Vec::new(), Vec::new());
        }

        let current = matching[0];
        let previous = matching[1];

        let mut prev_map: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
        for (name, size) in &previous.top_subdirs {
            prev_map.insert(name.as_str(), *size);
        }

        let mut growth = Vec::new();
        let mut shrinking = Vec::new();

        for (name, curr_size) in &current.top_subdirs {
            let prev_size = prev_map.get(name.as_str()).copied().unwrap_or(0);
            let diff = *curr_size as i64 - prev_size as i64;

            if diff > 10 * 1024 * 1024 {
                // > 10 MB growth
                growth.push(GrowthInsightItem {
                    id: format!("grow-{}", name),
                    path: format!("{}/{}", current.root_path, name),
                    change_formatted: format!("+{}", format_size(diff as u64)),
                    is_growth: true,
                    icon: "folder".to_string(),
                });
            } else if diff < -10 * 1024 * 1024 {
                // > 10 MB shrinking
                shrinking.push(GrowthInsightItem {
                    id: format!("shrink-{}", name),
                    path: format!("{}/{}", current.root_path, name),
                    change_formatted: format!("-{}", format_size((-diff) as u64)),
                    is_growth: false,
                    icon: "folder".to_string(),
                });
            }
        }

        growth.sort_by(|a, b| b.change_formatted.cmp(&a.change_formatted));
        shrinking.sort_by(|a, b| a.change_formatted.cmp(&b.change_formatted));

        (growth, shrinking)
    }
}
