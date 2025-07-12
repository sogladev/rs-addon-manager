/// Converts a duration in seconds to a human-readable string format.
///
/// # Arguments
///
/// * `seconds` - A floating point number representing the duration in seconds
///
/// # Returns
///
/// A `String` containing the formatted duration in the format:
/// * `"--"` if seconds is <= 0 or > 86400 (1 day)
/// * `"Xh00m00s"` for durations >= 1 hour
/// * `"Xm00s"` for durations >= 1 minute but < 1 hour
/// * `"Xs"` for durations < 1 minute
///
/// # Examples
///
/// ```
/// use rs_manifest_patcher::format::eta_to_human_readable;
/// assert_eq!(eta_to_human_readable(3661.0), "1h01m01s");
/// assert_eq!(eta_to_human_readable(61.0), "1m01s");
/// assert_eq!(eta_to_human_readable(30.0), "30s");
/// assert_eq!(eta_to_human_readable(0.0), "--");
/// ```
pub fn eta_to_human_readable(seconds: f64) -> String {
    if seconds <= 0.0 || seconds > 86400.0 {
        return String::from("--");
    }

    let hours = (seconds / 3600.0).floor();
    let minutes = ((seconds % 3600.0) / 60.0).floor();
    let secs = (seconds % 60.0).floor();

    if hours > 0.0 {
        format!("{}h{:02}m{:02}s", hours as u32, minutes as u32, secs as u32)
    } else if minutes > 0.0 {
        format!("{}m{:02}s", minutes as u32, secs as u32)
    } else {
        format!("{}s", secs as u32)
    }
}
