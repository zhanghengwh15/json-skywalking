use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
pub struct PushCookie {
    pub domain: String,
    pub name: String,
    pub value: String,
    pub path: String,
    pub expires: i64,
    pub secure: i32,
    pub http_only: i32,
}

#[derive(Debug, Deserialize)]
pub struct PushLocalStorage {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct PushPayload {
    pub domain: String,
    pub cookies: Option<Vec<PushCookie>>,
    pub local_storage: Option<Vec<PushLocalStorage>>,
    pub ts: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieItem {
    pub domain: String,
    pub name: String,
    pub path: String,
    pub value: String,
    pub expires: i64,
    pub secure: i32,
    pub http_only: i32,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStorageItem {
    pub domain: String,
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSnapshot {
    pub cookies: Vec<CookieItem>,
    pub local_storage: Vec<LocalStorageItem>,
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )?;

        let schema = r#"
            CREATE TABLE IF NOT EXISTS cookies (
                domain TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                value TEXT NOT NULL,
                expires INTEGER NOT NULL DEFAULT 0,
                secure INTEGER NOT NULL DEFAULT 0,
                http_only INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (domain, name, path)
            );
            CREATE TABLE IF NOT EXISTS local_storage (
                domain TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (domain, key)
            );
        "#;
        conn.execute_batch(schema)?;

        Ok(Db {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn push(&self, payload: &PushPayload) -> Result<(usize, usize)> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let cookie_count = if let Some(cookies) = &payload.cookies {
            tx.execute("DELETE FROM cookies WHERE domain=?1", [&payload.domain])?;
            let mut count = 0;
            for c in cookies {
                tx.execute(
                    "INSERT INTO cookies (domain, name, path, value, expires, secure, http_only, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(domain, name, path) DO UPDATE SET
                        value=excluded.value,
                        expires=excluded.expires,
                        secure=excluded.secure,
                        http_only=excluded.http_only,
                        updated_at=excluded.updated_at",
                    params![
                        c.domain, c.name, c.path, c.value, c.expires, c.secure, c.http_only,
                        payload.ts
                    ],
                )?;
                count += 1;
            }
            count
        } else {
            0
        };

        let ls_count = if let Some(ls) = &payload.local_storage {
            tx.execute("DELETE FROM local_storage WHERE domain=?1", [&payload.domain])?;
            let mut count = 0;
            for item in ls {
                tx.execute(
                    "INSERT INTO local_storage (domain, key, value, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(domain, key) DO UPDATE SET
                        value=excluded.value,
                        updated_at=excluded.updated_at",
                    params![payload.domain, item.key, item.value, payload.ts],
                )?;
                count += 1;
            }
            count
        } else {
            0
        };

        tx.commit()?;
        Ok((cookie_count, ls_count))
    }

    pub fn list_domains(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain FROM (
                SELECT domain, MAX(updated_at) as max_ts FROM cookies GROUP BY domain
                UNION
                SELECT domain, MAX(updated_at) as max_ts FROM local_storage GROUP BY domain
            ) GROUP BY domain ORDER BY MAX(max_ts) DESC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect()
    }

    pub fn get_domain(&self, domain: &str) -> Result<DomainSnapshot> {
        let conn = self.conn.lock().unwrap();

        let mut cookie_stmt = conn.prepare(
            "SELECT domain, name, path, value, expires, secure, http_only, updated_at
             FROM cookies WHERE domain = ?1",
        )?;
        let cookies = cookie_stmt
            .query_map([domain], |row| {
                Ok(CookieItem {
                    domain: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    value: row.get(3)?,
                    expires: row.get(4)?,
                    secure: row.get(5)?,
                    http_only: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        let mut ls_stmt = conn.prepare(
            "SELECT domain, key, value, updated_at
             FROM local_storage WHERE domain = ?1",
        )?;
        let local_storage = ls_stmt
            .query_map([domain], |row| {
                Ok(LocalStorageItem {
                    domain: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(DomainSnapshot {
            cookies,
            local_storage,
        })
    }
}
