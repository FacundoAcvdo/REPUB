use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub unlocked: bool,
    pub unlocked_at: Option<String>,
    pub icon_name: String,
}
