use crate::config::Config;
use crate::error::{AppError, Result};
use crate::model::{Snapshot, User, Zone};
use crate::priority::sort_waiting_with_first_timer_priority;
use crate::store::Store;
use std::sync::{Arc, Mutex};

const TRASH_CAP: usize = 10;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub struct AppState {
    inner: Mutex<Inner>,
    store: Arc<Store>,
    session_id: Mutex<Option<String>>,
}

struct Inner {
    playing: Vec<User>,
    waiting: Vec<User>,
    trash: Vec<User>,
    config: Config,
}

impl AppState {
    pub fn new(store: Arc<Store>) -> Self {
        let config = store.load_config().unwrap_or_default();
        let playing = store.load_zone(Zone::Playing).unwrap_or_default();
        let waiting = store.load_zone(Zone::Waiting).unwrap_or_default();
        let trash = store.load_zone(Zone::Trash).unwrap_or_default();
        Self {
            inner: Mutex::new(Inner {
                playing,
                waiting,
                trash,
                config,
            }),
            store,
            session_id: Mutex::new(None),
        }
    }

    pub fn set_session_id(&self, id: String) {
        *self.session_id.lock().unwrap() = Some(id);
    }

    pub fn get_session_id(&self) -> Option<String> {
        self.session_id.lock().unwrap().clone()
    }

    pub fn snapshot(&self) -> Snapshot {
        let g = self.inner.lock().unwrap();
        Snapshot::new(
            g.playing.clone(),
            g.waiting.clone(),
            g.trash.clone(),
            g.config.enabled,
            match g.config.language {
                crate::config::Language::Ja => "ja",
                crate::config::Language::En => "en",
                crate::config::Language::Ko => "ko",
            }
            .into(),
            g.config.max_waiting,
            serde_json::to_string(&g.config.theme)
                .unwrap_or_else(|_| "\"twitch\"".to_string())
                .trim_matches('"')
                .to_string(),
        )
    }

    pub fn config(&self) -> Config {
        self.inner.lock().unwrap().config.clone()
    }

    pub fn set_config(&self, mut cfg: Config) -> Result<()> {
        cfg.clamp_user_limits();
        let mut g = self.inner.lock().unwrap();
        g.config = cfg.clone();
        let waiting = std::mem::take(&mut g.waiting);
        g.waiting = sort_waiting_with_first_timer_priority(
            waiting,
            now_ms(),
            g.config.prioritize_first_timers,
        );
        Self::demote_playing_overflow(&mut g);
        drop(g);
        self.store.save_config(&cfg)?;
        // Raising max_playing can promote users; lowering max_playing demotes
        // overflow above, so persist either queue layout change.
        self.auto_promote();
        self.persist_all()
    }

    pub fn add_redemption(&self, user: User, now_ms: i64) -> Result<()> {
        // Check DB history OUTSIDE the mutex to avoid holding across I/O.
        let prior = self.store.load_user(&user.id).ok().flatten();
        let mut g = self.inner.lock().unwrap();
        if !g.config.enabled {
            return Ok(());
        }
        if g.playing.iter().any(|u| u.id == user.id)
            || g.waiting.iter().any(|u| u.id == user.id)
            || g.trash.iter().any(|u| u.id == user.id)
        {
            return Ok(());
        }
        let mut u = user;
        u.enqueued_at = now_ms;
        // If we have a history row (refund/restore trail), carry forward counts.
        if let Some((existing, _zone)) = prior {
            u.join_count = existing.join_count;
            u.last_join_at = existing.last_join_at;
        }
        // Stamp sticky badge BEFORE auto_promote bumps last_join_at.
        u.first_time_today = match u.last_join_at {
            None => true,
            Some(t) => (now_ms - t) > DAY_MS,
        };
        g.waiting.push(u);
        let waiting = std::mem::take(&mut g.waiting);
        g.waiting = sort_waiting_with_first_timer_priority(
            waiting,
            now_ms,
            g.config.prioritize_first_timers,
        );
        drop(g);
        self.auto_promote();
        self.persist_all()
    }

