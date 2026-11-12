use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadingStats {
    pub books_completed: u32,
    pub pages_read_total: u32,
    pub pages_read_this_year: u32,
    pub reading_hours: f32,
    pub current_streak_days: u32,
    pub weekly_activity: [u32; 7],
}
