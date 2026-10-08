use chrono::{DateTime, Local};
use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Inode key for hardlink deduplication across filesystems.
/// Combines device ID (st_dev) and inode number (st_ino).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InodeKey {
    pub device: u64,
    pub inode: u64,
}

impl InodeKey {
    pub fn new(device: u64, inode: u64) -> Self {
        Self { device, inode }
    }
}

/// Represents a group of hardlinked files sharing the same inode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardlinkGroup {
    pub inode_key: InodeKey,
    pub paths: Vec<PathBuf>,
    pub apparent_size: u64,
    pub allocated_size: u64,
    pub link_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileCategory {
    Videos,
    Archives,
    Images,
    Code,
    Documents,
    Audio,
    DiskImages,
    SystemConfig,
    Executables,
    Other,
}

impl FileCategory {
    /// Shared category taxonomy for filters, charts, legends, and exports.
    pub const ALL: [FileCategory; 10] = [
        FileCategory::Videos,
        FileCategory::Archives,
        FileCategory::Images,
        FileCategory::Code,
        FileCategory::Documents,
        FileCategory::Audio,
        FileCategory::DiskImages,
        FileCategory::SystemConfig,
        FileCategory::Executables,
        FileCategory::Other,
    ];

    pub fn from_extension(ext: &str) -> Self {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            // Videos
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "3gp" | "m2ts" => {
                FileCategory::Videos
            }

            // Archives & Packages
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" | "tgz" | "deb" | "rpm"
            | "apk" | "jar" | "war" => FileCategory::Archives,

            // Images
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" | "tiff" | "tif"
            | "avif" | "raw" | "nef" | "cr2" | "heic" => FileCategory::Images,

            // Code, Dev & Libraries
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "c" | "cpp" | "h" | "hpp" | "go"
            | "java" | "kt" | "swift" | "rb" | "php" | "html" | "css" | "scss" | "sass"
            | "json" | "xml" | "yaml" | "yml" | "toml" | "sql" | "sh" | "bash" | "zsh"
            | "dockerfile" | "proto" | "lock" | "rlib" | "rmeta" | "d" | "node" | "whl"
            | "crate" => FileCategory::Code,

            // Documents
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
            | "txt" | "md" | "rtf" | "csv" | "tsv" | "epub" | "pages" | "numbers" | "key" => {
                FileCategory::Documents
            }

            // Audio
            "mp3" | "flac" | "wav" | "aac" | "ogg" | "m4a" | "wma" | "opus" | "alac" | "aiff" => {
                FileCategory::Audio
            }

            // Disk Images & Virtualization
            "iso" | "img" | "qcow2" | "vmdk" | "vdi" | "dmg" | "vhd" | "vhdx" => {
                FileCategory::DiskImages
            }

            // Config, Logs, Caches & Temp
            "conf" | "cfg" | "ini" | "log" | "env" | "service" | "tmp" | "temp" | "cache"
            | "pyc" | "pyo" | "bak" | "old" | "swp" | "swo" | "ldb" | "body" => {
                FileCategory::SystemConfig
            }

            // Executables & System Binaries
            "exe" | "bin" | "so" | "dll" | "dylib" | "appimage" | "msi" | "o" | "a" | "wasm" => {
                FileCategory::Executables
            }

            _ => FileCategory::Other,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FileCategory::Videos => "Videos",
            FileCategory::Archives => "Archives",
            FileCategory::Images => "Images",
            FileCategory::Code => "Code & Dev",
            FileCategory::Documents => "Documents",
            FileCategory::Audio => "Audio",
            FileCategory::DiskImages => "Disk / VM",
            FileCategory::SystemConfig => "System & Logs",
            FileCategory::Executables => "Binaries",
            FileCategory::Other => "Other / Unknown",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            FileCategory::Videos => Color32::from_rgb(122, 162, 247),   // #7AA2F7 (Tokyo Blue)
            FileCategory::Archives => Color32::from_rgb(207, 201, 194), // #CFC9C2 (Warm Gray)
            FileCategory::Images => Color32::from_rgb(187, 154, 247),   // #BB9AF7 (Purple)
            FileCategory::Code => Color32::from_rgb(115, 218, 202),     // #73DACA (Cyan)
            FileCategory::Documents => Color32::from_rgb(224, 175, 104),// #E0AF68 (Amber)
            FileCategory::Audio => Color32::from_rgb(247, 118, 142),    // #F7768E (Coral Red)
            FileCategory::DiskImages => Color32::from_rgb(42, 195, 222),// #2AC3DE (Teal)
            FileCategory::SystemConfig => Color32::from_rgb(137, 221, 255), // #89DDFF (Sky)
            FileCategory::Executables => Color32::from_rgb(236, 72, 153), // #EC4899 (Pink)
            FileCategory::Other => Color32::from_rgb(121, 130, 169),    // #7982A9 (Muted Slate)
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            FileCategory::Videos => "🎬",
            FileCategory::Archives => "📦",
            FileCategory::Images => "🖼",
            FileCategory::Code => "💻",
            FileCategory::Documents => "📄",
            FileCategory::Audio => "🎵",
            FileCategory::DiskImages => "💿",
            FileCategory::SystemConfig => "🔧",
            FileCategory::Executables => "⚙",
            FileCategory::Other => "📁",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupRecommendation {
    EasyWin,      // Cache, temporary data, build targets
    ReviewNeeded, // Large videos, archives, old disk images
    KeepSystem,   // OS files, configs, system libraries
}

#[derive(Debug, Clone)]
pub struct StorageInsight {
    pub severity: &'static str, // "high", "medium", "info"
    pub title: String,
    pub description: String,
    pub why_it_matters: String,
    pub size_bytes: u64,
    pub category: FileCategory,
    pub action_label: &'static str,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    /// Apparent size (logical file size from st_size)
    pub size: u64,
    /// Actual disk allocation (st_blocks * 512) — accounts for sparse files, compression, and block alignment.
    /// Equals size for regular files on non-compressed filesystems; smaller for sparse files; larger for compressed (Btrfs/ZFS) with compression.
    pub allocated_size: u64,
    pub is_dir: bool,
    pub file_count: u64,
    pub dir_count: u64,
    pub children: Vec<FileNode>,
    #[serde(skip)]
    pub modified: Option<SystemTime>,
    pub extension: Option<String>,
    pub category: FileCategory,
    /// Inode number (st_ino) for hardlink deduplication. None for directories or on error.
    pub inode: Option<u64>,
    /// Device ID (st_dev) for cross-filesystem inode uniqueness.
    pub device: Option<u64>,
    /// Number of hardlinks (st_nlink). >1 indicates hardlinked file.
    pub nlink: u32,
    /// Permission error if the file/directory could not be fully read.
    /// Used to display visual warning badges and offer elevation via pkexec.
    pub permission_error: Option<String>,
}

impl FileNode {
    /// Convenience constructor with standard metadata (allocated_size = size, single link).
    pub fn new_file(
        name: String,
        path: PathBuf,
        size: u64,
        modified: Option<SystemTime>,
    ) -> Self {
        Self::new_file_extended(name, path, size, size, modified, None, None, 1, None)
    }

    /// Full constructor supporting Linux POSIX extended stat metadata (blocks, hardlink inodes, permissions).
    pub fn new_file_extended(
        name: String,
        path: PathBuf,
        size: u64,
        allocated_size: u64,
        modified: Option<SystemTime>,
        inode: Option<u64>,
        device: Option<u64>,
        nlink: u32,
        permission_error: Option<String>,
    ) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        let category = match &extension {
            Some(ext) => FileCategory::from_extension(ext),
            None => FileCategory::Other,
        };

        Self {
            name,
            path,
            size,
            allocated_size,
            is_dir: false,
            file_count: 1,
            dir_count: 0,
            children: Vec::new(),
            modified,
            extension,
            category,
            inode,
            device,
            nlink,
            permission_error,
        }
    }

    /// Convenience directory constructor.
    pub fn new_dir(name: String, path: PathBuf, modified: Option<SystemTime>) -> Self {
        Self::new_dir_extended(name, path, modified, None)
    }

    /// Directory constructor with permission error tracking.
    pub fn new_dir_extended(
        name: String,
        path: PathBuf,
        modified: Option<SystemTime>,
        permission_error: Option<String>,
    ) -> Self {
        Self {
            name,
            path,
            size: 0,
            allocated_size: 0,
            is_dir: true,
            file_count: 0,
            dir_count: 0,
            children: Vec::new(),
            modified,
            extension: None,
            category: FileCategory::Other,
            inode: None,
            device: None,
            nlink: 0,
            permission_error,
        }
    }

    pub fn total_items(&self) -> u64 {
        if self.is_dir {
            self.file_count + self.dir_count
        } else {
            1
        }
    }

    pub fn formatted_size(&self) -> String {
        format_size(self.size)
    }

    pub fn formatted_allocated_size(&self) -> String {
        format_size(self.allocated_size)
    }

    pub fn formatted_modified(&self) -> String {
        match self.modified {
            Some(time) => {
                let dt: DateTime<Local> = time.into();
                dt.format("%Y-%m-%d %H:%M").to_string()
            }
            None => "-".to_string(),
        }
    }

    /// Returns true if this file is a hardlink (nlink > 1) and has valid inode/device.
    pub fn is_hardlink(&self) -> bool {
        self.nlink > 1 && self.inode.is_some() && self.device.is_some()
    }

    /// Returns the inode key for hardlink grouping, if available.
    pub fn inode_key(&self) -> Option<InodeKey> {
        match (self.inode, self.device) {
            (Some(inode), Some(device)) => Some(InodeKey::new(device, inode)),
            _ => None,
        }
    }

    pub fn find_node_by_path(&self, target_path: &Path) -> Option<&FileNode> {
        if self.path == target_path {
            return Some(self);
        }
        if self.is_dir && target_path.starts_with(&self.path) {
            for child in &self.children {
                if let Some(found) = child.find_node_by_path(target_path) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn prune_path(&mut self, target: &Path) -> bool {
        if !self.is_dir {
            return false;
        }

        let mut removed = false;
        self.children.retain(|c| {
            if c.path == target {
                removed = true;
                false
            } else {
                true
            }
        });

        if removed {
            self.recalculate_stats();
            return true;
        }

        for child in &mut self.children {
            if child.is_dir && target.starts_with(&child.path) {
                if child.prune_path(target) {
                    self.recalculate_stats();
                    return true;
                }
            }
        }

        false
    }

    pub fn recalculate_stats(&mut self) {
        self.size = self.children.iter().map(|c| c.size).sum();
        self.allocated_size = self.children.iter().map(|c| c.allocated_size).sum();
        self.file_count = self
            .children
            .iter()
            .map(|c| if c.is_dir { c.file_count } else { 1 })
            .sum();
        self.dir_count = self
            .children
            .iter()
            .map(|c| if c.is_dir { c.dir_count + 1 } else { 0 })
            .sum();
    }

    pub fn collect_largest_files(&self, list: &mut Vec<LargeFileInfo>) {
        if !self.is_dir {
            let path_str = self.path.to_string_lossy().to_string();
            let recommendation = if path_str.contains("/.cache/")
                || path_str.contains("/tmp/")
                || path_str.ends_with(".pyc")
                || path_str.ends_with(".tmp")
            {
                CleanupRecommendation::EasyWin
            } else if self.category == FileCategory::Videos
                || self.category == FileCategory::DiskImages
                || self.size > 500 * 1024 * 1024
            {
                CleanupRecommendation::ReviewNeeded
            } else {
                CleanupRecommendation::KeepSystem
            };

            let is_cache_or_temp = matches!(recommendation, CleanupRecommendation::EasyWin);
            let is_review_needed = matches!(recommendation, CleanupRecommendation::ReviewNeeded);

            let risk_note = if is_cache_or_temp {
                "Application temporary cache. Safe to clean if regenerated.".to_string()
            } else if is_review_needed {
                "Large standalone media / disk payload. Review contents before moving to trash.".to_string()
            } else {
                "Standard user data or system dependency.".to_string()
            };

            list.push(LargeFileInfo {
                name: self.name.clone(),
                path: self.path.clone(),
                size: self.size,
                allocated_size: self.allocated_size,
                extension: self.extension.clone(),
                category: self.category,
                recommendation,
                is_cache_or_temp,
                is_review_needed,
                risk_note,
                modified: self.modified,
                inode: self.inode,
                device: self.device,
                nlink: self.nlink,
            });
        } else {
            for child in &self.children {
                child.collect_largest_files(list);
            }
        }
    }

    /// Collects all hardlink groups in the tree.
    /// Returns a map of InodeKey -> list of file nodes sharing that inode.
    pub fn collect_hardlink_groups<'a>(&'a self, groups: &mut HashMap<InodeKey, Vec<&'a FileNode>>) {
        if !self.is_dir {
            if let Some(key) = self.inode_key() {
                if self.nlink > 1 {
                    groups.entry(key).or_default().push(self);
                }
            }
        } else {
            for child in &self.children {
                child.collect_hardlink_groups(groups);
            }
        }
    }

    /// Builds deduplicated HardlinkGroup structs from the tree.
    /// Each group represents a unique inode with its hardlinked paths.
    pub fn build_hardlink_groups(&self) -> Vec<HardlinkGroup> {
        let mut groups: HashMap<InodeKey, Vec<&FileNode>> = HashMap::new();
        self.collect_hardlink_groups(&mut groups);

        groups
            .into_iter()
            .filter_map(|(inode_key, files)| {
                if files.len() < 2 {
                    return None; // Not actually hardlinked (nlink > 1 but only one path found in scan)
                }
                let first = files[0];
                let paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();
                let apparent_size = first.size;
                let allocated_size = first.allocated_size;
                let link_count = first.nlink;
                Some(HardlinkGroup {
                    inode_key,
                    paths,
                    apparent_size,
                    allocated_size,
                    link_count,
                })
            })
            .collect()
    }

    /// Returns total allocated size for the tree (sum of allocated_size for all files).
    pub fn total_allocated_size(&self) -> u64 {
        if self.is_dir {
            self.children.iter().map(|c| c.total_allocated_size()).sum()
        } else {
            self.allocated_size
        }
    }

    pub fn generate_storage_insights(&self) -> Vec<StorageInsight> {
        let mut insights = Vec::new();
        let total_size = self.size.max(1);

        // 1. Check for massive individual files (>20% of analyzed size or >1GB)
        let mut largest_files = Vec::new();
        self.collect_largest_files(&mut largest_files);
        largest_files.sort_by(|a, b| b.size.cmp(&a.size));

        if let Some(top_file) = largest_files.first() {
            let pct = (top_file.size as f64 / total_size as f64) * 100.0;
            if pct > 20.0 || top_file.size > 1024 * 1024 * 1024 {
                insights.push(StorageInsight {
                    severity: "high",
                    title: format!(
                        "{} in single file ({})",
                        top_file.formatted_size(),
                        top_file.name
                    ),
                    description: format!(
                        "Consumes {:.1}% of analyzed storage in {}.",
                        pct,
                        top_file.category.display_name()
                    ),
                    why_it_matters: "A single large file is dominating storage. Moving or archiving this single item can reclaim significant disk space.".to_string(),
                    size_bytes: top_file.size,
                    category: top_file.category,
                    action_label: "Open Location",
                    path: top_file.path.clone(),
                });
            }
        }

        // 2. Explain substantial unclassified data
        let mut category_stats = std::collections::HashMap::new();
        self.collect_category_stats(&mut category_stats);
        if let Some((count, other_size, _alloc)) = category_stats.get(&FileCategory::Other) {
            let pct = (*other_size as f64 / total_size as f64) * 100.0;
            if pct >= 10.0 || *other_size >= 512 * 1024 * 1024 {
                insights.push(StorageInsight {
                    severity: "medium",
                    title: format!("{} is Other / Unknown", format_size(*other_size)),
                    description: format!("{:.1}% of storage across {} files has no recognized extension.", pct, count),
                    why_it_matters: "Linux applications often store flatpak runtimes, databases, or cache blobs without extensions. Review the extension breakdown to inspect them safely.".to_string(),
                    size_bytes: *other_size,
                    category: FileCategory::Other,
                    action_label: "Investigate types",
                    path: self.path.clone(),
                });
            }
        }

        // 3. Check for Cache & Temporary folders
        for child in &self.children {
            if child.is_dir {
                let name = child.name.to_lowercase();
                if name == ".cache" || name == "cache" || name == "tmp" || name == ".tmp" {
                    insights.push(StorageInsight {
                        severity: "medium",
                        title: format!("{} in application caches ({})", child.formatted_size(), child.name),
                        description: "Temporary data and application caches that can be rebuilt.".to_string(),
                        why_it_matters: "Browsers, package managers, and build tools accumulate gigabytes of cache over time that can often be safely trimmed.".to_string(),
                        size_bytes: child.size,
                        category: FileCategory::SystemConfig,
                        action_label: "Review Cache",
                        path: child.path.clone(),
                    });
                } else if name == "target"
                    || name == "node_modules"
                    || name == ".venv"
                    || name == "venv"
                {
                    insights.push(StorageInsight {
                        severity: "medium",
                        title: format!("{} in build artifacts ({})", child.formatted_size(), child.name),
                        description: "Development build environment containing compiled binaries.".to_string(),
                        why_it_matters: "Intermediate build artifacts and third-party dependencies consume extensive storage and can be cleaned via cargo clean / npm prune.".to_string(),
                        size_bytes: child.size,
                        category: FileCategory::Code,
                        action_label: "Explore Project",
                        path: child.path.clone(),
                    });
                }
            }
        }

        // 4. Check for dominant single subfolders (>45% of storage)
        for child in &self.children {
            if child.is_dir {
                let pct = (child.size as f64 / total_size as f64) * 100.0;
                if pct > 45.0 && !child.name.starts_with('.') {
                    let files_inside = child.file_count;
                    insights.push(StorageInsight {
                        severity: "info",
                        title: format!("{} in folder '{}'", child.formatted_size(), child.name),
                        description: format!("Dominates storage with {:.1}% of space across {} items.", pct, files_inside),
                        why_it_matters: "This directory contains the bulk of your stored files. Inspecting subdirectories will quickly uncover which folders are growing largest.".to_string(),
                        size_bytes: child.size,
                        category: FileCategory::Other,
                        action_label: "Explore Folder",
                        path: child.path.clone(),
                    });
                }
            }
        }

        insights
    }

    pub fn estimate_recoverable_space(&self) -> u64 {
        let mut recoverable: u64 = 0;
        for child in &self.children {
            if child.is_dir {
                let name = child.name.to_lowercase();
                if name == ".cache" || name == "tmp" || name == ".tmp" {
                    recoverable += child.size;
                }
            }
        }
        recoverable
    }

    pub fn collect_category_stats(
        &self,
        stats: &mut std::collections::HashMap<FileCategory, (u64, u64, u64)>,
    ) {
        if !self.is_dir {
            let entry = stats.entry(self.category).or_insert((0, 0, 0));
            entry.0 += 1;
            entry.1 += self.size;
            entry.2 += self.allocated_size;
        } else {
            for child in &self.children {
                child.collect_category_stats(stats);
            }
        }
    }

    pub fn collect_extension_stats(
        &self,
        stats: &mut std::collections::HashMap<String, (u64, u64, u64, FileCategory)>,
    ) {
        if !self.is_dir {
            let ext = self
                .extension
                .clone()
                .unwrap_or_else(|| "no extension".to_string());
            let entry = stats.entry(ext).or_insert((0, 0, 0, self.category));
            entry.0 += 1;
            entry.1 += self.size;
            entry.2 += self.allocated_size;
        } else {
            for child in &self.children {
                child.collect_extension_stats(stats);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileInfo {
    pub name: String,
    pub path: PathBuf,
    /// Apparent size (logical file size)
    pub size: u64,
    /// Actual disk allocation (st_blocks * 512)
    pub allocated_size: u64,
    pub extension: Option<String>,
    pub category: FileCategory,
    #[serde(skip)]
    pub recommendation: CleanupRecommendation,
    #[serde(skip)]
    pub is_cache_or_temp: bool,
    #[serde(skip)]
    pub is_review_needed: bool,
    #[serde(skip)]
    pub risk_note: String,
    #[serde(skip)]
    pub modified: Option<SystemTime>,
    #[serde(skip)]
    pub inode: Option<u64>,
    #[serde(skip)]
    pub device: Option<u64>,
    #[serde(skip)]
    pub nlink: u32,
}

impl Default for CleanupRecommendation {
    fn default() -> Self {
        CleanupRecommendation::ReviewNeeded
    }
}

impl LargeFileInfo {
    pub fn formatted_size(&self) -> String {
        format_size(self.size)
    }

    pub fn formatted_allocated_size(&self) -> String {
        format_size(self.allocated_size)
    }

    pub fn formatted_modified(&self) -> String {
        match self.modified {
            Some(time) => {
                let dt: DateTime<Local> = time.into();
                dt.format("%Y-%m-%d %H:%M").to_string()
            }
            None => "-".to_string(),
        }
    }

    /// Returns the inode key for hardlink grouping, if available.
    pub fn inode_key(&self) -> Option<InodeKey> {
        match (self.inode, self.device) {
            (Some(inode), Some(device)) => Some(InodeKey::new(device, inode)),
            _ => None,
        }
    }

    /// Returns true if this file has hardlinks (nlink > 1).
    pub fn is_hardlink(&self) -> bool {
        self.nlink > 1
    }

    /// Returns the space savings from deduplicating hardlinks (allocated_size * (nlink - 1)).
    pub fn hardlink_waste(&self) -> u64 {
        if self.nlink > 1 {
            self.allocated_size * (self.nlink - 1) as u64
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: FileCategory,
    pub count: u64,
    /// Total apparent size (sum of st_size)
    pub total_size: u64,
    /// Total allocated size (sum of st_blocks * 512)
    pub total_allocated_size: u64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionSummary {
    pub extension: String,
    pub count: u64,
    /// Total apparent size (sum of st_size)
    pub total_size: u64,
    /// Total allocated size (sum of st_blocks * 512)
    pub total_allocated_size: u64,
    pub category: FileCategory,
    pub percentage: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanIssueType {
    PermissionDenied,
    BrokenSymlink,
    IoTimeout,
    InaccessiblePath,
}

impl ScanIssueType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "Permission Denied",
            Self::BrokenSymlink => "Broken Symlink",
            Self::IoTimeout => "I/O Timeout",
            Self::InaccessiblePath => "Inaccessible Path",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanIssue {
    pub id: String,
    pub path: PathBuf,
    pub issue_type: ScanIssueType,
    pub severity: &'static str, // "error" | "warning"
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub scanned_files: u64,
    pub scanned_dirs: u64,
    pub scanned_bytes: u64,
    pub current_path: String,
    pub elapsed_secs: f64,
    pub is_complete: bool,
    pub is_paused: bool,
    pub is_cancelled: bool,
    pub error_count: u64,
    pub issues: Vec<ScanIssue>,
}

impl Default for ScanProgress {
    fn default() -> Self {
        Self {
            scanned_files: 0,
            scanned_dirs: 0,
            scanned_bytes: 0,
            current_path: String::new(),
            elapsed_secs: 0.0,
            is_complete: false,
            is_paused: false,
            is_cancelled: false,
            error_count: 0,
            issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Size,
    ItemCount,
    Type,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisCache {
    pub largest_files: Vec<LargeFileInfo>,
    pub category_summaries: Vec<CategorySummary>,
    pub extension_summaries: Vec<ExtensionSummary>,
    pub insights: Vec<StorageInsight>,
    pub recoverable_bytes: u64,
    /// Total apparent size of the analyzed tree
    pub total_apparent_size: u64,
    /// Total allocated size of the analyzed tree (accounts for sparseness, compression)
    pub total_allocated_size: u64,
    /// Hardlink groups for deduplication analysis
    pub hardlink_groups: Vec<HardlinkGroup>,
    /// Total bytes wasted by hardlinks (allocated_size * (nlink - 1) for each group)
    pub hardlink_waste_bytes: u64,
}

impl AnalysisCache {
    pub fn build(root: &FileNode) -> Self {
        let mut cat_stats = std::collections::HashMap::new();
        root.collect_category_stats(&mut cat_stats);

        let mut ext_stats = std::collections::HashMap::new();
        root.collect_extension_stats(&mut ext_stats);

        let total_apparent_size = root.size.max(1);
        let total_allocated_size = root.total_allocated_size().max(1);

        let mut category_summaries: Vec<CategorySummary> = cat_stats
            .into_iter()
            .map(|(cat, (count, app_size, alloc_size))| CategorySummary {
                category: cat,
                count,
                total_size: app_size,
                total_allocated_size: alloc_size,
                percentage: (app_size as f32 / total_apparent_size as f32) * 100.0,
            })
            .collect();
        category_summaries.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        let mut extension_summaries: Vec<ExtensionSummary> = ext_stats
            .into_iter()
            .map(|(ext, (count, app_size, alloc_size, cat))| ExtensionSummary {
                extension: ext,
                count,
                total_size: app_size,
                total_allocated_size: alloc_size,
                category: cat,
                percentage: (app_size as f32 / total_apparent_size as f32) * 100.0,
            })
            .collect();
        extension_summaries.sort_by(|a, b| b.total_size.cmp(&a.total_size));

        let mut largest_files = Vec::new();
        root.collect_largest_files(&mut largest_files);
        largest_files.sort_by(|a, b| b.size.cmp(&a.size));

        let hardlink_groups = root.build_hardlink_groups();
        let hardlink_waste_bytes: u64 = hardlink_groups
            .iter()
            .map(|g| g.allocated_size * (g.link_count - 1) as u64)
            .sum();

        let insights = root.generate_storage_insights();
        let recoverable_bytes = root.estimate_recoverable_space();

        Self {
            largest_files,
            category_summaries,
            extension_summaries,
            insights,
            recoverable_bytes,
            total_apparent_size,
            total_allocated_size,
            hardlink_groups,
            hardlink_waste_bytes,
        }
    }
}
