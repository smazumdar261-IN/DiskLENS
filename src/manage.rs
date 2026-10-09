use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CleanupTargetItem {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub age_days: u32,
    pub is_removable: bool,
}

#[derive(Debug, Clone)]
pub struct CleanupTargetCategory {
    pub id: String,
    pub title: String,
    pub description: String,
    pub requires_sudo: bool,
    pub is_selected: bool,
    pub items: Vec<CleanupTargetItem>,
}

impl CleanupTargetCategory {
    pub fn total_reclaimable_bytes(&self) -> u64 {
        self.items
            .iter()
            .filter(|i| i.is_removable)
            .map(|i| i.size_bytes)
            .sum()
    }
}

pub struct StorageManager;

impl StorageManager {
    /// Scans real Linux cache and reclaimable storage directories safely without deleting.
    pub fn discover_cleanable_targets() -> Vec<CleanupTargetCategory> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        let mut categories = Vec::new();

        // 1. User Application Caches (~/.cache)
        let cache_dir = PathBuf::from(&home).join(".cache");
        if cache_dir.exists() {
            let items = Self::scan_directory_top_level_items(&cache_dir);
            if !items.is_empty() {
                categories.push(CleanupTargetCategory {
                    id: "user-cache".to_string(),
                    title: "User Application Caches".to_string(),
                    description: "Browser caches, thumbnail databases, and application temporary state in ~/.cache".to_string(),
                    requires_sudo: false,
                    is_selected: true,
                    items,
                });
            }
        }

        // 2. Developer Build Artifacts & Package Caches
        let mut dev_items = Vec::new();

        // Cargo cache
        let cargo_cache = PathBuf::from(&home).join(".cargo/registry/cache");
        if cargo_cache.exists() {
            let size = Self::calculate_dir_size(&cargo_cache);
            if size > 0 {
                dev_items.push(CleanupTargetItem {
                    name: "Rust Cargo Package Cache (.cargo/registry/cache)".to_string(),
                    path: cargo_cache,
                    size_bytes: size,
                    age_days: 14,
                    is_removable: true,
                });
            }
        }

        // npm / pnpm / yarn cache
        let npm_cache = PathBuf::from(&home).join(".npm/_cacache");
        if npm_cache.exists() {
            let size = Self::calculate_dir_size(&npm_cache);
            if size > 0 {
                dev_items.push(CleanupTargetItem {
                    name: "Node.js npm Package Cache (.npm/_cacache)".to_string(),
                    path: npm_cache,
                    size_bytes: size,
                    age_days: 21,
                    is_removable: true,
                });
            }
        }

        // Python pip cache
        let pip_cache = PathBuf::from(&home).join(".cache/pip");
        if pip_cache.exists() {
            let size = Self::calculate_dir_size(&pip_cache);
            if size > 0 {
                dev_items.push(CleanupTargetItem {
                    name: "Python pip Wheel & HTTP Cache (.cache/pip)".to_string(),
                    path: pip_cache,
                    size_bytes: size,
                    age_days: 30,
                    is_removable: true,
                });
            }
        }

        if !dev_items.is_empty() {
            categories.push(CleanupTargetCategory {
                id: "dev-cache".to_string(),
                title: "Development Package & Toolchain Caches".to_string(),
                description: "Downloaded crate archives, npm packages, and pip build wheels"
                    .to_string(),
                requires_sudo: false,
                is_selected: true,
                items: dev_items,
            });
        }

        // 3. FreeDesktop Staged Trash (~/.local/share/Trash)
        let trash_dir = PathBuf::from(&home).join(".local/share/Trash/files");
        if trash_dir.exists() {
            let items = Self::scan_directory_top_level_items(&trash_dir);
            if !items.is_empty() {
                categories.push(CleanupTargetCategory {
                    id: "user-trash".to_string(),
                    title: "FreeDesktop Staged Trash Bin".to_string(),
                    description: "Files currently residing in the user Trash staging sandbox (~/.local/share/Trash)".to_string(),
                    requires_sudo: false,
                    is_selected: false,
                    items,
                });
            }
        }

        // 4. System-level Package Caches (Pacman / APT)
        let mut sys_items = Vec::new();
        let pacman_pkg = PathBuf::from("/var/cache/pacman/pkg");
        if pacman_pkg.exists() {
            let size = Self::calculate_dir_size(&pacman_pkg);
            if size > 0 {
                sys_items.push(CleanupTargetItem {
                    name: "Arch / Pacman Package Archives (/var/cache/pacman/pkg)".to_string(),
                    path: pacman_pkg,
                    size_bytes: size,
                    age_days: 60,
                    is_removable: true,
                });
            }
        }