    fn auto_promote(&self) {
        let mut g = self.inner.lock().unwrap();
        while (g.playing.len() as u32) < g.config.max_playing && !g.waiting.is_empty() {
            let mut promoted = g.waiting.remove(0);
            promoted.join_count += 1;
            promoted.last_join_at = Some(now_ms());
            g.playing.push(promoted);
        }
    }

    fn demote_playing_overflow(g: &mut Inner) {
        let max = g.config.max_playing as usize;
        if g.playing.len() <= max {
            return;
        }
        let overflow = g.playing.split_off(max);
        g.waiting.splice(0..0, overflow);
    }

    pub fn trash_user(&self, user_id: &str) -> Result<()> {
        {
            let mut g = self.inner.lock().unwrap();
            if let Some(pos) = g.playing.iter().position(|u| u.id == user_id) {
                let u = g.playing.remove(pos);
                g.trash.insert(0, u);
            } else if let Some(pos) = g.waiting.iter().position(|u| u.id == user_id) {
                let u = g.waiting.remove(pos);
                g.trash.insert(0, u);
            } else {
                return Err(AppError::Other(format!("user {user_id} not found")));
            }
            if g.trash.len() > TRASH_CAP {
                g.trash.truncate(TRASH_CAP);
            }
        }
        self.auto_promote();
        self.persist_all()
    }

    pub fn clear_trash(&self) -> Result<()> {
        let ids: Vec<String> = {
            let mut g = self.inner.lock().unwrap();
            g.trash.drain(..).map(|u| u.id).collect()
        };
        for id in ids {
            self.store.delete_user(&id).ok();
        }
        self.persist_all()
    }

    pub fn restore_user(&self, user_id: &str) -> Result<()> {
        // Spec: restore back to end of waiting. Do NOT auto-promote —
        // otherwise a freed playing slot immediately pulls the restored user,
        // which is not the streamer's intent.
        {
            let mut g = self.inner.lock().unwrap();
            let pos = g
                .trash
                .iter()
                .position(|u| u.id == user_id)
                .ok_or_else(|| AppError::Other(format!("user {user_id} not in trash")))?;
            let u = g.trash.remove(pos);
            g.waiting.push(u);
        }
        self.persist_all()
    }

    pub fn refund_user(&self, user_id: &str) -> Result<()> {
        // Spec: remove from queue, decrement join_count (may re-grant 初 badge),
        //       keep row in history zone so future redemptions carry forward counts.
        let removed = {
            let mut g = self.inner.lock().unwrap();
            if let Some(pos) = g.playing.iter().position(|u| u.id == user_id) {
                Some(g.playing.remove(pos))
            } else if let Some(pos) = g.waiting.iter().position(|u| u.id == user_id) {
                Some(g.waiting.remove(pos))
            } else {
                // In trash or unknown — leave alone.
                None
            }
        };
        if let Some(mut u) = removed {
            u.join_count = u.join_count.saturating_sub(1);
            if u.join_count == 0 {
                u.last_join_at = None;
            }
            u.manual_order = None;
            self.store.upsert_user(&u, Zone::History, 0).ok();
        }
        self.auto_promote();
        self.persist_all()
    }

