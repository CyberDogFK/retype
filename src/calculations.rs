use std::cmp::min;
use std::time::SystemTime;
use crate::timer;

/// Return index at which there is a change in strings.
/// 
/// This is used to determine the index up to which text must be dimmed and
/// after which must be colored red (indicating mismatch).
pub fn first_index_at_which_strings_differ(string1: &str, string2: &str) -> usize {
    let length = min(string1.len(), string2.len());
    // todo: maybe we can use this to optimize the loop below
    // let string1_chars = string1.chars();
    // let string2_chars = string2.chars();

    for index in 0..length  {
        if string1.chars().nth(index) != string2.chars().nth(index) {
            return index;
        }
    }
    length
}

/// Count the number of lines required for displaying text.
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
    // format!("{:.2}", wpm)
    text.len() as f64 / time_taken
}

pub fn accuracy(total_chars_typed: usize, wrongly_typed: usize) -> f64 {
    ((total_chars_typed - wrongly_typed) as f64 / total_chars_typed as f64) * 100.0
}

// Since index is copy value, we can modify it without affecting the original value
pub fn get_space_count_after_ith_word(mut index: usize, text: &str) -> usize {
    let mut count = 0;
    // todo: do something with this unwrap()
    while index < text.len() && text.chars().nth(index).unwrap() == ' ' {
        index += 1;
        count += 1;
    }
    count
}

/// Wrap text on the screen according to the window width.
///
/// Returns text with extra spaces which makes the string word wrap.
pub fn word_wrap(text: &str, width: i32) -> String {
    // For the end of each line, move backwards until you find a space.
    // When you do, append those many spaces after the single space.
    let mut text = text.to_string();
    for line in (1..=number_of_lines_to_fit_text_in_window(&text, width) + 1) {
        // Current line fits in the window
        if line * width >= text.len() as i32 {
            continue;
        }

        // Last cell of that line
        let mut index: usize = (line * width - 1) as usize;

        // Continue if already a space
        if text.chars().nth(index).unwrap() == ' ' {
            continue;
        }

        index = text[0..index].rfind(' ').unwrap();

        let space_count = line * width - index as i32;
        let space_string = " ".repeat(space_count as usize);

        let first = text[0..index].to_string();
        let third = text[index + 1..text.len()].to_string();
        text = format!("{}{}{}", first, space_string, third);
    }
    text
}
