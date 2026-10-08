use crate::models::{CategorySummary, ExtensionSummary, FileNode, LargeFileInfo};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Exporter;

impl Exporter {
    pub fn export_json(
        path: &Path,
        root: &FileNode,
        categories: &[CategorySummary],
        extensions: &[ExtensionSummary],
        largest: &[LargeFileInfo],
    ) -> Result<(), String> {
        #[derive(serde::Serialize)]
        struct ExportData<'a> {
            scan_root: &'a FileNode,
            categories: &'a [CategorySummary],
            extensions: &'a [ExtensionSummary],
            largest_files: &'a [LargeFileInfo],
        }

        let data = ExportData {
            scan_root: root,
            categories,
            extensions,
            largest_files: largest,
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        file.write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    pub fn export_csv(path: &Path, largest: &[LargeFileInfo]) -> Result<(), String> {
        let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        writeln!(
            file,
            "Name,Path,Size (Bytes),Formatted Size,Category,Modified"
        )
        .map_err(|e| format!("Failed to write CSV header: {}", e))?;

        for item in largest {
            let name_escaped = item.name.replace('"', "\"\"");
            let path_escaped = item.path.display().to_string().replace('"', "\"\"");
            let formatted_size = item.formatted_size();
            let category = item.category.display_name();
            let modified = item.formatted_modified();

            writeln!(
                file,
                "\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\"",
                name_escaped, path_escaped, item.size, formatted_size, category, modified
            )
            .map_err(|e| format!("Failed to write CSV row: {}", e))?;
        }

        Ok(())
    }
}
