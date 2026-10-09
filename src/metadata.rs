/// Centralized project metadata for DiskLens.
///
/// Designed for future GitHub publication: all application metadata, developer credits,
/// and external links are centralized here. Missing/unconfigured links have clean `None`
/// placeholders that degrade gracefully without breaking offline usage or fabricating URLs.

#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub app_name: &'static str,
    pub subtitle: &'static str,
    pub description_primary: &'static str,
    pub description_secondary: &'static str,
    pub version: &'static str,
    pub engine: &'static str,
    pub license: &'static str,
    pub developer_name: &'static str,
    pub developer_role: &'static str,
    pub developer_bio: &'static str,

    // Configurable URLs for future publication
    pub repository_url: Option<&'static str>,
    pub documentation_url: Option<&'static str>,
    pub issues_url: Option<&'static str>,
    pub license_url: Option<&'static str>,
    pub github_url: Option<&'static str>,
    pub linkedin_url: Option<&'static str>,
    pub email_url: Option<&'static str>,
}

impl ProjectMetadata {
    /// Returns the global static project metadata instance.
    pub const fn current() -> Self {
        Self {
            app_name: "DiskLens",
            subtitle: "Disk Space Analyzer for Linux",
            description_primary: "A fast, safe and modern disk space analyzer built in Rust.",
            description_secondary: "Find what's using your storage, manage duplicate files, and reclaim space with confidence.",
            version: concat!("v", env!("CARGO_PKG_VERSION"), "-rel"),
            engine: "Rust / egui",
            license: "GPL-3.0",
            developer_name: "Sayantan Mazumdar",
            developer_role: "Developer & Maintainer",
            developer_bio: "Passionate about open source, Linux and building useful desktop tools. Feel free to connect or explore more on GitHub.",

            // Configured repository and profile for GitHub publication
            repository_url: Some("https://github.com/smazumdar261-IN/DiskLENS"),
            documentation_url: Some("https://github.com/smazumdar261-IN/DiskLENS#readme"),
            issues_url: Some("https://github.com/smazumdar261-IN/DiskLENS/issues"),
            license_url: Some("https://github.com/smazumdar261-IN/DiskLENS/blob/main/LICENSE"),
            github_url: Some("https://github.com/smazumdar261-IN"),
            linkedin_url: None,
            email_url: None,
        }
    }

    /// Dynamically computes the build target architecture and OS.
    pub fn build_target(&self) -> String {
        format!("Linux ({})", std::env::consts::ARCH)
    }

    /// Full open-source GPL-3.0 declaration and summary for offline viewing.
    pub fn license_text(&self) -> &'static str {
        "GNU GENERAL PUBLIC LICENSE\n\
        Version 3, 29 June 2007\n\n\
        Copyright (C) 2026 Sayantan Mazumdar\n\n\
        This program is free software: you can redistribute it and/or modify\n\
        it under the terms of the GNU General Public License as published by\n\
        the Free Software Foundation, either version 3 of the License, or\n\
        (at your option) any later version.\n\n\
        This program is distributed in the hope that it will be useful,\n\
        but WITHOUT ANY WARRANTY; without even the implied warranty of\n\
        MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the\n\
        GNU General Public License for more details.\n\n\
        DiskLens adheres to strict user privacy and system isolation:\n\
        - Non-destructive Safe Trash compliance (FreeDesktop.org standard)\n\
        - No remote telemetry, zero analytics, strictly offline\n\
        - High-throughput parallel filesystem indexing in safe Rust"
    }
}

pub static METADATA: ProjectMetadata = ProjectMetadata::current();
