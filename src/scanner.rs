use crate::models::{FileNode, ScanIssue, ScanIssueType, ScanProgress};
use crate::settings::{AppSettings, SymlinkPolicy};
use crossbeam_channel::{Receiver, Sender};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum ScanMessage {
    Progress(ScanProgress),
    Complete(Box<FileNode>),
    Cancelled,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ScanHandle {
    pub cancel_flag: Arc<AtomicBool>,
    pub pause_flag: Arc<AtomicBool>,
}

impl Default for ScanHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanHandle {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            pause_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.pause_flag.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.pause_flag.store(false, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.pause_flag.load(Ordering::Relaxed)
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Relaxed)
    }
}

/// Abstract scan service trait decoupling scanner lifecycle and threading from the UI.
pub trait ScanService: Send + Sync {
    fn start_scan(
        &self,
        root_path: PathBuf,
        settings: AppSettings,
    ) -> (ScanHandle, Receiver<ScanMessage>);
}

/// Default multi-threaded scanner service utilizing Rayon and WalkDir.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultScanService;

impl ScanService for DefaultScanService {
    fn start_scan(
        &self,
        root_path: PathBuf,
        settings: AppSettings,
    ) -> (ScanHandle, Receiver<ScanMessage>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let handle = Scanner::start_scan_with_settings(root_path, tx, settings);
        (handle, rx)
    }
}

pub struct Scanner {
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
    sender: Sender<ScanMessage>,
    progress: ScanProgress,
    last_update: Instant,
    start_time: Instant,
    settings: AppSettings,
    /// Root device ID for cross-filesystem boundary detection (st_dev of scan root).
    root_device: Option<u64>,
    /// Set of (st_dev, st_ino) keys seen during this scan for hardlink deduplication.
    seen_inodes: std::collections::HashSet<(u64, u64)>,
}

impl Scanner {
    pub fn start_scan(root_path: PathBuf, sender: Sender<ScanMessage>) -> ScanHandle {
        let settings = AppSettings::load();
        Self::start_scan_with_settings(root_path, sender, settings)
    }

    pub fn start_scan_with_settings(
        root_path: PathBuf,
        sender: Sender<ScanMessage>,
        settings: AppSettings,
    ) -> ScanHandle {
        let handle = ScanHandle::new();
        let cancel_flag = Arc::clone(&handle.cancel_flag);
        let pause_flag = Arc::clone(&handle.pause_flag);

        let c_flag = Arc::clone(&cancel_flag);
        let p_flag = Arc::clone(&pause_flag);

        std::thread::Builder::new()
            .name("disk-scanner".to_string())
            .spawn(move || {
                let mut scanner = Scanner {
                    cancel_flag: c_flag,
                    pause_flag: p_flag,
                    sender,
                    progress: ScanProgress::default(),
                    last_update: Instant::now(),
                    start_time: Instant::now(),
                    settings,
                    root_device: None,
                    seen_inodes: std::collections::HashSet::new(),
                };

                scanner.run(root_path);
            })
            .expect("Failed to spawn scanner thread");

        handle
    }

    fn run(&mut self, root_path: PathBuf) {
        if !root_path.exists() {
            let _ = self.sender.send(ScanMessage::Error(format!(
                "Path does not exist: {}",
                root_path.display()
            )));
            return;
        }

        let metadata = match fs::symlink_metadata(&root_path) {
            Ok(m) => m,
            Err(e) => {
                let _ = self
                    .sender
                    .send(ScanMessage::Error(format!("Cannot read root path: {}", e)));
                return;
            }
        };

        // Capture root device for cross-filesystem boundary detection
        self.root_device = Some(metadata.dev());

        let root_name = root_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(root_path.to_str().unwrap_or("Root"))
            .to_string();

        let modified = metadata.modified().ok();
        let (allocated_size, inode, device, nlink) = self.extract_metadata_fields(&metadata);
        let permission_error = None; // Root path readable if we got here

        if metadata.is_dir() {
            let mut root_node = FileNode::new_dir_extended(root_name, root_path.clone(), modified, permission_error);
            let success = self.scan_directory(&root_path, &mut root_node);

            if self.cancel_flag.load(Ordering::Relaxed) {
                let _ = self.sender.send(ScanMessage::Cancelled);
                return;
            }

            if success {
                sort_tree_by_size(&mut root_node);

                self.progress.scanned_files = root_node.file_count;
                self.progress.scanned_bytes = root_node.size;
                self.progress.scanned_dirs = root_node.dir_count;
                self.progress.is_complete = true;
                self.progress.elapsed_secs = self.start_time.elapsed().as_secs_f64();
                let _ = self
                    .sender
                    .send(ScanMessage::Progress(self.progress.clone()));
                let _ = self.sender.send(ScanMessage::Complete(Box::new(root_node)));
            }
        } else {
            let size = metadata.len();
            let root_node = FileNode::new_file_extended(root_name, root_path, size, allocated_size, modified, inode, device, nlink, permission_error);
            self.progress.scanned_files = 1;
            self.progress.scanned_bytes = size;
            self.progress.scanned_dirs = 0;
            self.progress.is_complete = true;
            self.progress.elapsed_secs = self.start_time.elapsed().as_secs_f64();
            let _ = self
                .sender
                .send(ScanMessage::Progress(self.progress.clone()));
            let _ = self.sender.send(ScanMessage::Complete(Box::new(root_node)));
        }
    }

