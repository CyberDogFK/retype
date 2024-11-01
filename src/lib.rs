use std::path::Path;

pub mod app;
pub mod calculations;
pub mod database;
pub mod history;
pub mod keycheck;
pub mod timer;

#[derive(Debug)]
pub enum FileError {
    IoError(String, std::io::Error),
    FileDoesNotExist(String),
    FileReadingError(String, std::io::Error),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::IoError(path, e) => {
                write!(f, "An IO error occurred for file: {}, {}", path, e)
            }
            FileError::FileDoesNotExist(path) => {
                write!(f, "The file does not exist: {}", path)
            }
            FileError::FileReadingError(path, e) => {
                write!(f, "Error reading file: {}, {}", path, e)
            }
        }
    }
}

pub type PreparedText = (String, String);

/// Load file contents
/// # Arguments
/// * `file_path` - Path to file
/// # Returns
/// * `Result<FileText>` containing file contents or error message
pub fn load_text_from_file<P: AsRef<Path>>(file_path: P) -> Result<PreparedText, FileError> {
    let get_path = || { file_path.as_ref().display().to_string() };
    if std::fs::exists(&file_path).map_err(|e| FileError::IoError(get_path(), e))? {
        let text = std::fs::read_to_string(&file_path)
            .map_err(|e| FileError::FileReadingError(get_path(), e))?;
        Ok((text, file_path.as_ref().display().to_string()))
    } else {
        Err(FileError::FileDoesNotExist(get_path()))
    }
}
