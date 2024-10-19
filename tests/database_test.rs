use rstype::database;

#[test]
fn test_fetching_text_from_db() {
    // let database_path = format!("tests/{}.db", Uuid::new_v4()); // -- change to this,
    // if there will be more than one test of db
    let database_path = "tests/data.db";
    let connection = sqlite::open(database_path).unwrap();
    connection.execute(
        "CREATE TABLE data (id INTEGER PRIMARY KEY, txt TEXT);",
    ).unwrap();
    let value = "Hello, world!";
    connection.execute(
        format!("INSERT INTO data (txt) VALUES ('{}');", value),
    ).unwrap();
    let serial_id = 1;
    let result = database::fetch_text_with_id(serial_id, database_path).unwrap();
    assert_eq!(result, value);
    std::fs::remove_file(database_path).unwrap()
}
