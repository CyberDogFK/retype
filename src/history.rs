use chrono::Datelike;
use std::fmt::Formatter;
use std::fs::OpenOptions;
use std::path::PathBuf;
use csv::StringRecord;

#[derive(Debug)]
pub enum HistoryError {
    CsvError(csv::Error),
    IoError(std::io::Error),
    HomeDirError(String),
    FileDoesNotExist,
    FileIsEmpty,
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
            HistoryError::FileDoesNotExist => {
                write!(f, "The history file does not exist")
            }
            HistoryError::FileIsEmpty => {
                write!(f, "The history file is empty")
            }
        }
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

pub enum NumberOfRecords {
    All,
    Last(usize),
}

impl From<usize> for NumberOfRecords {
    fn from(value: usize) -> Self {
        NumberOfRecords::Last(value)
    }
}

/// Get records from history
///
/// Defaults to -1 if argumetns value not provided on command line
/// # Arguments:
/// * `number_of_records` - Number of last records to print
/// # Returns:
/// * `Vec<String>` - The len of this list is `number_of_records` or all records
pub fn get_history_records(number_of_records: NumberOfRecords) -> Result<Vec<StringRecord>, HistoryError> {
    let history_file_path = history_file_absolute_path()?;

    if !history_file_path.exists() {
        return Err(HistoryError::FileDoesNotExist);
    }

    let mut reader = csv::Reader::from_path(history_file_path)?;
    if !reader.has_headers() {
        return Err(HistoryError::FileIsEmpty);
    }

    let mut records: Vec<StringRecord> = vec![];
    for record in reader.records() {
        let record = record?;
        records.push(record);
    }

    let total_records = records.len();

    let number_of_records = match number_of_records {
        NumberOfRecords::All => {
            total_records
        }
        NumberOfRecords::Last(n) => if n >= total_records {
            total_records
        } else { n },
    };

    let start_count = if number_of_records < total_records {
        total_records - number_of_records
    } else { 0 };

    Ok(records[start_count..total_records].to_vec())
}

pub fn show_history(number_of_records: NumberOfRecords) -> Result<(), HistoryError> {
    let records = get_history_records(number_of_records)?;

    if records.is_empty() {
        println!("0 records found");
    }

    println!("Last {} records:", records.len());
    println!("ID\tWPM\tDATE\t\tTIME\t\tACCURACY");
    for record in records {
        let formatter_row_data = record.iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join("\t");
        println!("{}%", formatter_row_data);
    }
    Ok(())
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
    let format_date = current_time.format("%Y-%m-%d").to_string();
    let format_time = current_time.format("%H:%M:%S").to_string();

    let test_data = [
        text_id,
        &format!("{:.2}", current_speed_wpm),
        &format_date,
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
