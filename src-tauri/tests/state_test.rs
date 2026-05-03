#![allow(clippy::field_reassign_with_default)]

use std::sync::Arc;
use tempfile::tempdir;
use twitch_sankagata_manager_lib::config::Config;
use twitch_sankagata_manager_lib::model::{User, Zone};
use twitch_sankagata_manager_lib::state::{now_ms, AppState};
use twitch_sankagata_manager_lib::store::Store;

fn mk_user(id: &str) -> User {
    User {
        id: id.into(),
        name: id.into(),
        display_name: id.into(),
        join_count: 0,
        last_join_at: None,
        enqueued_at: 0,
        manual_order: None,
        first_time_today: true,
    }
}

fn mk_repeat_user(id: &str, last_join_at: i64) -> User {
    let mut user = mk_user(id);
    user.join_count = 2;
    user.last_join_at = Some(last_join_at);
    user.first_time_today = false;
    user
}

fn new_state() -> AppState {
    new_state_with_store().0
}

fn new_state_with_store() -> (AppState, Arc<Store>) {
    let dir = tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("s.db")).unwrap());
    (AppState::new(store.clone()), store)
}

#[test]
fn first_timer_priority_is_enabled_by_default_in_state_queue() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 0;
    s.set_config(cfg).unwrap();

    s.add_redemption(mk_repeat_user("repeat", now_ms() - 1_000), 2_000)
        .unwrap();
    s.add_redemption(mk_user("first"), 3_000).unwrap();

    let snap = s.snapshot();
    assert_eq!(snap.waiting[0].id, "first");
    assert_eq!(snap.waiting[1].id, "repeat");
}

#[test]
fn first_timer_priority_can_be_disabled_and_reenabled() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 0;
    cfg.prioritize_first_timers = false;
    s.set_config(cfg.clone()).unwrap();

    s.add_redemption(mk_repeat_user("repeat", now_ms() - 1_000), 2_000)
        .unwrap();
    s.add_redemption(mk_user("first"), 3_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.waiting[0].id, "repeat");
    assert_eq!(snap.waiting[1].id, "first");

    cfg.prioritize_first_timers = true;
    s.set_config(cfg).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.waiting[0].id, "first");
    assert_eq!(snap.waiting[1].id, "repeat");
}

#[test]
fn add_redemption_goes_to_waiting() {
    // max_playing=0 short-circuits promotion so raw add path is observable.
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 0;
    s.set_config(cfg).unwrap();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.waiting.len(), 1);
    assert_eq!(snap.playing.len(), 0);
}

#[test]
fn auto_promote_fills_playing_slots() {
    let s = new_state();
    for i in 0..3 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 3);
    assert_eq!(snap.waiting.len(), 0);
}

#[test]
fn auto_promote_stops_at_max_playing() {
    let s = new_state();
    for i in 0..6 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 4);
    assert_eq!(snap.waiting.len(), 2);
}

#[test]
fn promotion_increments_join_count_and_stamps_last_join() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    let p = &snap.playing[0];
    assert_eq!(p.join_count, 1);
    assert!(p.last_join_at.is_some());
}

#[test]
fn dedupe_redemption_for_existing_user() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.add_redemption(mk_user("u1"), 2_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len() + snap.waiting.len(), 1);
}

#[test]
fn trash_sends_user_to_trash_and_auto_promotes_waiting() {
    let s = new_state();
    for i in 0..5 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    s.trash_user("u0").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 4);
    assert_eq!(snap.waiting.len(), 0);
    assert_eq!(snap.trash.len(), 1);
    assert!(snap.playing.iter().any(|u| u.id == "u4"));
}

#[test]
fn clear_playing_user_archives_without_trash_and_auto_promotes_waiting() {
    let (s, store) = new_state_with_store();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..3 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }

    s.clear_playing_user("u0").unwrap();
    let snap = s.snapshot();
    let (_cleared, zone) = store.load_user("u0").unwrap().unwrap();

    assert_eq!(zone, Zone::History);
    assert_eq!(snap.trash.len(), 0);
    assert_eq!(snap.playing.len(), 2);
    assert!(snap.playing.iter().all(|u| u.id != "u0"));
    assert!(snap.playing.iter().any(|u| u.id == "u2"));
}

