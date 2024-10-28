#[test]
fn load_text_from_file() {
    let file_address = "tests/test.txt";
    let content = "Hello, world!";
    std::fs::write(file_address, content).unwrap();
    let result = rstype::load_text_from_file(file_address).unwrap();
    assert_eq!(result.0, content);
    assert_eq!(result.1, file_address);
    std::fs::remove_file(file_address).unwrap()
}
