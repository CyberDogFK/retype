use std::fmt::{write, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use chrono::Datelike;

pub enum HistoryError {
    CsvError(csv::Error),
    IoError(std::io::Error),
    HomeDirError(String),
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::IoError(e) => {
                write!(f, "An IO error occurred: {}", e)
            }
            HistoryError::HomeDirError(s) => {
                write!(f, "Unable to get home directory: {}", s)
            }
            HistoryError::CsvError(e) => {
                write!(f, "An error occurred while reading or writing CSV: {}", e)
            }
        }
    }
}

impl std::fmt::Debug for HistoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<csv::Error> for HistoryError {
    fn from(e: csv::Error) -> Self {
        HistoryError::CsvError(e)
    }
}

impl From<std::io::Error> for HistoryError {
    fn from(e: std::io::Error) -> Self {
        HistoryError::IoError(e)
    }
}

pub fn get_history_records(number_of_records: i32) -> Result<Vec<String>, HistoryError> {
    todo!()
}

pub fn show_history(number_of_records: i32) -> Result<(), HistoryError> {
    todo!()
}

/// Save test stats to a history file
pub fn save_history(text_id: &str, current_speed_wpm: f64, accuracy: f64) -> Result<(), HistoryError> {
    let history_file_path = history_file_absolute_path()?;
    
    let file_exist = history_file_path.exists();

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(history_file_path)?;
    
    let mut writer = csv::Writer::from_writer(file);
    if !file_exist {
        writer.write_record(["ID", "WPM", "DATE", "TIME", "ACCURACY"])?;
    }
    let current_time = chrono::Local::now();
    let format_time = current_time.format("%H:%M:%S").to_string();
    
    let test_data = [
        text_id,
        &format!("{:.2}", current_speed_wpm),
        &current_time.day().to_string(),
        &format_time,
        &format!("{:.2}", accuracy),
    ];
    writer.write_record(test_data)?;
    writer.flush()?;
    Ok(())
}

fn history_file_absolute_path() -> Result<PathBuf, HistoryError> {
    let history_filename = ".rstype_history.csv";
    Ok(
        home::home_dir()
            .take_if(|p| !p.as_os_str().is_empty())
            .ok_or(HistoryError::HomeDirError(history_filename.to_string()))?
            .join(history_filename)
    )
}
