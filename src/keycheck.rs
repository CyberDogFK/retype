/// Detect ESC key
pub fn is_escape(key: &pancurses::Input) -> bool {
    match key {
        pancurses::Input::Character(c) => *c == 27 as char,
        _ => false
    }
}

pub fn is_ctrl_c(key_values: &pancurses::Input) -> bool {
    match key_values {
        pancurses::Input::Character(c) => *c == '\x03',
        _ => false
    }
}

/// Detect if the pressed key is a valid key to start timer
pub fn is_valid_initial_key(key: &pancurses::Input) -> bool { 
    match key {
        pancurses::Input::Character(c) => c.is_alphabetic(),
        _ => false
    }
}

/// Detect if terminal was resized
pub fn is_resize(key: &pancurses::Input) -> bool {
    match key {
        pancurses::Input::KeyResize => true,
        _ => false
    }
}

pub fn is_backspace(key: &pancurses::Input) -> bool {
    match key {
        pancurses::Input::KeyBackspace => true,
        _ => false
    }
}

pub fn is_ctrl_backspace(key: &pancurses::Input) -> bool {
    match key {
        pancurses::Input::Character(c) => *c == '\x17',
        _ => false
    }
}

pub fn get_key_mapping(key: &pancurses::Input) -> String {
    match key {
        pancurses::Input::Character(c) => c.to_string(),
        c => {
            format!("{:?}", c)
        }
    }
}