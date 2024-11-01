use crate::calculations::{
    accuracy, first_index_at_which_strings_differ,
    get_space_count_after_ith_word, number_of_lines_to_fit_text_in_window,
    speed_in_wpm, word_wrap
};
use crate::database::load_text_from_database;
use crate::keycheck::{
    get_key_mapping, is_backspace, is_ctrl_backspace, is_ctrl_c, is_ctrl_t, is_enter, is_escape,
    is_resize, is_tab, is_valid_initial_key,
};
use crate::{exit, history, timer, AppError, AppResult, PreparedText};
use pancurses::{ColorPair, Input};
use std::collections::HashMap;
use std::ops::Add;
use std::time;
use std::time::{Duration, SystemTime};


#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Color {
    Green,
    Red,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Black,
}

impl Color {
    fn not_found_err(self) -> AppError {
        AppError::ColorNotFoundError(self)
    }
}

pub struct App {
    text: String,
    text_id: String,

    tokens: Vec<String>,
    text_backup: String,

    // Current typed word and entire string
    current_word: String,
    current_string: String,

    // First valid key press
    first_key_pressed: bool,
    // Stores keypress, time tuple
    key_strokes: Vec<(f64, Input)>,
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
    time_taken: f64,

    total_chars_typed: usize,

    // Color mapping
    color: HashMap<Color, ColorPair>,
}

