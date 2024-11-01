use std::fmt::Formatter;
use std::path::Path;
use std::time::SystemTimeError;
use crate::database::DatabaseError;

pub mod app;
pub mod calculations;
pub mod database;
pub mod history;
pub mod keycheck;
pub mod timer;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    NoIndexFoundError(usize),
    NoCharFoundError(char),
    ColorNotFoundError(app::Color),
    TimeError(SystemTimeError),
    AppDatabaseError(DatabaseError),
    ParsingError(std::num::ParseIntError),
    AppHistoryError(history::HistoryError),
    TwitterError { url: String , error_description: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NoIndexFoundError(index) => {
                write!(f, "No index found in text: {}", index)
            }
            AppError::NoCharFoundError(c) => {
                write!(f, "No character found in text: {}", c)
            }
            AppError::ColorNotFoundError(color) => {
                write!(f, "Color for terminal not found: {:?}", color)
            }
            AppError::TimeError(e) => {
                write!(f, "Time error: {}", e)
            }
            AppError::AppDatabaseError(e) => {
                write!(f, "Database error: {}", e)
            }
            AppError::ParsingError(e) => {
                write!(f, "Parsing error: {}", e)
            }
            AppError::AppHistoryError(e) => {
                write!(f, "History error: {}", e)
            }
            AppError::TwitterError { url, error_description } => {
                write!(f, "Can't tweet result: {}\n{}", url, error_description)
            }
        }
    }
}

impl From<history::HistoryError> for AppError {
    fn from(value: history::HistoryError) -> Self {
        AppError::AppHistoryError(value)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(value: std::num::ParseIntError) -> Self {
        AppError::ParsingError(value)
    }
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        AppError::AppDatabaseError(value)
    }
}

impl From<SystemTimeError> for AppError {
    fn from(value: SystemTimeError) -> Self {
        AppError::TimeError(value)
    }
}


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
