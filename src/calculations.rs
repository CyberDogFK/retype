use std::cmp::min;
use std::time::SystemTime;
use crate::timer;

/// Return index at which there is a change in strings.
/// 
/// This is used to determine the index up to which text must be dimmed and
/// after which must be colored red (indicating mismatch).
pub fn first_index_at_which_strings_differ(string1: &str, string2: &str) -> usize {
    let length = min(string1.len(), string2.len());

    for index in 0..length  {
        if string1.chars().nth(index) != string2.chars().nth(index) {
            return index;
        }
    }
    length
}

/// Count number of lines required for displaying text.
pub fn number_of_lines_to_fit_text_in_window(string: &str, window_width: i32) -> i32 {
    let n = string.len() as f64 / window_width as f64;
    f64::ceil(n) as i32
}

/// Calculate speed in words per minute.
/// # Arguments:
/// * `text` - Text to calculate speed for
/// * `start_time` - Time at which typing started the sample text.
/// # Returns:
/// * `f64` Speed in words per minute
pub fn speed_in_wpm(text: &Vec<String>, start_time: SystemTime) -> f64 {
    let time_taken = timer::get_elapsed_minutes_since_first_keypress(start_time);
    let wpm = text.len() as f64 / time_taken;
    
    // format!("{:.2}", wpm)
    wpm
}

pub fn accuracy(total_chars_typed: usize, wrongly_typed: usize) -> f64 {
    ((total_chars_typed - wrongly_typed) as f64 / total_chars_typed as f64) * 100.0
}

pub fn get_space_count_after_ith_word(index: usize, text: &String) -> usize {
    let mut count = 0;
    let mut index = index;
    while index < text.len() && text.chars().nth(index).unwrap() == ' ' {
        index += 1;
        count += 1;
    }
    count
}