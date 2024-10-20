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