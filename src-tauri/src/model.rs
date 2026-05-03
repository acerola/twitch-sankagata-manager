use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub join_count: u32,
    pub last_join_at: Option<i64>, // unix millis utc
    pub enqueued_at: i64,
    pub manual_order: Option<i64>,
    /// Sticky session badge — stamped at add_redemption from prior history,
    /// preserved across promotion to playing. Distinct from `last_join_at`,
    /// which gets bumped on promotion for next-day priority logic.
    #[serde(default = "default_true")]
    pub first_time_today: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Zone {
    Playing,
    Waiting,
    Trash,
    /// Users cleared from the visible queue. Retained for history
    /// (join_count, last_join_at) but never shown in any UI pane.
    History,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    #[serde(rename = "type")]
    pub kind: &'static str, // "state"
    pub playing: Vec<User>,
    pub waiting: Vec<User>,
    pub waiting_total: usize,
    pub trash: Vec<User>,
    pub enabled: bool,
    pub language: String,
    pub max_waiting: u32,
    pub theme: String,
}

impl Snapshot {
    pub fn new(
        playing: Vec<User>,
        waiting: Vec<User>,
        trash: Vec<User>,
        enabled: bool,
        language: String,
        max_waiting: u32,
        theme: String,
    ) -> Self {
        let waiting_total = waiting.len();
        Self {
            kind: "state",
            playing,
            waiting,
            waiting_total,
            trash,
            enabled,
            language,
            max_waiting,
            theme,
        }
    }
}
