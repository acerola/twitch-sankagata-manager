use twitch_sankagata_manager_lib::config::{Config, Theme};

#[test]
fn old_config_defaults_first_timer_priority_to_on() {
    let old_config = r#"{
        "keyword": "参加券",
        "maxPlaying": 4,
        "maxWaiting": 3,
        "enabled": true,
        "language": "ja",
        "port": 24816
    }"#;

    let cfg: Config = serde_json::from_str(old_config).unwrap();
    assert!(cfg.prioritize_first_timers);
}

#[test]
fn old_config_migrates_keyword_to_first_time() {
    let old_config = r#"{
        "keyword": "参加券",
        "maxPlaying": 4,
        "maxWaiting": 3,
        "enabled": true,
        "language": "ja",
        "port": 24816
    }"#;

    let mut cfg: Config = serde_json::from_str(old_config).unwrap();
    cfg.migrate_legacy();
    assert_eq!(cfg.first_time_keyword, "参加券");
    assert!(cfg.keyword.is_none());
}

#[test]
fn migrate_legacy_does_not_overwrite_existing_keywords() {
    let old_config = r#"{
        "firstTimeKeyword": "custom",
        "keyword": "参加",
        "maxPlaying": 4,
        "maxWaiting": 3,
        "enabled": true,
        "language": "ja",
        "port": 24816
    }"#;

    let mut cfg: Config = serde_json::from_str(old_config).unwrap();
    cfg.migrate_legacy();
    assert_eq!(cfg.first_time_keyword, "custom");
    assert!(cfg.keyword.is_none());
}

#[test]
fn new_config_round_trips_keyword() {
    let new_config = r#"{
        "firstTimeKeyword": "参加券",
        "maxPlaying": 4,
        "maxWaiting": 3,
        "enabled": true,
        "language": "ja",
        "port": 24816
    }"#;

    let cfg: Config = serde_json::from_str(new_config).unwrap();
    assert_eq!(cfg.first_time_keyword, "参加券");
    let serialized = serde_json::to_string(&cfg).unwrap();
    assert!(serialized.contains("\"firstTimeKeyword\":\"参加券\""));
    // Legacy fields should not be serialized when None/empty.
    assert!(!serialized.contains("\"keyword\""));
    assert!(!serialized.contains("\"rewardIds\""));
}

#[test]
fn default_first_time_keyword_is_sankaku() {
    let cfg = Config::default();
    assert_eq!(cfg.first_time_keyword, "参加券");
}

#[test]
fn legacy_theme_id_loads_as_twitch_theme() {
    let old_config = r#"{
        "firstTimeKeyword": "参加券",
        "maxPlaying": 4,
        "maxWaiting": 3,
        "enabled": true,
        "language": "ja",
        "port": 24816,
        "theme": "mumamuma"
    }"#;

    let cfg: Config = serde_json::from_str(old_config).unwrap();
    assert_eq!(cfg.theme, Theme::Twitch);

    let serialized = serde_json::to_string(&cfg).unwrap();
    assert!(serialized.contains("\"theme\":\"twitch\""));
}
