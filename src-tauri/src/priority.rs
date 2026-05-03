use crate::model::User;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub fn is_first_time_today(user: &User, now_ms: i64) -> bool {
    match user.last_join_at {
        None => true,
        Some(t) => (now_ms - t) > DAY_MS,
    }
}

pub fn sort_waiting(users: Vec<User>, now_ms: i64) -> Vec<User> {
    sort_waiting_with_first_timer_priority(users, now_ms, true)
}

pub fn sort_waiting_with_first_timer_priority(
    mut users: Vec<User>,
    now_ms: i64,
    prioritize_first_timers: bool,
) -> Vec<User> {
    users.sort_by(|a, b| {
        match (a.manual_order, b.manual_order) {
            (Some(x), Some(y)) => return x.cmp(&y),
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        if prioritize_first_timers {
            let a_first = is_first_time_today(a, now_ms);
            let b_first = is_first_time_today(b, now_ms);
            if a_first != b_first {
                return b_first.cmp(&a_first);
            }
        }
        a.enqueued_at.cmp(&b.enqueued_at)
    });
    users
}
