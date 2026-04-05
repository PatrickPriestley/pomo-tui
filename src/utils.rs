pub fn format_duration(total_secs: u32) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0m");
        assert_eq!(format_duration(25 * 60), "25m");
        assert_eq!(format_duration(60 * 60), "1h 0m");
        assert_eq!(format_duration(90 * 60), "1h 30m");
        assert_eq!(format_duration(2 * 3600 + 15 * 60), "2h 15m");
    }
}