#[test]
fn clear_playing_archives_current_match_and_keeps_waiting_flow() {
    let (s, store) = new_state_with_store();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..4 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }

    s.clear_playing().unwrap();
    let snap = s.snapshot();
    let (_u0, u0_zone) = store.load_user("u0").unwrap().unwrap();
    let (_u1, u1_zone) = store.load_user("u1").unwrap().unwrap();

    assert_eq!(u0_zone, Zone::History);
    assert_eq!(u1_zone, Zone::History);
    assert_eq!(snap.trash.len(), 0);
    assert_eq!(snap.waiting.len(), 0);
    assert_eq!(
        snap.playing
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["u2", "u3"]
    );
}

#[test]
fn trash_keeps_only_the_newest_ten_users() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 0;
    s.set_config(cfg).unwrap();
    for i in 0..12 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
        s.trash_user(&format!("u{i}")).unwrap();
    }

    let snap = s.snapshot();
    let ids: Vec<&str> = snap.trash.iter().map(|u| u.id.as_str()).collect();

    assert_eq!(ids.len(), 10);
    assert_eq!(
        ids,
        ["u11", "u10", "u9", "u8", "u7", "u6", "u5", "u4", "u3", "u2"]
    );
}

#[test]
fn restore_user_returns_to_waiting_end_without_auto_promote() {
    // Spec: restore sends user to waiting tail; does NOT auto-promote even when
    // a playing slot is free. Streamer must explicitly move them back.
    let s = new_state();
    // Fill 2 slots in playing; restore target sits in trash, waiting empty.
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.add_redemption(mk_user("u2"), 2_000).unwrap();
    s.trash_user("u1").unwrap();
    let snap0 = s.snapshot();
    assert!(snap0.playing.iter().all(|u| u.id != "u1"));
    s.restore_user("u1").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.trash.len(), 0);
    assert!(snap.waiting.iter().any(|u| u.id == "u1"));
    assert!(snap.playing.iter().all(|u| u.id != "u1"));
}

#[test]
fn refund_removes_user_and_allows_fresh_add() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.refund_user("u1").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 0);
    assert_eq!(snap.waiting.len(), 0);
    s.add_redemption(mk_user("u1"), 2_000).unwrap();
    let snap2 = s.snapshot();
    let u = snap2.playing.iter().find(|u| u.id == "u1").unwrap();
    assert_eq!(u.join_count, 1);
}

#[test]
fn disabled_skips_adds_but_keeps_queue() {
    let mut cfg = Config::default();
    cfg.enabled = false;
    let s = new_state();
    s.set_config(cfg).unwrap();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 0);
    assert_eq!(snap.waiting.len(), 0);
}

#[test]
fn move_user_places_at_requested_index() {
    let s = new_state();
    for i in 0..5 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    s.move_user("u4", Zone::Waiting, 0).unwrap();
    let snap = s.snapshot();
    let idx = snap.waiting.iter().position(|u| u.id == "u4").unwrap();
    assert_eq!(idx, 0);
}

#[test]
fn trash_removes_only_the_targeted_user() {
    // Regression guard: clicking trash on user U must remove U exactly, never a neighbor.
    let s = new_state();
    for i in 0..5 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    // Pre-condition: u0..u3 promoted to playing (max 4), u4 waiting.
    s.trash_user("u2").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.trash.len(), 1);
    assert_eq!(snap.trash[0].id, "u2");
    // u2 is gone from both lists, other users preserved.
    for id in ["u0", "u1", "u3", "u4"] {
        let present = snap
            .playing
            .iter()
            .chain(snap.waiting.iter())
            .any(|u| u.id == id);
        assert!(present, "{id} must still exist after trashing u2");
    }
}

#[test]
fn move_user_to_huge_index_clamps_to_end() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 0; // keep everyone in waiting for deterministic indices
    s.set_config(cfg).unwrap();
    for i in 0..3 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    s.move_user("u0", Zone::Waiting, 999).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.waiting.last().map(|u| u.id.as_str()), Some("u0"));
}

