use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::is_debug_mode;

fn deserialize_bool_or_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::Number(n) => {
            Ok(n.as_i64().map_or(false, |v| v != 0))
        }
        _ => Err(serde::de::Error::custom("expected bool or int (0/1)")),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushCookie {
    pub domain: String,
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub path: String,
    #[serde(alias = "expirationDate", default)]
    pub expires: Option<f64>,
    #[serde(deserialize_with = "deserialize_bool_or_int")]
    pub secure: bool,
    #[serde(
        alias = "http_only",
        rename = "httpOnly",
        deserialize_with = "deserialize_bool_or_int"
    )]
    pub http_only: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushLocalStorage {
    pub key: String,
    pub value: String,
}

fn deserialize_local_storage<'de, D>(deserializer: D) -> Result<Option<Vec<PushLocalStorage>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    if is_debug_mode() {
        log::info!("[CookieBridge DB] deserialize_local_storage 收到: {:?}", value.as_ref().map(|v| v.to_string().len()));
    }
    match value {
        None => {
            if is_debug_mode() {
                log::info!("[CookieBridge DB] deserialize_local_storage: None");
            }
            Ok(None)
        }
        Some(serde_json::Value::Array(arr)) => {
            if is_debug_mode() {
                log::info!("[CookieBridge DB] deserialize_local_storage: 数组格式, len={}", arr.len());
            }
            let items: Vec<PushLocalStorage> = arr
                .into_iter()
                .map(|v| serde_json::from_value(v).map_err(serde::de::Error::custom))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(items))
        }
        Some(serde_json::Value::Object(map)) => {
            if is_debug_mode() {
                log::info!("[CookieBridge DB] deserialize_local_storage: 对象格式, keys={}", map.len());
            }
            let items: Vec<PushLocalStorage> = map
                .into_iter()
                .map(|(k, v)| {
                    let val_str = match v {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    };
                    PushLocalStorage {
                        key: k,
                        value: val_str,
                    }
                })
                .collect();
            Ok(Some(items))
        }
        Some(other) => {
            log::error!("[CookieBridge DB] deserialize_local_storage: 未知格式: {:?}", other);
            Err(serde::de::Error::custom(
                "local_storage must be an array or object",
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushPayload {
    pub domain: String,
    pub cookies: Option<Vec<PushCookie>>,
    #[serde(default, deserialize_with = "deserialize_local_storage", alias = "local_storage")]
    pub local_storage: Option<Vec<PushLocalStorage>>,
    #[serde(default)]
    pub ts: Option<i64>,
}

fn current_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieItem {
    pub domain: String,
    pub name: String,
    pub path: String,
    pub value: String,
    pub expires: i64,
    pub secure: bool,
    pub http_only: bool,
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
        log::info!("[CookieBridge DB] 打开数据库: {:?}", path);
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
        let effective_ts = payload.ts.unwrap_or_else(current_millis);
        log::info!(
            "[CookieBridge DB] push 开始: domain={}, cookies={}, local_storage={}, ts={}{}",
            payload.domain,
            payload.cookies.as_ref().map_or(0, |v| v.len()),
            payload.local_storage.as_ref().map_or(0, |v| v.len()),
            effective_ts,
            if payload.ts.is_none() { " (服务端补默认值)" } else { "" }
        );

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let cookie_count = if let Some(cookies) = &payload.cookies {
            let deleted = tx.execute("DELETE FROM cookies WHERE domain=?1", [&payload.domain])?;
            if is_debug_mode() {
                log::info!("[CookieBridge DB] 删除旧 cookies: domain={}, 删除条数={}", payload.domain, deleted);
            }
            let mut count = 0;
            for c in cookies {
                let expires_i64 = c.expires.unwrap_or(0.0) as i64;
                if is_debug_mode() {
                    log::info!(
                        "[CookieBridge DB] 写入 cookie: domain={}, name={}, path={}, expires={:?}, secure={}, http_only={}",
                        c.domain, c.name, c.path, c.expires, c.secure as i32, c.http_only as i32
                    );
                }
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
                        c.domain, c.name, c.path, c.value, expires_i64, c.secure, c.http_only,
                        effective_ts
                    ],
                )?;
                count += 1;
            }
            count
        } else {
            0
        };

        let ls_count = if let Some(ls) = &payload.local_storage {
            let deleted = tx.execute("DELETE FROM local_storage WHERE domain=?1", [&payload.domain])?;
            if is_debug_mode() {
                log::info!("[CookieBridge DB] 删除旧 local_storage: domain={}, 删除条数={}", payload.domain, deleted);
            }
            let mut count = 0;
            for item in ls {
                if is_debug_mode() {
                    log::info!(
                        "[CookieBridge DB] 写入 local_storage: domain={}, key={}, value_len={}",
                        payload.domain, item.key, item.value.len()
                    );
                }
                tx.execute(
                    "INSERT INTO local_storage (domain, key, value, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(domain, key) DO UPDATE SET
                        value=excluded.value,
                        updated_at=excluded.updated_at",
                    params![payload.domain, item.key, item.value, effective_ts],
                )?;
                count += 1;
            }
            count
        } else {
            0
        };

        tx.commit()?;
        log::info!(
            "[CookieBridge DB] push 完成: domain={}, 写入 cookies={}, 写入 local_storage={}",
            payload.domain, cookie_count, ls_count
        );
        Ok((cookie_count, ls_count))
    }

    pub fn list_domains(&self) -> Result<Vec<String>> {
        log::info!("[CookieBridge DB] list_domains 查询开始");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain FROM (
                SELECT domain, MAX(updated_at) as max_ts FROM cookies GROUP BY domain
                UNION
                SELECT domain, MAX(updated_at) as max_ts FROM local_storage GROUP BY domain
            ) GROUP BY domain ORDER BY MAX(max_ts) DESC",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let domains: Vec<String> = rows.collect::<Result<Vec<_>>>()?;
        log::info!("[CookieBridge DB] list_domains 查询完成: 共 {} 个域名", domains.len());
        Ok(domains)
    }

    pub fn get_domain(&self, domain: &str) -> Result<DomainSnapshot> {
        log::info!("[CookieBridge DB] get_domain 查询开始: domain={}", domain);
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

        log::info!(
            "[CookieBridge DB] get_domain 查询完成: domain={}, cookies={}, local_storage={}",
            domain,
            cookies.len(),
            local_storage.len()
        );
        Ok(DomainSnapshot {
            cookies,
            local_storage,
        })
    }

    pub fn delete_domain(&self, domain: &str) -> Result<(usize, usize)> {
        log::info!("[CookieBridge DB] 删除域名: domain={}", domain);
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let cookies_deleted = tx.execute("DELETE FROM cookies WHERE domain = ?1", [domain])?;
        let ls_deleted = tx.execute("DELETE FROM local_storage WHERE domain = ?1", [domain])?;

        tx.commit()?;
        log::info!(
            "[CookieBridge DB] 删除完成: domain={}, cookies={}, local_storage={}",
            domain, cookies_deleted, ls_deleted
        );
        Ok((cookies_deleted, ls_deleted))
    }
}
