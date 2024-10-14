/// Fetch row from data.db database.
/// # Arguments
/// * `serial_id` - The unique ID of database entry.
/// # Returns
/// * `Result<String>` - The text corresponding to the ID.
pub fn fetch_text_from_id(serial_id: u32) -> Result<String, sqlite::Error> {
    let conn = sqlite::open("data.db")?;
        // .map_err(|e| format!("Error opening database: {}", e))?;

    let query = "SELECT txt FROM data WHERE id = ?";

    let mut statement = conn.prepare(query)?;
        // .map_err(|e| format!("Error preparing query: {}", e))?;
    statement.bind((1, serial_id as i64))?;
        // .map_err(|e| format!("Error binding parameter: {}", e))?;
    statement.next()?;
        // .map_err(|e| format!("Error executing query: {}", e))?;
    let txt = statement.read("txt")?;
        // .map_err(|e| format!("Error reading result: {}", e))?;
    Ok(txt)
}