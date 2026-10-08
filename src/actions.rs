use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

/// Abstract interface for file system side-effects and external shell actions.
pub trait FileSystem: Send + Sync {
    fn open_in_file_manager(&self, path: &Path) -> Result<(), String>;
    fn open_file(&self, path: &Path) -> Result<(), String>;
    fn open_terminal(&self, path: &Path) -> Result<(), String>;
    fn move_to_trash(&self, path: &Path) -> Result<(), String>;
    fn permanent_delete(&self, path: &Path) -> Result<(), String>;
}

/// Real operating system file system operations.
#[derive(Debug, Default, Clone, Copy)]
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn open_in_file_manager(&self, path: &Path) -> Result<(), String> {
        FileActions::open_in_file_manager(path)
    }

    fn open_file(&self, path: &Path) -> Result<(), String> {
        FileActions::open_file(path)
    }

    fn open_terminal(&self, path: &Path) -> Result<(), String> {
        FileActions::open_terminal(path)
    }

    fn move_to_trash(&self, path: &Path) -> Result<(), String> {
        FileActions::move_to_trash(path)
    }

    fn permanent_delete(&self, path: &Path) -> Result<(), String> {
        FileActions::permanent_delete(path)
    }
}

/// In-memory mock file system for unit tests and headless environments.
#[derive(Debug, Default)]
pub struct MockFileSystem {
    pub opened_paths: Mutex<Vec<PathBuf>>,
    pub trashed_paths: Mutex<Vec<PathBuf>>,
    pub deleted_paths: Mutex<Vec<PathBuf>>,
    pub simulate_error: Mutex<Option<String>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_error(error: String) -> Self {
        Self {
            simulate_error: Mutex::new(Some(error)),
            ..Default::default()
        }
    }
}

impl FileSystem for MockFileSystem {
    fn open_in_file_manager(&self, path: &Path) -> Result<(), String> {
        if let Some(err) = self.simulate_error.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        self.opened_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn open_file(&self, path: &Path) -> Result<(), String> {
        if let Some(err) = self.simulate_error.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        self.opened_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn open_terminal(&self, path: &Path) -> Result<(), String> {
        if let Some(err) = self.simulate_error.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        self.opened_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn move_to_trash(&self, path: &Path) -> Result<(), String> {
        if let Some(err) = self.simulate_error.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        self.trashed_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }

    fn permanent_delete(&self, path: &Path) -> Result<(), String> {
        if let Some(err) = self.simulate_error.lock().unwrap().as_ref() {
            return Err(err.clone());
        }
        self.deleted_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }
}

pub struct FileActions;

impl FileActions {
    pub fn open_in_file_manager(path: &Path) -> Result<(), String> {
        let target = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        match open::that(target) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open file manager: {}", e)),
        }
    }

    pub fn open_file(path: &Path) -> Result<(), String> {
        match open::that(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open file: {}", e)),
        }
    }

    pub fn open_terminal(path: &Path) -> Result<(), String> {
        let dir = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        // Try standard Linux terminal emulators
        let terminals = [
            "xdg-terminal-exec",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "kitty",
            "alacritty",
            "xterm",
        ];
        for term in &terminals {
            if let Ok(mut child) = Command::new(term).current_dir(dir).spawn() {
                // Spawned successfully
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                return Ok(());
            }
        }

        // Fallback: try opening with generic opener
        match open::that(dir) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to spawn terminal: {}", e)),
        }
    }

    pub fn move_to_trash(path: &Path) -> Result<(), String> {
        match trash::delete(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to move to trash: {}", e)),
        }
    }

    pub fn permanent_delete(path: &Path) -> Result<(), String> {
        if path.is_dir() {
            match fs::remove_dir_all(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to remove directory: {}", e)),
            }
        } else {
            match fs::remove_file(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to remove file: {}", e)),
            }
        }
    }
}
