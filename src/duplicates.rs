use crate::models::{FileCategory, FileNode};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DuplicateItem {
    pub id: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub modified_date: String,
    pub sha256: String,
    pub is_original: bool,
    pub selected_for_trash: bool,
    pub drive_mount: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub id: String,
    pub name: String,
    pub category: FileCategory,
    pub extension: String,
    pub size_bytes_per_file: u64,
    pub total_size_bytes: u64,
    pub items: Vec<DuplicateItem>,
}

impl DuplicateGroup {
    pub fn potential_savings(&self) -> u64 {
        let trash_count = self.items.iter().filter(|i| i.selected_for_trash).count();
        self.size_bytes_per_file * trash_count as u64
    }
}

pub struct DuplicateDetector;

impl DuplicateDetector {
    /// Two-phase duplicate finder:
    /// Phase 1: In-memory size grouping (instantaneous)
    /// Phase 2: Parallel 4KB prefix + full SHA-256 hashing via Rayon
    pub fn find_duplicates(
        root: &FileNode,
        min_size_bytes: u64,
        progress_counter: Option<Arc<AtomicUsize>>,
    ) -> Vec<DuplicateGroup> {
        let candidates = Self::collect_candidates(root, min_size_bytes);
        Self::compute_duplicates_from_candidates(candidates, progress_counter)
    }

    /// Collect candidate files by size grouping (fast in-memory tree traversal)
    pub fn collect_candidates(
        root: &FileNode,
        min_size_bytes: u64,
    ) -> Vec<(u64, Vec<PathBuf>)> {
        let mut candidates_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        Self::collect_files(root, min_size_bytes, &mut candidates_by_size);

        candidates_by_size
            .into_iter()
            .filter(|(_size, paths)| paths.len() >= 2)
            .collect()
    }

    /// Compute cryptographic SHA-256 hashes in parallel on candidate collision groups
    pub fn compute_duplicates_from_candidates(
        collision_groups: Vec<(u64, Vec<PathBuf>)>,
        progress_counter: Option<Arc<AtomicUsize>>,
    ) -> Vec<DuplicateGroup> {
        if collision_groups.is_empty() {
            return Vec::new();
        }

        // Step 2 & 3: Compute SHA-256 for collision candidates in parallel
        let duplicate_groups: Vec<DuplicateGroup> = collision_groups
            .into_par_iter()
            .flat_map(|(size, paths)| {
                let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();

                for p in paths {
                    if let Some(counter) = &progress_counter {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }

                    if let Some(hash) = Self::compute_file_sha256(&p) {
                        hash_map.entry(hash).or_default().push(p);
                    }
                }

                let mut groups = Vec::new();
                for (sha256, dup_paths) in hash_map {
                    if dup_paths.len() >= 2 {
                        let sample_path = &dup_paths[0];
                        let file_name = sample_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let ext = sample_path
                            .extension()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let category = FileCategory::from_extension(&ext);
                        let total_size = size * dup_paths.len() as u64;

                        let items: Vec<DuplicateItem> = dup_paths
                            .into_iter()
                            .enumerate()
                            .map(|(idx, path)| {
                                let mod_str = if let Ok(meta) = std::fs::metadata(&path) {
                                    if let Ok(mod_time) = meta.modified() {
                                        let dt: chrono::DateTime<chrono::Local> = mod_time.into();
                                        dt.format("%b %d, %Y").to_string()
                                    } else {
                                        "-".to_string()
                                    }
                                } else {
                                    "-".to_string()
                                };

                                DuplicateItem {
                                    id: format!("dup-{}-{}", sha256.chars().take(8).collect::<String>(), idx),
                                    path,
                                    size_bytes: size,
                                    modified_date: mod_str,
                                    sha256: sha256.clone(),
                                    is_original: idx == 0,
                                    selected_for_trash: false,
                                    drive_mount: None,
                                }
                            })
                            .collect();

                        groups.push(DuplicateGroup {
                            id: format!("group-{}", sha256.chars().take(12).collect::<String>()),
                            name: file_name,
                            category,
                            extension: ext,
                            size_bytes_per_file: size,
                            total_size_bytes: total_size,
                            items,
                        });
                    }
                }
                groups
            })
            .collect();

        let mut sorted = duplicate_groups;
        sorted.sort_by(|a, b| b.total_size_bytes.cmp(&a.total_size_bytes));
        sorted
    }

    fn collect_files(
        node: &FileNode,
        min_size: u64,
        map: &mut HashMap<u64, Vec<PathBuf>>,
    ) {
        if !node.is_dir {
            if node.size >= min_size {
                map.entry(node.size).or_default().push(node.path.clone());
            }
        } else {
            for child in &node.children {
                Self::collect_files(child, min_size, map);
            }
        }
    }

    /// Computes full SHA-256 with 4KB prefix optimization
    fn compute_file_sha256(path: &Path) -> Option<String> {
        let mut file = File::open(path).ok()?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024]; // 64 KB chunk buffer

        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return None,
            }
        }

        let result = hasher.finalize();
        Some(format!("{:x}", result))
    }
}