    #[allow(clippy::manual_map)]
    pub fn move_user(&self, user_id: &str, zone: Zone, index: usize) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        let source: Option<(Zone, usize)> =
            if let Some(pos) = g.playing.iter().position(|u| u.id == user_id) {
                Some((Zone::Playing, pos))
            } else if let Some(pos) = g.waiting.iter().position(|u| u.id == user_id) {
                Some((Zone::Waiting, pos))
            } else if let Some(pos) = g.trash.iter().position(|u| u.id == user_id) {
                Some((Zone::Trash, pos))
            } else {
                None
            };
        let Some((from, source_index)) = source else {
            return Err(AppError::Other(format!("user {user_id} not found")));
        };
        if zone == Zone::Playing
            && from != Zone::Playing
            && (g.playing.len() as u32) >= g.config.max_playing
        {
            return Err(AppError::Other(format!(
                "playing list is full (max {})",
                g.config.max_playing
            )));
        }
        let mut user = match from {
            Zone::Playing => g.playing.remove(source_index),
            Zone::Waiting => g.waiting.remove(source_index),
            Zone::Trash => g.trash.remove(source_index),
            Zone::History => unreachable!("history is never a move source"),
        };
        user.manual_order = Some(index as i64);
        let dest = match zone {
            Zone::Playing => &mut g.playing,
            Zone::Waiting => &mut g.waiting,
            Zone::Trash => &mut g.trash,
            Zone::History => {
                return Err(AppError::Other("cannot move user into history zone".into()))
            }
        };
        let idx = index.min(dest.len());
        dest.insert(idx, user);
        drop(g);
        // If streamer explicitly demoted a playing user to waiting, DON'T immediately
        // auto-promote — the move would bounce back and feel broken. Any other
        // transition (trash→waiting, reorder within waiting, promote-to-playing) can
        // still trigger promotion to fill free slots.
        let demoted_to_waiting = from == Zone::Playing && zone == Zone::Waiting;
        if !demoted_to_waiting {
            self.auto_promote();
        }
        self.persist_all()
    }

    pub fn clear_playing(&self) -> Result<()> {
        let cleared = {
            let mut g = self.inner.lock().unwrap();
            std::mem::take(&mut g.playing)
        };
        if cleared.is_empty() {
            return Ok(());
        }
        self.archive_cleared_users(cleared)?;
        self.auto_promote();
        self.persist_all()
    }

    pub fn clear_playing_user(&self, user_id: &str) -> Result<()> {
        let cleared = {
            let mut g = self.inner.lock().unwrap();
            let pos = g
                .playing
                .iter()
                .position(|u| u.id == user_id)
                .ok_or_else(|| AppError::Other(format!("user {user_id} not in playing")))?;
            vec![g.playing.remove(pos)]
        };
        self.archive_cleared_users(cleared)?;
        self.auto_promote();
        self.persist_all()
    }

    pub fn reset_counts(&self) -> Result<()> {
        let mut g = self.inner.lock().unwrap();
        for u in g.playing.iter_mut() {
            u.join_count = 0;
            u.last_join_at = None;
            u.manual_order = None;
            u.first_time_today = true;
        }
        for u in g.waiting.iter_mut() {
            u.join_count = 0;
            u.last_join_at = None;
            u.manual_order = None;
            u.first_time_today = true;
        }
        for u in g.trash.iter_mut() {
            u.join_count = 0;
            u.last_join_at = None;
            u.manual_order = None;
            u.first_time_today = true;
        }
        drop(g);
        self.store.reset_counts()?;
        Ok(())
    }

    fn archive_cleared_users(&self, users: Vec<User>) -> Result<()> {
        for (i, mut u) in users.into_iter().enumerate() {
            u.manual_order = None;
            self.store.upsert_user(&u, Zone::History, i as i64)?;
        }
        Ok(())
    }

    #[cfg(debug_assertions)]
    pub fn debug_clear_all(&self) -> Result<()> {
        let ids: Vec<String> = {
            let mut g = self.inner.lock().unwrap();
            let mut ids = Vec::new();
            ids.extend(g.playing.drain(..).map(|u| u.id));
            ids.extend(g.waiting.drain(..).map(|u| u.id));
            ids.extend(g.trash.drain(..).map(|u| u.id));
            ids
        };
        for id in ids {
            self.store.delete_user(&id).ok();
        }
        Ok(())
    }

    #[cfg(debug_assertions)]
    pub fn debug_refund_first_playing(&self) -> Result<()> {
        let target_id = {
            let g = self.inner.lock().unwrap();
            g.playing.first().map(|u| u.id.clone())
        };
        if let Some(id) = target_id {
            self.refund_user(&id)?;
        }
        Ok(())
    }

    fn persist_all(&self) -> Result<()> {
        let (playing, waiting, trash) = {
            let g = self.inner.lock().unwrap();
            (g.playing.clone(), g.waiting.clone(), g.trash.clone())
        };
        self.store
            .persist_visible_zones(&playing, &waiting, &trash, TRASH_CAP)
    }
}

pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
