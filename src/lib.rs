use std::path::Path;

pub mod database;
pub mod app;
pub mod calculations;
pub mod timer;
pub mod history;

pub type PreparedText = (String, String);

/// Load file contents
/// # Arguments
/// * `file_path` - Path to file
/// # Returns
/// * `Result<FileText>` containing file contents or error message
pub fn load_text_from_file<P: AsRef<Path>>(file_path: P) -> Result<PreparedText, String> {
    if std::fs::exists(&file_path)
        .map_err(|e| format!("Error checking file: {}", e))? {
        let text = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Error reading file: {}", e))?;
        Ok((text, file_path.as_ref().display().to_string()))
    } else {
        Err(format!("File not found: {}", file_path.as_ref().display()))
    }
}