impl App {
    pub fn from_prepared_text(prepared_text: PreparedText) -> Self {
        let (text, text_id) = prepared_text;
        let tokens: Vec<String> = text
            .split_ascii_whitespace()
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
            time_taken: 0.0,
            total_chars_typed: 0,
            color: HashMap::new(),
        }
    }

    pub fn run(&mut self, win: &pancurses::Window) -> AppResult<()> {
        self.initialize_windows(win)?;
        win.nodelay(false);
        win.keypad(true);

        loop {
            let key = win.getch();

            if let Some(key) = key {
                if !self.first_key_pressed {
                    match key {
                        Input::Character('\u{1b}') => {
                            exit(0)
                        }
                        Input::KeyLeft => self.switch_text(win, -1)?,
                        Input::KeyRight => self.switch_text(win, 1)?,
                        _ => {}
                    }
                }

                // Test mode
                if self.mode == 0 {
                    self.typing_mode(win, &key)?;
                } else {
                    // Again mode
                    // Tab to retry last test
                    if is_tab(&key) {
                        win.clear();
                        self.reset_test();
                        self.setup_print(win)?;
                        self.update_state(win)?;
                    }

                    // Replay
                    if is_enter(&key) {
                        self.replay(win)?;
                    }

                    // Tweet result
                    if is_ctrl_t(&key) {
                        self.share_result()?;
                    }
                }
            }

            win.refresh();
        }
    }

    /// Configure the initial state of the curses interface
    ///
    /// # Arguments
    /// * `win` - The curses window
    pub fn initialize_windows(&mut self, win: &pancurses::Window) -> AppResult<()> {
        {
            let (window_height, window_width) = get_dimensions(win);
            self.window_height = window_height;
            self.window_width = window_width;
        }
        // This works by adding extra spaces to the text where needed
        self.text = word_wrap(&self.text, self.window_width)?;

        // Check if we can fit text in the current window after adding word wrap
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

        self.setup_print(win)
    }

    /// Start recording typing session progress
    fn typing_mode(&mut self, win: &pancurses::Window, key: &Input) -> AppResult<()> {
        // Note start time when the first valid key is pressed
        if !self.first_key_pressed && is_valid_initial_key(key) {
            self.start_time = SystemTime::now();
            self.first_key_pressed = true;
        }

        if is_resize(key) {
            self.resize(win)?;
        }

        if !self.first_key_pressed {
            return Ok(());
        }

        self.key_strokes.push((
            SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs_f64(),
            *key,
        ));

        self.print_realtime_wpm(win)?;

        self.key_printer(win, key)
    }

    /// Print required key to terminal
    fn key_printer(&mut self, win: &pancurses::Window, key: &Input) -> AppResult<()> {
        // reset test
        if is_escape(key) {
            self.reset_test()
        } else if is_ctrl_c(key) {
            exit(0)
        } else if is_resize(key) {
            self.resize(win)?;
        } else if is_backspace(key) {
            self.erase_key();
        } else if is_ctrl_backspace(key) {
            self.erase_word()?;
        }
        // Ignore spaces at the start of the word (Plover support)
        else if key == &Input::Character(' ') && self.current_word.len() < self.current_word_limit
        {
            self.total_chars_typed += 1;
            if !self.current_word.is_empty() {
                self.check_word()?;
            }
        } else if is_valid_initial_key(key) {
            let key = get_key_mapping(key);
            self.appendkey(&key);
            self.total_chars_typed += 1;
        }
        self.update_state(win)
    }

    fn appendkey(&mut self, key: &String) {
        if self.current_word.len() < self.current_word_limit {
            self.current_word += key;
            self.current_string += key;
        }
    }

    /// Accept finalized word
    fn check_word(&mut self) -> AppResult<()> {
        let spc = get_space_count_after_ith_word(self.current_string.len(), &self.text)?;
        if self.current_word == self.tokens[self.token_index] {
            self.token_index += 1;
            self.current_word = "".to_string();
            self.current_string += " ".repeat(spc).as_str();
        } else {
            self.current_word = format!("{} ", self.current_word);
            self.current_string = format!("{} ", self.current_string);
        }
        Ok(())
    }

    /// Open twitter intent on a browser.
    fn share_result(&mut self) -> AppResult<()> {
        let message =
            format!("My typing speed is {:.2} WPM!\n\
            Know yours on rstype.\n\
            \"https://github.com/CyberDogFK/rstype\" by @CyberDogFK\n\
            #TypingTest #Rust", self.current_speed_wpm);
        let url = format!("https://twitter.com/intent/tweet?text={}", message);
        open::that(&url).map_err(|e| {
            AppError::TwitterError {
                url,
                error_description: e.to_string(),
            }
        })
    }

    /// Erase the last typed word
    fn erase_word(&mut self) -> AppResult<()> {
        if !self.current_word.is_empty() {
            let index_word = self.current_word.rfind(" ")
                .ok_or(AppError::NoCharFoundError(' '))?;
            if index_word as i32 == -1 {
                // Single word
                let word_length = self.current_word.len();
                self.current_string =
                    self.current_string[0..self.current_string.len() - word_length].to_string();
                self.current_word = "".to_string();
            } else {
                let diff = self.current_word.len() - index_word;
                self.current_word =
                    self.current_word[0..self.current_word.len() - diff].to_string();
                self.current_string =
                    self.current_string[0..self.current_string.len() - diff].to_string();
            }
        }
        Ok(())
    }

    /// Erase the last typed character
    fn erase_key(&mut self) {
        if !self.current_word.is_empty() {
            self.current_word.pop();
            self.current_string.pop();
        }
    }

    /// Response to window resize events
    fn resize(&mut self, win: &pancurses::Window) -> AppResult<()> {
        win.clear();

        let (window_height, window_width) = get_dimensions(win);
        self.window_height = window_height;
        self.window_width = window_width;
        self.text = word_wrap(&self.text_backup, self.window_width)?;

        self.screen_size_check();

        self.print_realtime_wpm(win)?;
        self.setup_print(win)?;
        self.update_state(win)?;
        Ok(())
    }

    /// Print setup text at beginning of each typing sessions.
    fn setup_print(&mut self, win: &pancurses::Window) -> AppResult<()> {
        win.attrset(*self.color.get(&Color::Cyan)
            .ok_or(Color::Cyan.not_found_err())?);
        win.mvaddstr(0, 0, format!(" ID:{} ", self.text_id));
        win.attrset(*self.color.get(&Color::Blue).
            ok_or(Color::Blue.not_found_err())?);
        win.mvaddstr(0, self.window_width / 2 - 4, " RSTYPE ");

        // Text is printed BOLD initially
        // It is dimmed as user types on top of it
        win.attrset(pancurses::A_BOLD);
        win.mvaddstr(2, 0, &self.text);

        self.print_realtime_wpm(win)?;

        win.mv(2, 0);
        win.refresh();
        Ok(())
    }

    fn print_realtime_wpm(&mut self, win: &pancurses::Window) -> AppResult<()> {
        let mut current_wpm = 0.0;
        let total_time = timer::get_elapsed_minutes_since_first_keypress(self.start_time)?;
        if total_time != 0.0 {
            let words = self.current_string.split_ascii_whitespace();
            let word_count = words.count() as f64;
            current_wpm = word_count / total_time;
        }
        win.attrset(*self.color.get(&Color::Cyan).
            ok_or(Color::Cyan.not_found_err())?);
        win.mvaddstr(0, self.window_width - 14, format!("{:.2}", current_wpm));
        win.addstr(" WPM ");
        Ok(())
    }

    /// Check if screen size is enough to print text.
    fn screen_size_check(&mut self) {
        self.number_of_lines_to_print_text =
            number_of_lines_to_fit_text_in_window(&self.text, self.window_width) + 3;
        if self.number_of_lines_to_print_text + 7 >= self.window_height {
            eprintln!("Window too small to print given text");
            exit(0)
        }
    }

    /// Play out a recordning of the user's last session
    fn replay(&mut self, win: &pancurses::Window) -> AppResult<()> {
        win.clear();
        self.print_stats(win)?;
        win.mvaddstr(self.number_of_lines_to_print_text + 2, 0, " ".repeat(self.window_width as usize));
        pancurses::curs_set(1);

        win.attrset(*self.color.get(&Color::Cyan)
            .ok_or(Color::Cyan.not_found_err())?);
        win.mvaddstr(
            0,
            self.window_width,
            format!(" {} ", self.current_speed_wpm),
        );
        win.attrset(pancurses::A_NORMAL);

        self.setup_print(win)?;

        win.timeout(10);

        let mut next_tick = SystemTime::now();
        for key in &self.key_strokes.clone() {
            next_tick = next_tick.add(Duration::from_secs_f64(key.0));
            let wait_duration = 0.0_f64.max(next_tick.duration_since(time::UNIX_EPOCH)?
                .as_secs_f64() - SystemTime::now()
                .duration_since(time::UNIX_EPOCH)?
                .as_secs_f64()
            );
            std::thread::sleep(Duration::from_secs_f64(wait_duration));

            if let Some(_key) = win.getch() {
                if is_escape(&_key) || is_ctrl_c(&_key) {
                    exit(0)
                }
            }
            self.key_printer(win, &key.1)?;
        }
        win.timeout(100);
        Ok(())
    }

    /// Report on typing session results
    fn update_state(&mut self, win: &pancurses::Window) -> AppResult<()> {
        self.clear_line(win, self.number_of_lines_to_print_text);
        self.clear_line(win, self.number_of_lines_to_print_text + 2);
        self.clear_line(win, self.number_of_lines_to_print_text + 4);

        // Highlight in RED if a word reaches the word limit length
        if self.current_word.len() >= self.current_word_limit {
            win.attrset(*self.color.get(&Color::Red)
                .ok_or(Color::Red.not_found_err())?);
            win.mvaddstr(self.number_of_lines_to_print_text, 0, &self.current_word);
        } else {
            win.mvaddstr(self.number_of_lines_to_print_text, 0, &self.current_word);
        }

        // Text is printed BOLD initially
        // It is dimmed as user types on top of it
        win.attrset(pancurses::A_BOLD);
        win.mvaddstr(2, 0, &self.text);
        win.attrset(pancurses::A_DIM);
        win.mvaddstr(2, 0, &self.text[0..self.current_string.len()]);

        let index = first_index_at_which_strings_differ(&self.current_string, &self.text);
        // Check if difference was found
        if index < self.current_string.len() && self.current_string.len() <= self.text.len() {
            self.mistyped_keys.push(self.current_string.len() - 1)
        }

        win.attrset(*self.color.get(&Color::Red)
            .ok_or(Color::Red.not_found_err())?);
        win.mvaddstr(
            2 + index as i32 / self.window_width,
            index as i32 % self.window_width,
            &self.text[index..self.current_string.len()],
        );

        // End of test, all characters are typed out
        if index == self.text.len() {
            self.test_end(win)?;
        }

        win.refresh();
        Ok(())
    }

    /// Trigger at the end of the test
    ///
    /// Display options for the user to choose at the end of the test.
    /// Display stats.
    fn test_end(&mut self, win: &pancurses::Window) -> AppResult<()> {
        for i in self.mistyped_keys.iter() {
            win.attrset(*self.color.get(&Color::Red)
                .ok_or(Color::Red.not_found_err())?);
            win.mvaddstr(
                2 + *i as i32 / self.window_width,
                *i as i32 % self.window_width,
                &self.text[*i..=*i],
            );
        }

        pancurses::curs_set(0);

        // Calculate stats at the end of the test
        if self.mode == 0 {
            self.current_speed_wpm = speed_in_wpm(&self.tokens, self.start_time)?;
            let total_chars_in_text = self.text_backup.len();
            let wrongly_typed_chars = self.total_chars_typed - total_chars_in_text;
            self.accuracy = accuracy(self.total_chars_typed, wrongly_typed_chars);
            self.time_taken = timer::get_elapsed_minutes_since_first_keypress(self.start_time)?;

            self.mode = 1;
            // Find time difference between the keystrokes
            // The key_strokes list is storing the time at which the key is pressed
            for index in (1..=(self.key_strokes.len() - 1)).rev() {
                self.key_strokes[index].0 -= self.key_strokes[index - 1].0;
            }
            self.key_strokes[0].0 = Duration::from_secs(0).as_secs_f64();
        }

        win.attrset(pancurses::A_NORMAL);
        win.mvaddstr(
            self.number_of_lines_to_print_text,
            0,
            " Your typing speed is ",
        );
        win.attrset(*self.color.get(&Color::Magenta)
            .ok_or(Color::Magenta.not_found_err())?);
        win.addstr(format!(" {:.2} ", self.current_speed_wpm));
        win.attroff(*self.color.get(&Color::Magenta)
            .ok_or(Color::Magenta.not_found_err())?);
        win.addstr(" WPM ");

        win.attrset(*self.color.get(&Color::Black)
            .ok_or(Color::Black.not_found_err())?);
        win.mvaddstr(self.number_of_lines_to_print_text + 2, 1, " Enter ");
        win.attrset(pancurses::A_NORMAL);
        win.addstr(" to see replay, ");

        win.attrset(*self.color.get(&Color::Black)
            .ok_or(Color::Black.not_found_err())?);
        win.addstr(" Tab ");
        win.attrset(pancurses::A_NORMAL);
        win.addstr(" to retry.");

        win.attrset(*self.color.get(&Color::Black)
            .ok_or(Color::Black.not_found_err())?);
        win.mvaddstr(self.number_of_lines_to_print_text + 3, 1, " Arrow keys ");
        win.attrset(pancurses::A_NORMAL);
        win.addstr(" to change text.");

        win.attrset(*self.color.get(&Color::Black)
            .ok_or(Color::Black.not_found_err())?);
        win.mvaddstr(self.number_of_lines_to_print_text + 4, 1, " CTRL+T ");
        win.attrset(pancurses::A_NORMAL);
        win.addstr(" to tweet result.");

        self.print_stats(win)?;

        self.first_key_pressed = false;
        self.end_time = SystemTime::now();
        self.current_string = "".to_string();
        self.current_word = "".to_string();
        self.token_index = 0;

        self.start_time = SystemTime::now();
        if !self.test_complete {
            win.refresh();
            history::save_history(
                &self.text_id,
                self.current_speed_wpm,
                self.accuracy,
            )?;
            self.test_complete = true;
        }
        Ok(())
    }

    /// Print the bottom stats bar after each run.
    fn print_stats(&mut self, win: &pancurses::Window) -> AppResult<()> {
        win.attrset(*self.color.get(&Color::Magenta)
            .ok_or(Color::Magenta.not_found_err())?);
        win.mvaddstr(
            self.window_height - 1,
            0,
            format!(" WPM: {:.2} ", self.current_speed_wpm),
        );

        win.attrset(*self.color.get(&Color::Green)
            .ok_or(Color::Green.not_found_err())?);
        win.addstr(format!(" Time: {:.2}s ", self.time_taken * 60.0));

        win.attrset(*self.color.get(&Color::Cyan)
            .ok_or(Color::Cyan.not_found_err())?);
        win.addstr(format!(" Accuracy: {:.2}% ", self.accuracy));
        Ok(())
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
        self.start_time = SystemTime::now();
        self.token_index = 0;
        self.current_speed_wpm = 0.0;
        self.total_chars_typed = 0;
        self.accuracy = 0.0;
        self.time_taken = 0.0;
        self.test_complete = false;
        pancurses::curs_set(1);
    }

    /// Load next of previous text snippet from database.
    fn switch_text(&mut self, win: &pancurses::Window, direction: i32) -> AppResult<()> {
        win.clear();

        let text_id = self.text_id.parse::<i32>()? + direction;
        self.text_id = text_id.to_string();
        self.text = load_text_from_database(text_id as u32, "data.db")?.0;
        self.tokens = self.text
            .split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect();
        self.text = self.tokens.join(" ");
        self.text_backup = self.text.clone();

        self.text = word_wrap(&self.text, self.window_width)?;

        self.reset_test();
        self.setup_print(win)?;
        self.update_state(win)?;
        Ok(())
    }
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
