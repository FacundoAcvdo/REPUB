use crate::domain::stats::ReadingStats;

pub fn mock_stats() -> ReadingStats {
    ReadingStats {
        books_completed: 14,
        pages_read_total: 4210,
        pages_read_this_year: 1890,
        reading_hours: 87.5,
        current_streak_days: 18,
        weekly_activity: [42, 35, 60, 50, 75, 90, 48],
    }
}
