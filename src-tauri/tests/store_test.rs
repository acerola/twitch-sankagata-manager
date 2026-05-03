use tempfile::tempdir;
use twitch_sankagata_manager_lib::model::{User, Zone};
use twitch_sankagata_manager_lib::store::Store;

#[test]
fn opens_and_migrates_fresh_db() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.db");
    let store = Store::open(&path).unwrap();
    assert!(path.exists());
    let cfg = store.load_config().unwrap();
    assert_eq!(cfg.first_time_keyword, "参加券");
    assert!(cfg.keyword.is_none());
    assert_eq!(cfg.max_playing, 4);
}

#[test]
fn upserts_and_loads_user() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(),
        name: "alice".into(),
        display_name: "Alice".into(),
        join_count: 2,
        last_join_at: Some(1_700_000_000_000),
        enqueued_at: 1_700_000_000_500,
        manual_order: None,
        first_time_today: false,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    let loaded = store.load_zone(Zone::Waiting).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name, "Alice");
}

#[test]
fn moves_user_between_zones() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(),
        name: "a".into(),
        display_name: "A".into(),
        join_count: 0,
        last_join_at: None,
        enqueued_at: 1,
        manual_order: None,
        first_time_today: true,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    store.move_user("u1", Zone::Playing, 0).unwrap();
    assert!(store.load_zone(Zone::Waiting).unwrap().is_empty());
    assert_eq!(store.load_zone(Zone::Playing).unwrap().len(), 1);
}

#[test]
fn reset_counts_wipes_history_only() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    let user = User {
        id: "u1".into(),
        name: "a".into(),
        display_name: "A".into(),
        join_count: 5,
        last_join_at: Some(1),
        enqueued_at: 10,
        manual_order: None,
        first_time_today: false,
    };
    store.upsert_user(&user, Zone::Waiting, 0).unwrap();
    store.reset_counts().unwrap();
    let u = &store.load_zone(Zone::Waiting).unwrap()[0];
    assert_eq!(u.join_count, 0);
    assert_eq!(u.last_join_at, None);
}

#[test]
fn trim_trash_keeps_newest() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path().join("s.db")).unwrap();
    for i in 0..5 {
        let u = User {
            id: format!("u{i}"),
            name: format!("u{i}"),
            display_name: format!("u{i}"),
            join_count: 0,
            last_join_at: None,
            enqueued_at: i,
            manual_order: None,
            first_time_today: true,
        };
        store.upsert_user(&u, Zone::Trash, i).unwrap();
    }
    store.trim_trash(3).unwrap();
    let trash = store.load_zone(Zone::Trash).unwrap();
    assert_eq!(trash.len(), 3);
    let ids: Vec<String> = trash.iter().map(|u| u.id.clone()).collect();
    // positions 0,1,2 retained — newest (by our convention position=0 is newest)
    assert_eq!(
        ids,
        vec!["u0".to_string(), "u1".to_string(), "u2".to_string()]
    );
}
