use uuid::Uuid;
use rstype::database;

#[test]
fn test_fetching_text_from_db_based_on_difficulty() {
    let (connection, database_path) = prepare_connection_with_table();
    
    let value = "Hello, world!";
    
    for _ in 0..6000 {
        connection.execute(
            format!("INSERT INTO data (txt) VALUES ('{}');", value),
        ).unwrap();
    }
    
    let difficulty = 1;
    let result = database::load_text_from_database_based_on_difficulty(difficulty, &database_path).unwrap();
    assert_eq!(result.0, value);
    let id = result.1.parse::<u32>().unwrap();
    assert!((1..=1200).contains(&id));
    std::fs::remove_file(&database_path).unwrap()
}

#[test]
fn test_fetching_text_from_db() {
    let (connection, database_path) = prepare_connection_with_table();
    let value = "Hello, world!";
    
    connection.execute(
        format!("INSERT INTO data (txt) VALUES ('{}');", value),
    ).unwrap();
    
    let serial_id = 1;
    let result = database::fetch_text_with_id(serial_id, &database_path).unwrap();
    assert_eq!(result, value);
    std::fs::remove_file(database_path).unwrap() 
}

fn prepare_connection_with_table() -> (sqlite::Connection, String) {
    let database_path = format!("tests/{}.db", Uuid::new_v4());
    let connection = sqlite::open(&database_path).unwrap();
    connection.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, txt TEXT);",
    ).unwrap();
    (connection, database_path)
}
