use std::fmt::Formatter;
use crate::PreparedText;
use rand::Rng;

#[derive(Debug)]
pub enum DatabaseError {
    SqliteError(sqlite::Error),
    OutOfRangeError(u32),
    DifficultyOutOfRangeError(u32),
}

impl From<sqlite::Error> for DatabaseError {
    fn from(error: sqlite::Error) -> Self {
        DatabaseError::SqliteError(error)
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::SqliteError(e) => {
                write!(f, "Sqlite error: {}", e)
            }
            DatabaseError::OutOfRangeError(n) => {
                write!(f, "ID out of range: {}, select in range [1,6000]", n)
            }
            DatabaseError::DifficultyOutOfRangeError(n) => {
                write!(f, "Difficulty out of range: {}, select in range [1,5]", n)
            }
        }
    }
}

/// Load given text from database with given id.
/// # Arguments
/// * `text_id` - ID of text to load
/// # Returns
/// * `Result<FileText>` containing file contents or error message
pub fn load_text_from_database(text_id: u32, database_path: &str) -> Result<PreparedText, DatabaseError> {
    let row_count = 6000;
    if 1 <= text_id && text_id <= row_count {
        let text = fetch_text_with_id(text_id, database_path)?;
        Ok((text, text_id.to_string()))
    } else {
        Err(DatabaseError::OutOfRangeError(text_id))
    }
}

pub fn load_text_from_database_with_random_difficulty(
    database_path: &str,
) -> Result<PreparedText, DatabaseError> {
    let random = rand::thread_rng().gen_range(1..6);
    load_text_from_database_based_on_difficulty(random, database_path)
}

/// Load text of given difficulty from database if parameter is passed.
/// # Arguments::
/// * `difficulty` - Difficulty level of text to load
/// # Returns:
/// * `Result<FileText>` - Text and ID of text
pub fn load_text_from_database_based_on_difficulty(
    difficulty: u32,
    database_path: &str,
) -> Result<PreparedText, DatabaseError> {
    let max_level = 5;

    if 1 <= difficulty && difficulty <= max_level {
        // Each difficulty section has 6000/5 = 1200 texts each
        let upper_limit = difficulty * 1200;
        let lower_limit = upper_limit - 1200 + 1;

        let text_id = rand::thread_rng().gen_range(lower_limit..upper_limit + 1);
        let text = fetch_text_with_id(text_id, database_path)?;
        Ok((text, text_id.to_string()))
    } else {
        Err(DatabaseError::DifficultyOutOfRangeError(difficulty))
    }
}

/// Fetch row from data.db database.
/// # Arguments
/// * `serial_id` - The unique ID of database entry.
/// # Returns
/// * `Result<String>` - The text corresponding to the ID.
pub fn fetch_text_with_id(serial_id: u32, database_path: &str) -> Result<String, sqlite::Error> {
    let conn = sqlite::open(database_path)?;

    let query = "SELECT txt FROM data WHERE id = ?";

    let mut statement = conn.prepare(query)?;
    statement.bind((1, serial_id as i64))?;
    statement.next()?;
    let txt = statement.read("txt")?;
    Ok(txt)
}