        let apt_archives = PathBuf::from("/var/cache/apt/archives");
        if apt_archives.exists() {
            let size = Self::calculate_dir_size(&apt_archives);
            if size > 0 {
                sys_items.push(CleanupTargetItem {
                    name: "Debian / Ubuntu APT Downloaded .deb Archives (/var/cache/apt/archives)"
                        .to_string(),
                    path: apt_archives,
                    size_bytes: size,
                    age_days: 45,
                    is_removable: true,
                });
            }
        }

        if !sys_items.is_empty() {
            categories.push(CleanupTargetCategory {
                id: "system-package-cache".to_string(),
                title: "System Package Manager Caches".to_string(),
                description:
                    "Archived package tarballs in /var/cache. Requires root privileges to purge."
                        .to_string(),
                requires_sudo: true,
                is_selected: false,
                items: sys_items,
            });
        }

        categories
    }

    /// Recursively computes total size of a path in bytes using walkdir.
    pub fn calculate_dir_size(path: &Path) -> u64 {
        if path.is_file() {
            return std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        }

        WalkDir::new(path)
            .follow_links(false)
            .same_file_system(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
            .sum()
    }

    /// Scans top-level entries inside a directory in parallel using Rayon to create granular targets.
    fn scan_directory_top_level_items(dir: &Path) -> Vec<CleanupTargetItem> {
        let entries: Vec<_> = match std::fs::read_dir(dir) {
            Ok(iter) => iter.flatten().collect(),
            Err(_) => return Vec::new(),
        };

        let mut items: Vec<CleanupTargetItem> = entries
            .into_par_iter()
            .filter_map(|entry| {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type().ok()?;
                let size = if file_type.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else if file_type.is_dir() {
                    Self::calculate_dir_size(&p)
                } else {
                    0
                };

                if size > 1024 * 100 {
                    // Only surface items > 100 KiB
                    let age_days = if let Ok(meta) = entry.metadata() {
                        if let Ok(modified) = meta.modified() {
                            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                                (elapsed.as_secs() / 86400) as u32
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    Some(CleanupTargetItem {
                        name,
                        path: p,
                        size_bytes: size,
                        age_days,
                        is_removable: true,
                    })
                } else {
                    None
                }
            })
            .collect();

        items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
        items.truncate(30); // Top 30 items per category
        items
    }

    /// Executes SSD TRIM on the specified mount point safely using the Linux `fstrim` utility.
    pub fn execute_fstrim(mount_point: &Path) -> Result<String, String> {
        let output = Command::new("fstrim").arg("-v").arg(mount_point).output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if msg.is_empty() {
                        Ok(format!(
                            "fstrim completed successfully on {}",
                            mount_point.display()
                        ))
                    } else {
                        Ok(msg)
                    }
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    if err.contains("Permission denied") || err.contains("Operation not permitted")
                    {
                        Err(format!(
                            "fstrim on {} requires root/sudo privileges (run with sudo or polkit)",
                            mount_point.display()
                        ))
                    } else if err.contains("FITRIM ioctl failed: Operation not supported") {
                        Err(format!("SSD TRIM is not supported by the underlying storage hardware or virtual device for {}", mount_point.display()))
                    } else {
                        Err(format!("fstrim failed: {}", err))
                    }
                }
            }
            Err(e) => Err(format!("Could not spawn fstrim binary: {}", e)),
        }
    }

    /// Reads Btrfs subvolume information if the target mount point is a Btrfs filesystem.
    pub fn query_btrfs_subvolumes(
        mount_point: &Path,
    ) -> Result<Vec<(u64, String, String, bool)>, String> {
        let output = Command::new("btrfs")
            .arg("subvolume")
            .arg("list")
            .arg(mount_point)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let mut subvolumes = Vec::new();
                    // btrfs subvolume list output format:
                    // ID 256 gen 1234 top level 5 path @
                    for line in text.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 9 && parts[0] == "ID" {
                            let id: u64 = parts[1].parse().unwrap_or(0);
                            let path = parts[8].to_string();
                            let is_snapshot = path.contains(".snapshots")
                                || path.contains("snapshot")
                                || path.contains("@snap");
                            let name = path.split('/').last().unwrap_or(&path).to_string();
                            subvolumes.push((id, name, path, is_snapshot));
                        }
                    }
                    Ok(subvolumes)
                } else {
                    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    Err(err)
                }
            }
            Err(e) => Err(format!("Could not execute btrfs tool: {}", e)),
        }
    }
}
