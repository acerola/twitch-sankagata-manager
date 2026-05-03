use crate::config::Config;
use crate::eventsub::Redemption;
use crate::model::User;

pub fn should_accept(r: &Redemption, cfg: &Config) -> bool {
    // If a keyword is set, use it for matching (contains check).
    if !cfg.first_time_keyword.is_empty() {
        return r.reward.title.contains(&cfg.first_time_keyword);
    }

    // Legacy fallback — reward IDs.
    if let Some(ref ids) = cfg.reward_ids {
        if !ids.is_empty() {
            return ids.iter().any(|id| id == &r.reward.id);
        }
    }
    if let Some(ref legacy) = cfg.reward_id {
        return r.reward.id == legacy.as_str();
    }

    // Legacy fallback — single keyword.
    if let Some(ref kw) = cfg.keyword {
        return r.reward.title.contains(kw);
    }

    false
}

pub struct ParsedRedemption {
    pub user: User,
    pub redemption_id: String,
    pub reward_id: String,
}

impl From<&Redemption> for ParsedRedemption {
    fn from(r: &Redemption) -> Self {
        Self {
            user: User {
                id: r.user_id.clone(),
                name: r.user_login.clone(),
                display_name: r.user_name.clone(),
                join_count: 0,
                last_join_at: None,
                enqueued_at: 0,
                manual_order: None,
                first_time_today: true,
            },
            redemption_id: r.id.clone(),
            reward_id: r.reward.id.clone(),
        }
    }
}
