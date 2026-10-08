# 🔬 DiskLens - Native Rust Disk Space Analyzer for Linux / Ubuntu

**DiskLens** is a high-performance, 100% native Linux desktop GUI application written in **Rust** using `eframe`/`egui` with hardware-accelerated OpenGL / X11 / Wayland rendering. It provides rapid multi-threaded disk usage exploration, interactive visual treemaps, directory breakdowns, and safe storage cleanup.

---

## 🌟 Key Features

1. **📊 Interactive Squarified Treemap**
   - Visualizes space usage with aspect-ratio-optimized squarified rectangles.
   - **Click to Drill Down**: Zoom into any directory to analyze its subfolders.
   - **Hover Inspection**: Instant tooltips with full file path, exact size, % share, and modification date.
   - **Adjustable Detail**: Sliders to control visual treemap recursion depth (1–6 levels).

2. **📁 Sortable Directory Breakdown Tree**
   - Full hierarchy explorer with visual proportional usage bars.
   - Sort by **Size**, **Name**, **Item Count**, **Type**, or **Modified Date**.
   - Built-in search filtering and minimum size threshold filter.
   - Breadcrumb bar for effortless navigation up/down the file tree.

3. **⚡ Top Largest Files (Space Hog Finder)**
   - Displays Top 50, 100, 250, 500, or 1000 largest files across the scanned hierarchy.
   - Filter by file category (Videos, Images, Archives, Binaries, etc.) or custom search query.

4. **🏷️ File Types & Extensions Distribution**
   - Multi-colored space distribution bar.
   - Category summary cards (Videos, Images, Archives, Code, Documents, Audio, etc.).
   - Extension breakdown table with file count and total size percentages.

5. **💾 Drives & Partitions Overview**
   - Auto-detects all mounted storage devices, SSDs, HDDs, and partitions.
   - Visual health and capacity gauges (<70% green, 70-85% warning, >85% critical).
   - One-click **"Scan Drive"** button.

6. **🛠️ File Management Actions & Context Menu**
   - Right-click on any file or folder:
     - 📂 **Open in File Manager** (Nautilus, Dolphin, Nemo, Thunar)
     - 📄 **Open File** with default application
     - 🖥️ **Open Terminal Here**
     - 📋 **Copy Full Path** to clipboard
     - 🗑️ **Move to Trash** (Safe deletion adhering to Freedesktop standards)
     - 💥 **Permanent Delete** (with safety confirmation dialog)

7. **⚡ Multi-Threaded Non-Blocking Scanner**
   - Real-time live statistics: Scanned files, directories, throughput (items/sec), and elapsed time.
   - **Pause**, **Resume**, and **Cancel** support.
   - Handles symlinks and filesystem permission restrictions gracefully.

8. **💾 Export Reports**
   - Export full scan metadata to **JSON**.
   - Export largest files to **CSV**.

9. **🌙 Modern UI & Theming**
   - Sleek Dark Mode (default) and Light Mode toggle.

---

## 🚀 How to Run

### Method 1: Application Menu (Ubuntu / Zorin / GNOME)
Search for **"DiskLens"** in your application launcher / start menu and click to launch.

### Method 2: Command Line (Direct Binary or Script)
```bash
# Using the launcher script
/home/sayantan/disk-analyzer-gui/run.sh

# Or directly running the compiled release binary
/home/sayantan/disk-analyzer-gui/target/release/disk-lens
```

### Method 3: Development Run
```bash
cd /home/sayantan/disk-analyzer-gui
cargo run --release
```

---

## 🏗️ Project Architecture

```
disk-analyzer-gui/
├── Cargo.toml                  # Rust dependencies & configuration
├── run.sh                      # Quick launcher script
├── README.md                   # Complete documentation
└── src/
    ├── main.rs                 # Native window initialization & entry point
    ├── app.rs                  # Main application state, toolbar, tabs & loop
    ├── models.rs               # FileNode tree, categories, progress & formatting
    ├── scanner.rs              # Multi-threaded background scanning engine
    ├── treemap.rs              # Squarified Treemap layout algorithm & color palette
    ├── theme.rs                # Modern Dark & Light theme visuals
    ├── actions.rs              # Linux desktop integrations (file manager, terminal, trash)
    ├── export.rs               # JSON & CSV export functionality
    └── ui/
        ├── mod.rs
        ├── treemap_view.rs     # Treemap canvas rendering & interactive zoom
        ├── tree_view.rs        # Sortable directory table with usage bars
        ├── largest_files_view.rs # Top largest files view & filters
        ├── categories_view.rs  # Category cards & extension breakdown
        ├── drives_view.rs      # System drives & partition usage gauges
        └── modals.rs           # Confirmations, trash modals & toast alerts
```
