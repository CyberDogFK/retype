use std::collections::HashMap;
use std::io::Write;
use std::time::SystemTime;
use pancurses::{ColorPair, Input};
use crate::calculations::{accuracy, first_index_at_which_strings_differ, number_of_lines_to_fit_text_in_window, speed_in_wpm};
use crate::{history, timer, PreparedText};
use crate::database::load_text_from_database;

#[derive(PartialEq, Eq, Hash)]
enum Color {
    Green,
    Red,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Black,
}

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
    key_strokes: Vec<(usize, usize)>,
    mistyped_keys: Vec<usize>,

    // Time at which test started
    start_time: SystemTime,
    // Time at which test ended
    end_time: SystemTime,

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
    color: HashMap<Color, ColorPair>,

    stdout: std::io::Stdout,
}

impl App {
    pub fn from_prepared_text(prepared_text: PreparedText) -> Self {
        let (text, text_id) = prepared_text;
        let tokens: Vec<String> = text.split_ascii_whitespace()
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
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
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
            color: HashMap::new(),
            stdout: std::io::stdout(),
        }
    }

    pub fn main(&mut self, win: &pancurses::Window) {
        self.initialize_windows(win);

        loop {
            // let key = keyinput(win);
            let key = win.getch().unwrap_or(Input::Character(' '));

            if !self.first_key_pressed {
                if key == Input::Character(27 as char) {
                    pancurses::endwin();
                    std::process::exit(0);
                }
                if key == Input::KeyLeft {
                    self.switch_text(win, -1)
                }
                if key == Input::KeyRight {
                    self.switch_text(win, 1)
                }
            }
            
            // Test mode
            if self.mode == 0 {
                self.typing_mode(win, key)
            }

            if let Input::Character(c) = key {
                self.key = c.to_string();
            } else {
                self.key = "".to_string();
            }
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

        self.color = {
            let mut color = HashMap::new();
            color.insert(Color::Green, ColorPair(1));
            color.insert(Color::Red, ColorPair(2));
            color.insert(Color::Blue, ColorPair(3));
            color.insert(Color::Yellow, ColorPair(4));
            color.insert(Color::Cyan, ColorPair(5));
            color.insert(Color::Magenta, ColorPair(6));
            color.insert(Color::Black, ColorPair(7));
            color
        };

        // This sets input to be a non-blocking call and will block for 100ms
        // Returns -1 if no input found at the end of time
        win.nodelay(true);
        win.timeout(100);

        self.setup_print(win);
    }

    /// Print setup text at beginning of each typing sessions.
    fn setup_print(&mut self, win: &pancurses::Window) {
        win.attrset(*self.color.get(&Color::Cyan).unwrap());
        win.mvaddstr(0, 0, format!("ID:{} ", self.text_id));
        win.attrset(*self.color.get(&Color::Blue).unwrap());
        win.mvaddstr(0, self.window_width / 2 - 5, " MITYPE");

        // Text is printed BOLD initially
        // It is dimmed as user types on top of it
        win.attrset(pancurses::A_BOLD);
        win.mvaddstr(2, 0, &self.text);

        self.print_realtime_wpm(win);

        win.mv(2, 0);
        win.refresh();
    }

    fn print_realtime_wpm(&mut self, win: &pancurses::Window) {
        let mut current_wpm = 0.0;
        let total_time = timer::get_elapsed_minutes_since_first_keypress(self.start_time);
        if total_time != 0.0 {
            let words = self.current_string.split_ascii_whitespace();
            let word_count = words.count() as f64;
            current_wpm = word_count / total_time;
        }
        win.attrset(*self.color.get(&Color::Cyan).unwrap());
        win.mvaddstr(0, self.window_width - 14, format!("{:.2}", current_wpm));
        win.addstr(" WPM ");
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

    /// Load next of previous text snippet from database.
    fn switch_text(&mut self, win: &pancurses::Window, direction: i32) {
        win.clear();

        let text_id = self.text_id.parse::<i32>().unwrap() + direction;
        self.text_id = text_id.to_string();
        self.text = load_text_from_database(text_id as u32, "data.db").unwrap().0;
        self.tokens = self.text.split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect();
        self.text = self.tokens.join(" ");
        self.text_backup = self.text.clone();

        self.text = word_wrap(&self.text, self.window_width);

        self.reset_test();
        self.setup_print(win);
        self.update_state(win);
    }

    /// Report on typing session results
    fn update_state(&mut self, win: &pancurses::Window) {
        self.clear_line(win, self.number_of_lines_to_print_text);
        self.clear_line(win, self.number_of_lines_to_print_text + 2);
        self.clear_line(win, self.number_of_lines_to_print_text + 4);

        // Highlight in RED if word reaches the word limit length
        if self.current_word.len() >= self.current_word_limit {
            win.attrset(*self.color.get(&Color::Red).unwrap());
            win.mvaddstr(self.number_of_lines_to_print_text, 0, &self.current_word);
        } else {
            win.attroff(*self.color.get(&Color::Red).unwrap());
            win.mvaddstr(self.number_of_lines_to_print_text, 0, &self.current_word);
        }

        // Text is printed BOLD initially
        // It is dimmed as user types on top of it
        win.attrset(pancurses::A_BOLD);
        win.mvaddstr(2, 0, &self.text);
        win.attrset(pancurses::A_DIM);
        win.mvaddstr(2, 0, &self.text[0..=self.current_string.len()]);

        let index = first_index_at_which_strings_differ(&self.current_string, &self.text);
        // Check if difference was found
        if index < self.current_string.len() && self.current_string.len() <= self.text.len() {
            self.mistyped_keys.push(self.current_string.len() - 1)
        }

        win.attrset(*self.color.get(&Color::Red).unwrap());
        win.mvaddstr(
            (2 + index as i32 / self.window_width),
            index as i32 % self.window_width,
            &self.text[index..=index]
        );

        // End of test, all characters are typed out
        if index == self.text.len() {
            self.test_end(win);
        }

        win.refresh();
    }

    /// Trigger at the end of the test
    ///
    /// Display options for the user to choose at the end of the test.
    /// Display stats.
    fn test_end(&mut self, win: &pancurses::Window) {
        for i in self.mistyped_keys.iter() {
            win.attrset(*self.color.get(&Color::Red).unwrap());
            win.mvaddstr(
                (2 + *i as i32 / self.window_width),
                *i as i32 % self.window_width,
                &self.text[*i..=*i]
            );
        }

        pancurses::curs_set(0);

        // Calculate stats at the end of the test
        if self.mode == 0 {
            self.current_speed_wpm = speed_in_wpm(&self.tokens, self.start_time);
            let total_chars_in_text = self.text_backup.len();
            let wrongly_typed_chars = self.total_chars_typed - total_chars_in_text;
            self.accuracy = accuracy(self.total_chars_typed, wrongly_typed_chars);
            self.time_taken = timer::get_elapsed_minutes_since_first_keypress(self.start_time) as u64;

            self.mode = 1;
            // Find time difference between the key strokes
            // The key_strokes list is storing the time at which the key is pressed
            for index in (0..(self.key_strokes.len() - 1)).rev() {
                self.key_strokes[index].0 -= self.key_strokes[index - 1].0
            }
            self.key_strokes[0].0 = 0;
        }


        win.mvaddstr(self.number_of_lines_to_print_text, 0, " Your typing speed is ");
        win.attrset(*self.color.get(&Color::Magenta).unwrap());
        win.addstr(format!(" {} ", self.current_speed_wpm));
        win.attroff(*self.color.get(&Color::Magenta).unwrap());
        win.addstr(" WPM ");

        win.attrset(*self.color.get(&Color::Black).unwrap());
        win.mvaddstr(self.number_of_lines_to_print_text + 2, 1, " Enter ");
        win.attroff(*self.color.get(&Color::Black).unwrap());
        win.addstr(" to see replay, ");

        win.attrset(*self.color.get(&Color::Black).unwrap());
        win.mvaddstr(self.number_of_lines_to_print_text + 3, 1, " Arrow keys");
        win.attroff(*self.color.get(&Color::Black).unwrap());
        win.addstr(" to change text ");

        win.attrset(*self.color.get(&Color::Black).unwrap());
        win.mvaddstr(self.number_of_lines_to_print_text + 4, 1, " CTRL+T ");
        win.attroff(*self.color.get(&Color::Black).unwrap());
        win.addstr(" to tweet result.");

        self.print_stats(win);

        self.first_key_pressed = false;
        self.end_time = SystemTime::now();
        self.current_string = "".to_string();
        self.current_word = "".to_string();
        self.token_index = 0;
        
        self.start_time = SystemTime::now();
        if !self.test_complete {
            win.refresh();
            history::save_history(&self.text_id, self.current_speed_wpm, &format!("{:.2}", self.accuracy));
            self.test_complete = true;
        }
    }

    /// Print the bottom stats bar after each run.
    fn print_stats(&mut self, win: &pancurses::Window) {
        win.attrset(*self.color.get(&Color::Magenta).unwrap());
        win.mvaddstr(self.window_height - 1, 0,
                     &format!(" WPM: {:.2} ", self.current_speed_wpm));

        win.attrset(*self.color.get(&Color::Green).unwrap());
        win.addstr(&format!(" Time: {:.2} ", self.time_taken * 60));

        win.attrset(*self.color.get(&Color::Cyan).unwrap());
        win.addstr(&format!(" Accuracy: {:.2} ", self.accuracy));
    }

    /// Clear a line on the window
    fn clear_line(&self, win: &pancurses::Window, line: i32) {
        win.mv(line, 0);
        win.clrtoeol();
    }

    /// Reset the data for current typing session.
    fn reset_test(&mut self) {
        self.mode = 0;
        self.current_word = "".to_string();
        self.current_string = "".to_string();
        self.first_key_pressed = false;
        self.key_strokes = vec![];
        self.mistyped_keys = vec![];
        self.start_time = 0.0;
        self.token_index = 0;
        self.current_speed_wpm = 0.0;
        self.total_chars_typed = 0;
        self.accuracy = 0.0;
        self.time_taken = 0;
        self.test_complete = false;
        pancurses::curs_set(1);
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

/// Retrieve next character of text input
/// # Returns
/// * `String` containing the next character of text input
fn keyinput(win: &pancurses::Window) -> String {
    match win.getch() {
        Some(Input::Character(key)) => {
            "".to_string()
        }
        None => "".to_string(),
        _ => "".to_string(),
    }
}
