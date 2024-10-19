use std::path::Path;
use crate::database::fetch_text_with_id;

pub mod database;

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

/// Load given text from database with given id.
/// # Arguments
/// * `text_id` - ID of text to load
/// $ Returns 
/// * `Result<FileText>` containing file contents or error message
pub fn load_text_from_database(text_id: u32) -> Result<PreparedText, String> {
    let row_count = 6000;
    if 1 <= text_id && text_id <= row_count {
        let text = fetch_text_with_id(text_id, "data.db")
            .map_err(|e| format!("Error fetching text: {}", e))?;
        Ok((text, text_id.to_string()))
    } else {
        Err(format!("ID out of range: {}", text_id))
    }
}