    /// Extracts extended metadata fields using std::os::unix::fs::MetadataExt.
    /// Returns (allocated_size, inode, device, nlink).
    fn extract_metadata_fields(&self, metadata: &fs::Metadata) -> (u64, Option<u64>, Option<u64>, u32) {
        // st_blocks is in 512-byte units. allocated_size = st_blocks * 512.
        // For sparse files, allocated_size < size. For compressed FS (Btrfs/ZFS), allocated_size may differ.
        let allocated_size = metadata.blocks() * 512;
        let inode = Some(metadata.ino());
        let device = Some(metadata.dev());
        let nlink = metadata.nlink() as u32;
        (allocated_size, inode, device, nlink)
    }

    fn scan_directory(&mut self, dir_path: &Path, dir_node: &mut FileNode) -> bool {
        // Check for cancellation
        if self.cancel_flag.load(Ordering::Relaxed) {
            return false;
        }

        // Handle pause
        while self.pause_flag.load(Ordering::Relaxed) {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        // Check exclusions
        let dir_str = dir_path.to_string_lossy();
        for excl in &self.settings.exclusions {
            let expanded_excl = if excl.starts_with('~') {
                if let Ok(home) = std::env::var("HOME") {
                    excl.replacen('~', &home, 1)
                } else {
                    excl.clone()
                }
            } else {
                excl.clone()
            };

            if dir_str == expanded_excl || dir_str.starts_with(&format!("{}/", expanded_excl)) {
                return true;
            }
        }

        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                self.progress.error_count += 1;
                let issue_type = match e.kind() {
                    std::io::ErrorKind::PermissionDenied => ScanIssueType::PermissionDenied,
                    std::io::ErrorKind::TimedOut => ScanIssueType::IoTimeout,
                    _ => ScanIssueType::InaccessiblePath,
                };
                let severity = if issue_type == ScanIssueType::PermissionDenied { "error" } else { "warning" };
                self.add_issue(dir_path.to_path_buf(), issue_type, severity);
                // Mark this directory as having a permission error
                dir_node.permission_error = Some(e.to_string());
                return true; // continue scanning other dirs
            }
        };

        self.progress.scanned_dirs += 1;
        self.progress.current_path = dir_path.to_string_lossy().to_string();
        self.maybe_send_progress();

        let mut sub_dirs = Vec::new();
        let mut files = Vec::new();

