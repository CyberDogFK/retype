use std::time;

/// Get time elapsed since initial keypress.
/// This is required to calculate speed.
/// # Arguments:
/// * `start_time` - The time when user starts typing the sample text.
/// # Returns:
/// * `f64` - The time elapsed since initial keypress.
pub fn get_elapsed_minutes_since_first_keypress(start_time: f64) -> f64 {
    let system_time = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64() - start_time;
    system_time / 60.0
}