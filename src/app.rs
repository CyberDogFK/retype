use std::io::Write;
use crate::calculations::number_of_lines_to_fit_text_in_window;
use crate::PreparedText;

// pub enum Color {
//     Red,
//     Black,
//     Magenta,
//     Cyan,
// }

pub struct App {
    text: String,
    text_id: String,

    tokens: Vec<String>,
    text_backup: String,

    // Current typed word and entire string
    current_word: String,
    current_string: String,

    key: String,
    // First valid key press
    first_key_pressed: bool,
    // Stores keypress, time tuple
    key_strokes: Vec<String>,
    mistyped_keys: Vec<String>,

    // Time at which test started
    start_time: u64,
    // Time at which test ended
    end_time: u64,

    // Keep track of the token index in text
    token_index: usize,
    // mode = 0 when in test
    // mode = 1 when in replay
    mode: u8,

    window_height: i32,
    window_width: i32,

    number_of_lines_to_print_text: i32,

    // Restrict current word length to a limit
    // Used to highlight one the limit is reached
    // limit is set to the length of largest word in string + 5 for buffer
    current_word_limit: usize,

    test_complete: bool,

    // Real-time speed, the value at the end of the test is the result
    // And a few other stats
    current_speed_wpm: f64,
    accuracy: f64,
    time_taken: u64,

    total_chars_typed: usize,

    // Color mapping
    color: Color,

    stdout: std::io::Stdout,
}

impl App {
    pub fn from_prepared_text(prepared_text: PreparedText) -> Self {
        let (text, text_id) = prepared_text;
        let tokens: Vec<String> = text.split("")
            .map(|s| s.to_string())
            .collect();

        let text = tokens.join(" ");
        let text_backup = text.clone();
        let current_word_limit = tokens.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0) + 5;

        Self {
            text,
            text_id,
            tokens,
            text_backup,
            current_word: "".to_string(),
            current_string: "".to_string(),
            key: "".to_string(),
            first_key_pressed: false,
            key_strokes: vec![],
            mistyped_keys: vec![],
            start_time: 0,
            end_time: 0,
            token_index: 0,
            mode: 0,
            window_height: 0,
            window_width: 0,
            number_of_lines_to_print_text: 0,
            current_word_limit,
            test_complete: false,
            current_speed_wpm: 0.0,
            accuracy: 0.0,
            time_taken: 0,
            total_chars_typed: 0,
            color: Color::Red,
            stdout: std::io::stdout(),
        }
    }

    pub fn main(&mut self, win: &pancurses::Window) {
        self.initialize_windows(win);

        while true {

        }
    }

    /// Configure the initial state of the curses interface
    ///
    /// # Arguments
    /// * `win` - The curses window
    pub fn initialize_windows(&mut self, win: &pancurses::Window) {
        {
            let (window_height, window_width) = get_dimensions(win);
            self.window_height = window_height;
            self.window_width = window_width;
        }
        // This works by adding extra spaces to the text where needed
        self.text = word_wrap(&self.text, self.window_width);
        
        // Check if we can fit text in current window after adding word wrap
        self.screen_size_check();
        
        pancurses::init_pair(1, pancurses::COLOR_WHITE, pancurses::COLOR_GREEN);
        pancurses::init_pair(2, pancurses::COLOR_WHITE, pancurses::COLOR_RED);
        pancurses::init_pair(3, pancurses::COLOR_WHITE, pancurses::COLOR_BLUE);
        pancurses::init_pair(4, pancurses::COLOR_WHITE, pancurses::COLOR_YELLOW);
        pancurses::init_pair(5, pancurses::COLOR_WHITE, pancurses::COLOR_CYAN);
        pancurses::init_pair(6, pancurses::COLOR_WHITE, pancurses::COLOR_MAGENTA);
        pancurses::init_pair(7, pancurses::COLOR_BLACK, pancurses::COLOR_WHITE);
        
        pancurses::color
        enum Color {
            Green()
        }
    }

    /// Check if screen size is enough to print text.
    fn screen_size_check(&mut self) {
        self.number_of_lines_to_print_text = 
            number_of_lines_to_fit_text_in_window(&self.text, self.window_width) + 3;
        if self.number_of_lines_to_print_text + 7 >= self.window_height {
            pancurses::endwin();
            std::io::stdout().write_all(b"Window too small to print given text").unwrap();
            // writeln!(stdout(), "Window too small to print given text").unwrap();
            std::process::exit(1);
        }
    }
}

/// Wrap text on the screen according to the window width.
///
/// Returns text with extra spaces which makes the string word wrap.
fn word_wrap(text: &str, width: i32) -> String {
    // For the end of each line, move backwards until you find a space.
    // When you do, append those many spaces after the single space.
    let mut text = text.to_string();
    for line in (1..=number_of_lines_to_fit_text_in_window(&text, width) + 1) {
        // Current line fits in the window
        if line * width >= text.len() as i32 {
            continue
        }

        // Last cell of that line
        let mut index: usize = (line * width - 1) as usize;

        // Continue if already a space
        if text.chars().nth(index).unwrap() == ' ' {
            continue
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

/// Get the height and width of terminal
///
/// # Arguments
/// * `win` - The curses window
/// # Returns
/// * `(i32, i32)` containing the height and width of the terminal
fn get_dimensions(win: &pancurses::Window) -> (i32, i32) {
    win.get_max_yx()
}