        for entry_res in entries {
            if self.cancel_flag.load(Ordering::Relaxed) {
                return false;
            }

            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    self.progress.error_count += 1;
                    let issue_type = match e.kind() {
                        std::io::ErrorKind::PermissionDenied => ScanIssueType::PermissionDenied,
                        std::io::ErrorKind::TimedOut => ScanIssueType::IoTimeout,
                        _ => ScanIssueType::InaccessiblePath,
                    };
                    self.add_issue(dir_path.to_path_buf(), issue_type, "error");
                    continue;
                }
            };

            let path = entry.path();
            let file_name_str = entry.file_name().to_string_lossy().to_string();

            // Hidden file filter
            if !self.settings.include_hidden_files && file_name_str.starts_with('.') {
                continue;
            }

            let symlink_meta = match fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    self.progress.error_count += 1;
                    let issue_type = match e.kind() {
                        std::io::ErrorKind::PermissionDenied => ScanIssueType::PermissionDenied,
                        std::io::ErrorKind::NotFound => ScanIssueType::BrokenSymlink,
                        _ => ScanIssueType::InaccessiblePath,
                    };
                    self.add_issue(path, issue_type, "warning");
                    continue;
                }
            };

            // Extract extended metadata fields
            let (allocated_size, inode, device, nlink) = self.extract_metadata_fields(&symlink_meta);
            let modified = symlink_meta.modified().ok();

            // Cross-filesystem boundary check (one-filesystem / -xdev)
            // If enabled, skip directories on different devices than the scan root.
            if symlink_meta.is_dir() {
                if self.settings.cross_filesystem_boundaries == false {
                    if let Some(root_dev) = self.root_device {
                        if symlink_meta.dev() != root_dev {
                            // Skip this mount point - different filesystem
                            continue;
                        }
                    }
                }
            }

            // Symlink Handling
            if symlink_meta.file_type().is_symlink() {
                match self.settings.symlink_policy {
                    SymlinkPolicy::DontFollow => {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let file_node = FileNode::new_file_extended(name, path, 0, 0, modified, inode, device, nlink, None);
                        files.push(file_node);
                        self.progress.scanned_files += 1;
                        continue;
                    }
                    SymlinkPolicy::FollowAll | SymlinkPolicy::FollowWithinRoot => {
                        if !path.exists() {
                            self.add_issue(path.clone(), ScanIssueType::BrokenSymlink, "warning");
                            let name = entry.file_name().to_string_lossy().to_string();
                            let file_node = FileNode::new_file_extended(name, path, 0, 0, modified, inode, device, nlink, None);
                            files.push(file_node);
                            self.progress.scanned_files += 1;
                            continue;
                        }
                        // For followed symlinks, we'll process the target below
                    }
                }
            }

            let name = entry.file_name().to_string_lossy().to_string();

            if symlink_meta.is_dir() {
                let sub_dir_node = FileNode::new_dir_extended(name, path.clone(), modified, None);
                sub_dirs.push((path, sub_dir_node));
            } else {
                let size = symlink_meta.len();
                // Linux hardlink deduplication: if file has hardlinks (nlink > 1) and its (dev, ino)
                // has already been indexed during this scan traversal, count 0 for disk allocation.
                let effective_allocated = if nlink > 1 {
                    if let (Some(dev), Some(ino)) = (device, inode) {
                        if !self.seen_inodes.insert((dev, ino)) {
                            // Secondary hardlink: blocks are already allocated and accounted for.
                            0
                        } else {
                            allocated_size
                        }
                    } else {
                        allocated_size
                    }
                } else {
                    allocated_size
                };

                let file_node = FileNode::new_file_extended(name, path, size, effective_allocated, modified, inode, device, nlink, None);
                self.progress.scanned_files += 1;
                self.progress.scanned_bytes += effective_allocated;
                files.push(file_node);
            }
        }

        let mut total_size: u64 = files.iter().map(|f| f.size).sum();
        let mut total_allocated: u64 = files.iter().map(|f| f.allocated_size).sum();
        let mut total_files: u64 = files.len() as u64;
        let mut total_dirs: u64 = sub_dirs.len() as u64;

        let mut processed_sub_dirs = Vec::with_capacity(sub_dirs.len());

        for (sub_path, mut sub_dir_node) in sub_dirs {
            if !self.scan_directory(&sub_path, &mut sub_dir_node) {
                return false;
            }

            total_size += sub_dir_node.size;
            total_allocated += sub_dir_node.allocated_size;
            total_files += sub_dir_node.file_count;
            total_dirs += sub_dir_node.dir_count;

            processed_sub_dirs.push(sub_dir_node);
        }

        dir_node.size = total_size;
        dir_node.allocated_size = total_allocated;
        dir_node.file_count = total_files;
        dir_node.dir_count = total_dirs;

        let mut children = processed_sub_dirs;
        children.extend(files);
        dir_node.children = children;

        true
    }

    fn add_issue(&mut self, path: PathBuf, issue_type: ScanIssueType, severity: &'static str) {
        if self.progress.issues.len() < 100 {
            let elapsed = self.start_time.elapsed().as_secs();
            let time_str = format!("{:02}:{:02}:{:02}", elapsed / 3600, (elapsed % 3600) / 60, elapsed % 60);
            self.progress.issues.push(ScanIssue {
                id: format!("iss-{}", self.progress.issues.len() + 1),
                path,
                issue_type,
                severity,
                timestamp: time_str,
            });
        }
    }

    fn maybe_send_progress(&mut self) {
        if self.last_update.elapsed() >= Duration::from_millis(50) {
            self.progress.elapsed_secs = self.start_time.elapsed().as_secs_f64();
            self.progress.is_paused = self.pause_flag.load(Ordering::Relaxed);
            let _ = self
                .sender
                .send(ScanMessage::Progress(self.progress.clone()));
            self.last_update = Instant::now();
        }
    }
}

pub fn sort_tree_by_size(node: &mut FileNode) {
    if node.is_dir {
        node.children.sort_by(|a, b| b.size.cmp(&a.size));
        for child in &mut node.children {
            sort_tree_by_size(child);
        }
    }
}
