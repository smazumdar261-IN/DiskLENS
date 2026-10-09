# Changelog

All notable changes to **DiskLens** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0-rel] - 2026-10-09

### Added
- **About Page Redesign**: Premium product introduction hero card with night sky artwork, glowing moon, and silhouetted mountain ridges.
- **Open Capability Highlights**: 3-column open summaries for Fast Analysis, Safe by Design, and Native Linux.
- **Developer Credit**: Credited developer Sayantan Mazumdar — Developer & Maintainer with circular 'S' avatar, bio, and social actions.
- **4-Column Application Information**: Compact system panel showing Version, Build Target (`Linux (x86_64)`), Engine (`Rust / egui`), and License (`GPL-3.0`).
- **Sidebar Redesign**: Brand header with squircle drive glyph, grouped navigation (`EXPLORE`, `SYSTEM`, `SETTINGS`), Lucide-style vector outline icons, and vibrant blue capsule active pills (`#2563EB`).
- **Non-Blocking Safe Trash Traversal**: Asynchronous background discovery for cache directories preventing UI lockups and OS "Force Quit or Wait" dialogs.
- **Centralized Metadata**: Single source of truth in `src/metadata.rs` for versioning, developer details, offline GPL-3.0 text, and GitHub links.
- **Interactive License Modal**: In-app viewer for the GNU General Public License v3.0.
- **Comprehensive Integration Tests**: Test suite covering responsive rendering, navigation lifecycle, and metadata integrity.

### Fixed
- Fixed UI freeze in Safe Trash when enumerating large directories (>300,000 files in `~/.cache`).
- Fixed table and card clipping on narrow window resolutions.
- Removed redundant technical bento grids and permanent keyboard shortcut panels from the About view.
