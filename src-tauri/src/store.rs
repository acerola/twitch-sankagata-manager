use crate::config::Config;
use crate::error::{AppError, Result};
use crate::model::{User, Zone};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

pub struct Store {
    conn: Mutex<Connection>,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    join_count INTEGER NOT NULL DEFAULT 0,
    last_join_at INTEGER,
    enqueued_at INTEGER NOT NULL,
    manual_order INTEGER,
    zone TEXT NOT NULL,
    position INTEGER NOT NULL,
    first_time_today INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_users_zone ON users(zone, position);

CREATE TABLE IF NOT EXISTS config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    blob TEXT NOT NULL
);
"#;

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        // Migration for DBs created before first_time_today existed. Duplicate-column
        // error is expected on already-migrated DBs and is intentionally ignored.
        let _ = conn.execute(
            "ALTER TABLE users ADD COLUMN first_time_today INTEGER NOT NULL DEFAULT 1",
            [],
        );
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM config", [], |r| r.get(0))?;
        if count == 0 {
            let cfg = Config::default();
            conn.execute(
                "INSERT INTO config (id, blob) VALUES (1, ?1)",
                params![serde_json::to_string(&cfg)?],
            )?;
        }
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn load_config(&self) -> Result<Config> {
        let conn = self.conn.lock().unwrap();
        let blob: String =
            conn.query_row("SELECT blob FROM config WHERE id=1", [], |r| r.get(0))?;
        let mut cfg: Config = serde_json::from_str(&blob)?;
        cfg.migrate_legacy();
        cfg.clamp_user_limits();
        Ok(cfg)
    }

    pub fn save_config(&self, cfg: &Config) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE config SET blob = ?1 WHERE id = 1",
            params![serde_json::to_string(cfg)?],
        )?;
        Ok(())
    }

    pub fn upsert_user(&self, user: &User, zone: Zone, position: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO users (id,name,display_name,join_count,last_join_at,enqueued_at,manual_order,zone,position,first_time_today)
            VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
            ON CONFLICT(id) DO UPDATE SET
                name=excluded.name,
                display_name=excluded.display_name,
                join_count=excluded.join_count,
                last_join_at=excluded.last_join_at,
                enqueued_at=excluded.enqueued_at,
                manual_order=excluded.manual_order,
                zone=excluded.zone,
                position=excluded.position,
                first_time_today=excluded.first_time_today
            "#,
            params![
                user.id, user.name, user.display_name, user.join_count,
                user.last_join_at, user.enqueued_at, user.manual_order,
                zone_to_str(zone), position, user.first_time_today as i64
            ],
        )?;
        Ok(())
    }

    pub fn load_zone(&self, zone: Zone) -> Result<Vec<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id,name,display_name,join_count,last_join_at,enqueued_at,manual_order,first_time_today \
             FROM users WHERE zone=?1 ORDER BY position ASC"
        )?;
        let rows = stmt.query_map(params![zone_to_str(zone)], |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
                display_name: r.get(2)?,
                join_count: r.get(3)?,
                last_join_at: r.get(4)?,
                enqueued_at: r.get(5)?,
                manual_order: r.get(6)?,
                first_time_today: r.get::<_, i64>(7)? != 0,
            })
        })?;
        let mut v = Vec::new();
        for row in rows {
            v.push(row?);
        }
        Ok(v)
    }

    pub fn move_user(&self, user_id: &str, zone: Zone, position: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE users SET zone=?1, position=?2 WHERE id=?3",
            params![zone_to_str(zone), position, user_id],
        )?;
        if rows == 0 {
            return Err(AppError::Other(format!("user {user_id} not found")));
        }
        Ok(())
    }

    pub fn load_user(&self, user_id: &str) -> Result<Option<(User, Zone)>> {
        let conn = self.conn.lock().unwrap();
        let row = conn
            .query_row(
                "SELECT id,name,display_name,join_count,last_join_at,enqueued_at,manual_order,zone,first_time_today \
                 FROM users WHERE id=?1",
                params![user_id],
                |r| {
                    let zone: String = r.get(7)?;
                    Ok((
                        User {
                            id: r.get(0)?,
                            name: r.get(1)?,
                            display_name: r.get(2)?,
                            join_count: r.get(3)?,
                            last_join_at: r.get(4)?,
                            enqueued_at: r.get(5)?,
                            manual_order: r.get(6)?,
                            first_time_today: r.get::<_, i64>(8)? != 0,
                        },
                        zone,
                    ))
                },
            )
            .optional()?;
        Ok(row.map(|(u, z)| {
            let zone = match z.as_str() {
                "playing" => Zone::Playing,
                "waiting" => Zone::Waiting,
                "trash" => Zone::Trash,
                _ => Zone::History,
            };
            (u, zone)
        }))
    }

    pub fn delete_user(&self, user_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM users WHERE id=?1", params![user_id])?;
        Ok(())
    }

    pub fn reset_counts(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET join_count=0, last_join_at=NULL, manual_order=NULL, first_time_today=1",
            [],
        )?;
        Ok(())
    }

    pub fn trim_trash(&self, cap: usize) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM users WHERE id IN (
                SELECT id FROM users WHERE zone='trash'
                ORDER BY position DESC
                LIMIT max(0, (SELECT COUNT(*) FROM users WHERE zone='trash') - ?1)
             )",
            params![cap as i64],
        )?;
        Ok(())
    }

    pub fn persist_visible_zones(
        &self,
        playing: &[User],
        waiting: &[User],
        trash: &[User],
        trash_cap: usize,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for (i, user) in playing.iter().enumerate() {
            upsert_user_tx(&tx, user, Zone::Playing, i as i64)?;
        }
        for (i, user) in waiting.iter().enumerate() {
            upsert_user_tx(&tx, user, Zone::Waiting, i as i64)?;
        }
        for (i, user) in trash.iter().enumerate() {
            upsert_user_tx(&tx, user, Zone::Trash, i as i64)?;
        }
        tx.execute(
            "DELETE FROM users WHERE id IN (
                SELECT id FROM users WHERE zone='trash'
                ORDER BY position DESC
                LIMIT max(0, (SELECT COUNT(*) FROM users WHERE zone='trash') - ?1)
             )",
            params![trash_cap as i64],
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn zone_to_str(z: Zone) -> &'static str {
    match z {
        Zone::Playing => "playing",
        Zone::Waiting => "waiting",
        Zone::Trash => "trash",
        Zone::History => "history",
    }
}

fn upsert_user_tx(
    tx: &rusqlite::Transaction<'_>,
    user: &User,
    zone: Zone,
    position: i64,
) -> Result<()> {
    tx.execute(
        r#"
        INSERT INTO users (id,name,display_name,join_count,last_join_at,enqueued_at,manual_order,zone,position,first_time_today)
        VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
        ON CONFLICT(id) DO UPDATE SET
            name=excluded.name,
            display_name=excluded.display_name,
            join_count=excluded.join_count,
            last_join_at=excluded.last_join_at,
            enqueued_at=excluded.enqueued_at,
            manual_order=excluded.manual_order,
            zone=excluded.zone,
            position=excluded.position,
            first_time_today=excluded.first_time_today
        "#,
        params![
            user.id,
            user.name,
            user.display_name,
            user.join_count,
            user.last_join_at,
            user.enqueued_at,
            user.manual_order,
            zone_to_str(zone),
            position,
            user.first_time_today as i64
        ],
    )?;
    Ok(())
}
