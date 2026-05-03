use twitch_sankagata_manager_lib::model::User;
use twitch_sankagata_manager_lib::priority::{
    is_first_time_today, sort_waiting, sort_waiting_with_first_timer_priority,
};

fn user(id: &str, count: u32, last: Option<i64>, enq: i64, manual: Option<i64>) -> User {
    User {
        id: id.into(),
        name: id.into(),
        display_name: id.into(),
        join_count: count,
        last_join_at: last,
        enqueued_at: enq,
        manual_order: manual,
        first_time_today: count == 0,
    }
}

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const NOW: i64 = 1_700_000_000_000;

#[test]
fn first_time_when_never_joined() {
    assert!(is_first_time_today(&user("u", 0, None, 0, None), NOW));
}

#[test]
fn first_time_when_24h_passed() {
    let past = NOW - DAY_MS - 1;
    assert!(is_first_time_today(&user("u", 3, Some(past), 0, None), NOW));
}

#[test]
fn not_first_time_when_within_24h() {
    let recent = NOW - DAY_MS + 1;
    assert!(!is_first_time_today(
        &user("u", 1, Some(recent), 0, None),
        NOW
    ));
}

#[test]
fn sort_puts_first_timers_above_repeats() {
    let repeat = user("r", 2, Some(NOW - 1000), 100, None);
    let first = user("f", 0, None, 200, None);
    let sorted = sort_waiting(vec![repeat.clone(), first.clone()], NOW);
    assert_eq!(sorted[0].id, "f");
    assert_eq!(sorted[1].id, "r");
}

#[test]
fn sort_can_disable_first_timer_priority() {
    let repeat = user("r", 2, Some(NOW - 1000), 100, None);
    let first = user("f", 0, None, 200, None);
    let sorted =
        sort_waiting_with_first_timer_priority(vec![repeat.clone(), first.clone()], NOW, false);
    assert_eq!(sorted[0].id, "r");
    assert_eq!(sorted[1].id, "f");
}

#[test]
fn sort_breaks_tie_with_enqueued_at_fifo() {
    let a = user("a", 0, None, 100, None);
    let b = user("b", 0, None, 50, None);
    let sorted = sort_waiting(vec![a, b], NOW);
    assert_eq!(sorted[0].id, "b");
    assert_eq!(sorted[1].id, "a");
}

#[test]
fn manual_order_overrides_priority() {
    let first = user("f", 0, None, 100, None);
    let repeat_pinned = user("r", 5, Some(NOW - 1000), 50, Some(0));
    let sorted = sort_waiting(vec![first, repeat_pinned], NOW);
    assert_eq!(sorted[0].id, "r");
    assert_eq!(sorted[1].id, "f");
}

#[test]
fn multiple_manual_orders_sorted_ascending() {
    let a = user("a", 0, None, 100, Some(2));
    let b = user("b", 0, None, 50, Some(0));
    let c = user("c", 0, None, 200, Some(1));
    let sorted = sort_waiting(vec![a, b, c], NOW);
    assert_eq!(sorted[0].id, "b");
    assert_eq!(sorted[1].id, "c");
    assert_eq!(sorted[2].id, "a");
}
