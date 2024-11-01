use std::time;
use std::time::{SystemTime, SystemTimeError};

/// Get time elapsed since initial keypress.
/// This is required to calculate speed.
/// # Arguments:
/// * `start_time` - The time when user starts typing the sample text.
/// # Returns:
/// * `f64` - The time elapsed since initial keypress.
pub fn get_elapsed_minutes_since_first_keypress(start_time: SystemTime) -> Result<f64, SystemTimeError> {
    let system_time = SystemTime::now()
        .duration_since(time::UNIX_EPOCH)?
        .as_secs_f64()
        - start_time
        .duration_since(time::UNIX_EPOCH)?
        .as_secs_f64();
    Ok(system_time / 60.0)
}
