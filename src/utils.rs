use crate::persistence::models::DaySummary;

pub fn format_duration(total_secs: u32) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn weekly_total_seconds(stats: &[DaySummary]) -> u32 {
    stats.iter().map(|s| s.total_seconds).sum()
}

pub fn weekly_session_count(stats: &[DaySummary]) -> u32 {
    stats.iter().map(|s| s.session_count).sum()
}

pub fn weekly_completion_rate(stats: &[DaySummary]) -> f32 {
    let total: u32 = stats.iter().map(|s| s.session_count).sum();
    let completed: u32 = stats.iter().map(|s| s.completed_count).sum();
    if total == 0 {
        0.0
    } else {
        (completed as f32 / total as f32) * 100.0
    }
}

/// Average seconds per day, only counting days with at least one session.
pub fn weekly_daily_average_seconds(stats: &[DaySummary]) -> u32 {
    let active_days: Vec<&DaySummary> = stats.iter().filter(|s| s.session_count > 0).collect();
    if active_days.is_empty() {
        return 0;
    }
    let total: u32 = active_days.iter().map(|s| s.total_seconds).sum();
    total / active_days.len() as u32
}

pub fn weekly_best_day(stats: &[DaySummary]) -> Option<&DaySummary> {
    stats.iter().filter(|s| s.session_count > 0).max_by_key(|s| s.total_seconds)
}

pub fn weekly_total_interruptions(stats: &[DaySummary]) -> u32 {
    stats.iter().map(|s| s.interruption_count).sum()
}

pub fn weekly_interruptions_per_session(stats: &[DaySummary]) -> f32 {
    let total_sessions: u32 = stats.iter().map(|s| s.session_count).sum();
    let total_interruptions: u32 = stats.iter().map(|s| s.interruption_count).sum();
    if total_sessions == 0 {
        0.0
    } else {
        total_interruptions as f32 / total_sessions as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_day(date_str: &str, secs: u32, sessions: u32, completed: u32, interruptions: u32) -> DaySummary {
        DaySummary {
            date: NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap(),
            total_seconds: secs,
            session_count: sessions,
            completed_count: completed,
            interruption_count: interruptions,
        }
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0m");
        assert_eq!(format_duration(25 * 60), "25m");
        assert_eq!(format_duration(60 * 60), "1h 0m");
        assert_eq!(format_duration(90 * 60), "1h 30m");
        assert_eq!(format_duration(2 * 3600 + 15 * 60), "2h 15m");
    }

    #[test]
    fn test_weekly_total_seconds() {
        let stats = vec![
            make_day("2026-04-01", 25 * 60, 1, 1, 0),
            make_day("2026-04-02", 50 * 60, 2, 2, 0),
            make_day("2026-04-03", 0, 0, 0, 0),
        ];
        assert_eq!(weekly_total_seconds(&stats), 75 * 60);
    }

    #[test]
    fn test_weekly_completion_rate() {
        let stats = vec![
            make_day("2026-04-01", 50 * 60, 2, 2, 0),
            make_day("2026-04-02", 50 * 60, 2, 1, 0),
        ];
        assert_eq!(weekly_completion_rate(&stats), 75.0);
    }

    #[test]
    fn test_weekly_completion_rate_no_sessions() {
        let stats = vec![make_day("2026-04-01", 0, 0, 0, 0)];
        assert_eq!(weekly_completion_rate(&stats), 0.0);
    }

    #[test]
    fn test_weekly_daily_average_ignores_zero_days() {
        let stats = vec![
            make_day("2026-04-01", 60 * 60, 2, 2, 0), // 1 hour
            make_day("2026-04-02", 0, 0, 0, 0),         // no sessions
            make_day("2026-04-03", 30 * 60, 1, 1, 0),   // 30 min
        ];
        // Average of 1h and 30m = 45m = 2700s
        assert_eq!(weekly_daily_average_seconds(&stats), 2700);
    }

    #[test]
    fn test_weekly_best_day() {
        let stats = vec![
            make_day("2026-04-01", 25 * 60, 1, 1, 0),
            make_day("2026-04-02", 75 * 60, 3, 3, 0),
            make_day("2026-04-03", 50 * 60, 2, 2, 0),
        ];
        let best = weekly_best_day(&stats).unwrap();
        assert_eq!(best.date.to_string(), "2026-04-02");
        assert_eq!(best.total_seconds, 75 * 60);
    }

    #[test]
    fn test_weekly_total_interruptions() {
        let stats = vec![
            make_day("2026-04-01", 25 * 60, 1, 1, 3),
            make_day("2026-04-02", 50 * 60, 2, 2, 5),
        ];
        assert_eq!(weekly_total_interruptions(&stats), 8);
    }

    #[test]
    fn test_weekly_interruptions_per_session_no_sessions() {
        let stats = vec![make_day("2026-04-01", 0, 0, 0, 0)];
        assert_eq!(weekly_interruptions_per_session(&stats), 0.0);
    }
}
