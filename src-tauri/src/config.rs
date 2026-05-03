use serde::{Deserialize, Serialize};

pub const MAX_USER_SETTING: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomColors {
    pub bg: String,
    pub primary: String,
    pub secondary: String,
    pub tertiary: String,
    pub text: String,
}

impl Default for CustomColors {
    fn default() -> Self {
        Self {
            bg: "#0f0f18".to_string(),
            primary: "#16e9f3".to_string(),
            secondary: "#ff7bc4".to_string(),
            tertiary: "#ff4fa8".to_string(),
            text: "#e8e8f0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[serde(alias = "mumamuma")]
    Twitch,
    Midnight,
    Daylight,
    Sakura,
    Forest,
    Contrast,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    // Keyword to detect participation reward redemptions.
    // Any reward whose title contains this keyword is accepted.
    #[serde(default = "default_first_time_keyword")]
    pub first_time_keyword: String,

    // Legacy fields — kept for backward compat on load, cleared on save.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,

    pub max_playing: u32,
    pub max_waiting: u32,
    #[serde(default = "default_prioritize_first_timers")]
    pub prioritize_first_timers: bool,
    pub enabled: bool,
    pub language: Language,
    pub port: u16,
    /// Enable mock mode for local Twitch CLI testing (ignores real Twitch)
    #[serde(default)]
    pub mock_mode: bool,
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_colors: Option<CustomColors>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Ja,
    En,
    Ko,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            first_time_keyword: default_first_time_keyword(),
            reward_ids: None,
            reward_id: None,
            keyword: None,
            max_playing: 4,
            max_waiting: 3,
            prioritize_first_timers: true,
            enabled: true,
            language: Language::Ja,
            port: 24816,
            mock_mode: false,
            theme: Theme::Twitch,
            custom_colors: None,
        }
    }
}

impl Config {
    /// One-shot migration from legacy fields. Call once after deserialising.
    pub fn migrate_legacy(&mut self) {
        // If keyword is still at default, try to pull from legacy `keyword` field.
        if self.first_time_keyword == default_first_time_keyword() {
            if let Some(kw) = self.keyword.take() {
                if !kw.is_empty() {
                    self.first_time_keyword = kw;
                }
            }
        } else {
            self.keyword = None;
        }

        // Drop legacy reward IDs — there is no reliable way to turn opaque
        // Twitch reward IDs back into human-readable keywords.
        self.reward_ids = None;
        self.reward_id = None;
    }

    pub fn clamp_user_limits(&mut self) {
        self.max_playing = self.max_playing.min(MAX_USER_SETTING);
        self.max_waiting = self.max_waiting.min(MAX_USER_SETTING);
    }
}

fn default_first_time_keyword() -> String {
    "参加券".to_string()
}

fn default_prioritize_first_timers() -> bool {
    true
}

fn default_theme() -> Theme {
    Theme::Twitch
}
