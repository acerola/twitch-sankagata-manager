#![allow(clippy::field_reassign_with_default)]

use twitch_sankagata_manager_lib::config::Config;
use twitch_sankagata_manager_lib::eventsub::{Redemption, RewardRef};
use twitch_sankagata_manager_lib::filter::{should_accept, ParsedRedemption};

fn red(reward_id: &str, title: &str) -> Redemption {
    Redemption {
        id: "rd1".into(),
        user_id: "u1".into(),
        user_name: "A".into(),
        user_login: "a".into(),
        reward: RewardRef {
            id: reward_id.into(),
            title: title.into(),
        },
        status: "UNFULFILLED".into(),
    }
}

#[test]
fn accepts_when_keyword_matches() {
    let mut cfg = Config::default();
    cfg.first_time_keyword = "参加券".into();
    assert!(should_accept(&red("x", "参加券を使う"), &cfg));
    assert!(!should_accept(&red("x", "ゲーム無関係"), &cfg));
}

#[test]
fn falls_back_to_legacy_reward_ids() {
    let mut cfg = Config::default();
    cfg.first_time_keyword = String::new();
    cfg.reward_ids = Some(vec!["target".into()]);
    assert!(should_accept(&red("target", "anything"), &cfg));
    assert!(!should_accept(&red("other", "anything"), &cfg));
}

#[test]
fn falls_back_to_legacy_keyword() {
    let mut cfg = Config::default();
    cfg.first_time_keyword = String::new();
    cfg.keyword = Some("参加".into());
    assert!(should_accept(&red("x", "参加する"), &cfg));
    assert!(!should_accept(&red("x", "ゲーム無関係"), &cfg));
}

#[test]
fn new_keyword_wins_over_legacy() {
    let mut cfg = Config::default();
    cfg.first_time_keyword = "参加券".into();
    cfg.reward_ids = Some(vec!["target".into()]);
    // New keyword takes precedence
    assert!(should_accept(&red("other", "参加券!"), &cfg));
    assert!(!should_accept(&red("other", "ゲーム"), &cfg));
}

#[test]
fn migrate_legacy_moves_keyword_to_first_time() {
    let mut cfg = Config::default();
    cfg.keyword = Some("参加券".into());
    cfg.migrate_legacy();
    assert_eq!(cfg.first_time_keyword, "参加券");
    assert!(cfg.keyword.is_none());
}

#[test]
fn migrate_legacy_does_not_overwrite_existing() {
    let mut cfg = Config::default();
    cfg.first_time_keyword = "custom".into();
    cfg.keyword = Some("参加券".into());
    cfg.migrate_legacy();
    assert_eq!(cfg.first_time_keyword, "custom");
}

#[test]
fn parsed_redemption_maps_fields() {
    let r = red("r", "t");
    let p = ParsedRedemption::from(&r);
    assert_eq!(p.user.id, "u1");
    assert_eq!(p.user.display_name, "A");
    assert_eq!(p.redemption_id, "rd1");
}