#[test]
fn move_waiting_user_to_full_playing_is_rejected() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..3 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }

    let err = s.move_user("u2", Zone::Playing, 0).unwrap_err();
    let snap = s.snapshot();

    assert!(err.to_string().contains("playing list is full"));
    assert_eq!(snap.playing.len(), 2);
    assert_eq!(snap.waiting.len(), 1);
    assert_eq!(snap.waiting[0].id, "u2");
}

#[test]
fn move_trash_user_to_full_playing_is_rejected() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..3 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    s.trash_user("u2").unwrap();

    let err = s.move_user("u2", Zone::Playing, 0).unwrap_err();
    let snap = s.snapshot();

    assert!(err.to_string().contains("playing list is full"));
    assert_eq!(snap.playing.len(), 2);
    assert_eq!(snap.trash.len(), 1);
    assert_eq!(snap.trash[0].id, "u2");
}

#[test]
fn move_within_full_playing_still_reorders() {
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..2 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }

    s.move_user("u1", Zone::Playing, 0).unwrap();
    let snap = s.snapshot();

    assert_eq!(snap.playing.len(), 2);
    assert_eq!(snap.playing[0].id, "u1");
    assert_eq!(snap.playing[1].id, "u0");
}

#[test]
fn lowering_max_playing_demotes_overflow_to_waiting() {
    let s = new_state();
    for i in 0..4 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();

    let snap = s.snapshot();

    assert_eq!(
        snap.playing
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["u0", "u1"]
    );
    assert_eq!(
        snap.waiting
            .iter()
            .map(|u| u.id.as_str())
            .collect::<Vec<_>>(),
        ["u2", "u3"]
    );
}

#[test]
fn refund_of_playing_user_auto_promotes_waiting_head() {
    let s = new_state();
    for i in 0..6 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    // Playing = u0..u3, waiting = u4, u5 (maxPlaying default 4)
    s.refund_user("u1").unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.playing.len(), 4);
    assert!(snap.playing.iter().any(|u| u.id == "u4"));
    assert!(!snap.playing.iter().any(|u| u.id == "u1"));
}

#[test]
fn move_from_playing_to_waiting_does_not_snap_back() {
    // Dragging a playing user into waiting must not be immediately re-promoted by
    // auto_promote — that would make drag-to-rest feel broken.
    let s = new_state();
    for i in 0..4 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    // Pre: playing = u0..u3, waiting empty.
    s.move_user("u0", Zone::Waiting, 0).unwrap();
    let snap = s.snapshot();
    assert!(snap.waiting.iter().any(|u| u.id == "u0"));
    assert!(snap.playing.iter().all(|u| u.id != "u0"));
}

#[test]
fn move_into_playing_still_promotes_from_waiting_to_fill_gap() {
    // Regression guard: skipping auto_promote must only apply to playing→waiting.
    // Other transitions should still fill free playing slots.
    let s = new_state();
    let mut cfg = s.config();
    cfg.max_playing = 2;
    s.set_config(cfg).unwrap();
    for i in 0..4 {
        s.add_redemption(mk_user(&format!("u{i}")), i as i64)
            .unwrap();
    }
    // Pre: playing = u0, u1 ; waiting = u2, u3.
    s.trash_user("u0").unwrap();
    let snap = s.snapshot();
    // Trash freed a slot → u2 promoted automatically.
    assert_eq!(snap.playing.len(), 2);
    assert!(snap.playing.iter().any(|u| u.id == "u2"));
}

#[test]
fn reset_counts_zeroes_history() {
    let s = new_state();
    s.add_redemption(mk_user("u1"), 1_000).unwrap();
    s.reset_counts().unwrap();
    let snap = s.snapshot();
    let u = snap
        .playing
        .iter()
        .chain(snap.waiting.iter())
        .find(|u| u.id == "u1")
        .unwrap();
    assert_eq!(u.join_count, 0);
    assert_eq!(u.last_join_at, None);
}
