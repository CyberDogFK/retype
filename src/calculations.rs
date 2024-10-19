/// Count number of lines required for displaying text.
pub fn number_of_lines_to_fit_text_in_window(string: &str, window_width: i32) -> i32 {
    let n = string.len() as f64 / window_width as f64;
    f64::ceil(n) as i32
}