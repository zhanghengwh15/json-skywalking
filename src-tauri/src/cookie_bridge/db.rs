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
    pub domain_name: String,
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
    pub domain_name: String,
    pub key: String,
    pub value: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Domain {
    pub id: i64,
    pub domain_name: String,
    pub urls: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDomain {
    pub domain_name: String,
    #[serde(default)]
    pub urls: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDomain {
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub urls: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSnapshot {
    pub domain: Option<Domain>,
    pub cookies: Vec<CookieItem>,
    pub local_storage: Vec<LocalStorageItem>,
}

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub(crate) fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn open(path: &Path) -> Result<Self> {
        log::info!("[CookieBridge DB] 打开数据库: {:?}", path);
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;",
        )?;

        // 检查是否需要迁移旧 schema（旧表含 domain TEXT 列）
        let needs_migration = Self::detect_old_schema(&conn)?;
        if needs_migration {
            log::info!("[CookieBridge DB] 检测到旧 schema，执行迁移（丢弃历史数据）");
            conn.execute_batch(
                "DROP TABLE IF EXISTS cookies;
                 DROP TABLE IF EXISTS local_storage;",
            )?;
        }

        let schema = r#"
            CREATE TABLE IF NOT EXISTS domains (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain_name TEXT NOT NULL UNIQUE,
                urls TEXT,
                description TEXT,
                created_at DATETIME NOT NULL DEFAULT (datetime('now')),
                updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS cookies (
                domain_id INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                value TEXT NOT NULL,
                expires INTEGER NOT NULL DEFAULT 0,
                secure INTEGER NOT NULL DEFAULT 0,
                http_only INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (domain_id, name, path)
            );
            CREATE TABLE IF NOT EXISTS local_storage (
                domain_id INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (domain_id, key)
            );
            CREATE TABLE IF NOT EXISTS task_branch_group (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tb_name TEXT NOT NULL DEFAULT '',
                task_id TEXT NOT NULL DEFAULT '',
                branch_name TEXT NOT NULL DEFAULT '',
                group_type INTEGER NOT NULL DEFAULT 1,
                create_time DATETIME NOT NULL DEFAULT (datetime('now')),
                modify_time DATETIME NOT NULL DEFAULT (datetime('now')),
                rec_status INTEGER NOT NULL DEFAULT 1,
                create_by INTEGER NOT NULL DEFAULT 0,
                modify_by INTEGER NOT NULL DEFAULT 0
            );
            CREATE TRIGGER IF NOT EXISTS trg_task_branch_group_modify_time
            AFTER UPDATE ON task_branch_group
            BEGIN
                UPDATE task_branch_group SET modify_time = datetime('now') WHERE id = NEW.id;
            END;
        "#;
        conn.execute_batch(schema)?;

        Ok(Db {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn detect_old_schema(conn: &Connection) -> Result<bool> {
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM pragma_table_info('cookies') WHERE name = 'domain'",
        )?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count > 0)
    }

    fn get_or_create_domain_id(&self, domain_name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO domains (domain_name) VALUES (?1)",
            [domain_name],
        )?;
        let mut stmt = conn.prepare("SELECT id FROM domains WHERE domain_name = ?1")?;
        let id: i64 = stmt.query_row([domain_name], |row| row.get(0))?;
        Ok(id)
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

        let domain_id = self.get_or_create_domain_id(&payload.domain)?;

        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let cookie_count = if let Some(cookies) = &payload.cookies {
            let deleted = tx.execute("DELETE FROM cookies WHERE domain_id=?1", [domain_id])?;
            if is_debug_mode() {
                log::info!("[CookieBridge DB] 删除旧 cookies: domain_id={}, 删除条数={}", domain_id, deleted);
            }
            let mut count = 0;
            for c in cookies {
                let expires_i64 = c.expires.unwrap_or(0.0) as i64;
                if is_debug_mode() {
                    log::info!(
                        "[CookieBridge DB] 写入 cookie: domain_id={}, name={}, path={}, expires={:?}, secure={}, http_only={}",
                        domain_id, c.name, c.path, c.expires, c.secure as i32, c.http_only as i32
                    );
                }
                tx.execute(
                    "INSERT INTO cookies (domain_id, name, path, value, expires, secure, http_only, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                     ON CONFLICT(domain_id, name, path) DO UPDATE SET
                        value=excluded.value,
                        expires=excluded.expires,
                        secure=excluded.secure,
                        http_only=excluded.http_only,
                        updated_at=excluded.updated_at",
                    params![
                        domain_id, c.name, c.path, c.value, expires_i64, c.secure, c.http_only,
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
            let deleted = tx.execute("DELETE FROM local_storage WHERE domain_id=?1", [domain_id])?;
            if is_debug_mode() {
                log::info!("[CookieBridge DB] 删除旧 local_storage: domain_id={}, 删除条数={}", domain_id, deleted);
            }
            let mut count = 0;
            for item in ls {
                if is_debug_mode() {
                    log::info!(
                        "[CookieBridge DB] 写入 local_storage: domain_id={}, key={}, value_len={}",
                        domain_id, item.key, item.value.len()
                    );
                }
                tx.execute(
                    "INSERT INTO local_storage (domain_id, key, value, updated_at)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(domain_id, key) DO UPDATE SET
                        value=excluded.value,
                        updated_at=excluded.updated_at",
                    params![domain_id, item.key, item.value, effective_ts],
                )?;
                count += 1;
            }
            count
        } else {
            0
        };

        tx.commit()?;
        log::info!(
            "[CookieBridge DB] push 完成: domain={}, domain_id={}, 写入 cookies={}, 写入 local_storage={}",
            payload.domain, domain_id, cookie_count, ls_count
        );
        Ok((cookie_count, ls_count))
    }

    pub fn list_domains(&self) -> Result<Vec<Domain>> {
        log::info!("[CookieBridge DB] list_domains 查询开始");
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT d.id, d.domain_name, d.urls, d.description, d.created_at, d.updated_at
             FROM domains d
             INNER JOIN (
                 SELECT domain_id, MAX(updated_at) as max_ts FROM cookies GROUP BY domain_id
                 UNION
                 SELECT domain_id, MAX(updated_at) as max_ts FROM local_storage GROUP BY domain_id
             ) t ON d.id = t.domain_id
             ORDER BY t.max_ts DESC",
        )?;
        let domains: Vec<Domain> = stmt
            .query_map([], |row| {
                Ok(Domain {
                    id: row.get(0)?,
                    domain_name: row.get(1)?,
                    urls: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        log::info!("[CookieBridge DB] list_domains 查询完成: 共 {} 个域名", domains.len());
        Ok(domains)
    }

    pub fn get_domain(&self, domain_name: &str) -> Result<DomainSnapshot> {
        log::info!("[CookieBridge DB] get_domain 查询开始: domain_name={}", domain_name);
        let conn = self.conn.lock().unwrap();

        let domain: Option<Domain> = {
            let mut stmt = conn.prepare(
                "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains WHERE domain_name = ?1",
            )?;
            let mut rows = stmt.query_map([domain_name], |row| {
                Ok(Domain {
                    id: row.get(0)?,
                    domain_name: row.get(1)?,
                    urls: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;
            rows.next().transpose()?
        };

        let mut cookie_stmt = conn.prepare(
            "SELECT d.domain_name, c.name, c.path, c.value, c.expires, c.secure, c.http_only, c.updated_at
             FROM cookies c
             JOIN domains d ON c.domain_id = d.id
             WHERE d.domain_name = ?1",
        )?;
        let cookies = cookie_stmt
            .query_map([domain_name], |row| {
                Ok(CookieItem {
                    domain_name: row.get(0)?,
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
            "SELECT d.domain_name, l.key, l.value, l.updated_at
             FROM local_storage l
             JOIN domains d ON l.domain_id = d.id
             WHERE d.domain_name = ?1",
        )?;
        let local_storage = ls_stmt
            .query_map([domain_name], |row| {
                Ok(LocalStorageItem {
                    domain_name: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        log::info!(
            "[CookieBridge DB] get_domain 查询完成: domain_name={}, cookies={}, local_storage={}",
            domain_name,
            cookies.len(),
            local_storage.len()
        );
        Ok(DomainSnapshot {
            domain,
            cookies,
            local_storage,
        })
    }

    pub fn delete_domain(&self, domain_name: &str) -> Result<(usize, usize)> {
        log::info!("[CookieBridge DB] 删除域名: domain_name={}", domain_name);
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let domain_id: Option<i64> = {
            let mut stmt = tx.prepare("SELECT id FROM domains WHERE domain_name = ?1")?;
            let mut rows = stmt.query_map([domain_name], |row| row.get::<_, i64>(0))?;
            rows.next().transpose()?
        };

        let (cookies_deleted, ls_deleted) = if let Some(id) = domain_id {
            let cd = tx.execute("DELETE FROM cookies WHERE domain_id = ?1", [id])?;
            let ld = tx.execute("DELETE FROM local_storage WHERE domain_id = ?1", [id])?;
            tx.execute("DELETE FROM domains WHERE id = ?1", [id])?;
            (cd, ld)
        } else {
            (0, 0)
        };

        tx.commit()?;
        log::info!(
            "[CookieBridge DB] 删除完成: domain_name={}, cookies={}, local_storage={}",
            domain_name, cookies_deleted, ls_deleted
        );
        Ok((cookies_deleted, ls_deleted))
    }

    // Domain CRUD

    pub fn domain_create(&self, item: &CreateDomain) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO domains (domain_name, urls, description) VALUES (?1, ?2, ?3)",
            params![item.domain_name, item.urls, item.description],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn domain_list(&self) -> Result<Vec<Domain>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains ORDER BY id DESC",
        )?;
        let domains: Vec<Domain> = stmt
            .query_map([], |row| {
                Ok(Domain {
                    id: row.get(0)?,
                    domain_name: row.get(1)?,
                    urls: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(domains)
    }

    pub fn domain_get(&self, id: i64) -> Result<Option<Domain>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Domain {
                id: row.get(0)?,
                domain_name: row.get(1)?,
                urls: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn domain_get_by_name(&self, domain_name: &str) -> Result<Option<Domain>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains WHERE domain_name = ?1",
        )?;
        let mut rows = stmt.query_map([domain_name], |row| {
            Ok(Domain {
                id: row.get(0)?,
                domain_name: row.get(1)?,
                urls: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn domain_update(&self, id: i64, item: &UpdateDomain) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut fields: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(domain_name) = &item.domain_name {
            fields.push("domain_name = ?".to_string());
            params_vec.push(Box::new(domain_name.clone()));
        }
        if let Some(urls) = &item.urls {
            fields.push("urls = ?".to_string());
            params_vec.push(Box::new(urls.clone()));
        }
        if let Some(description) = &item.description {
            fields.push("description = ?".to_string());
            params_vec.push(Box::new(description.clone()));
        }

        if fields.is_empty() {
            return Ok(false);
        }

        fields.push("updated_at = datetime('now')".to_string());

        let sql = format!("UPDATE domains SET {} WHERE id = ?", fields.join(", "));
        params_vec.push(Box::new(id));
        let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let affected = conn.execute(&sql, params_ref.as_slice())?;
        Ok(affected > 0)
    }

    pub fn domain_delete(&self, id: i64) -> Result<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM cookies WHERE domain_id = ?1", [id])?;
        tx.execute("DELETE FROM local_storage WHERE domain_id = ?1", [id])?;
        let affected = tx.execute("DELETE FROM domains WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(affected > 0)
    }

    pub fn domain_match_url(&self, url: &str) -> Result<Option<(Domain, Vec<CookieItem>, Vec<LocalStorageItem>)>> {
        // 解析 host
        let host = if let Some(stripped) = url.strip_prefix("https://") {
            stripped.split('/').next().unwrap_or(stripped).to_string()
        } else if let Some(stripped) = url.strip_prefix("http://") {
            stripped.split('/').next().unwrap_or(stripped).to_string()
        } else {
            url.split('/').next().unwrap_or(url).to_string()
        };

        log::info!("[CookieBridge DB] domain_match_url: url={}, host={}", url, host);

        // 精确匹配 domain_name
        if let Some(domain) = self.domain_get_by_name(&host)? {
            let snapshot = self.get_domain_snapshot_by_id(domain.id)?;
            return Ok(Some((domain, snapshot.cookies, snapshot.local_storage)));
        }

        // 回退到 urls 字段匹配
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains",
        )?;
        let domains: Vec<Domain> = stmt
            .query_map([], |row| {
                Ok(Domain {
                    id: row.get(0)?,
                    domain_name: row.get(1)?,
                    urls: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        for domain in domains {
            if let Some(urls_json) = &domain.urls {
                let patterns: Vec<String> = serde_json::from_str(urls_json).unwrap_or_default();
                for pattern in patterns {
                    let pattern_host = if let Some(stripped) = pattern.strip_prefix("https://") {
                        stripped.split('/').next().unwrap_or(stripped).to_string()
                    } else if let Some(stripped) = pattern.strip_prefix("http://") {
                        stripped.split('/').next().unwrap_or(stripped).to_string()
                    } else {
                        pattern.split('/').next().unwrap_or(&pattern).to_string()
                    };
                    // 去掉 *. 前缀进行后缀匹配
                    let pattern_host_clean = pattern_host.strip_prefix("*.").unwrap_or(&pattern_host);
                    if host == pattern_host || host.ends_with(pattern_host_clean) {
                        let snapshot = self.get_domain_snapshot_by_id(domain.id)?;
                        return Ok(Some((domain, snapshot.cookies, snapshot.local_storage)));
                    }
                }
            }
        }

        Ok(None)
    }

    fn get_domain_snapshot_by_id(&self, domain_id: i64) -> Result<DomainSnapshot> {
        let conn = self.conn.lock().unwrap();

        let domain: Option<Domain> = {
            let mut stmt = conn.prepare(
                "SELECT id, domain_name, urls, description, created_at, updated_at FROM domains WHERE id = ?1",
            )?;
            let mut rows = stmt.query_map([domain_id], |row| {
                Ok(Domain {
                    id: row.get(0)?,
                    domain_name: row.get(1)?,
                    urls: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })?;
            rows.next().transpose()?
        };

        let mut cookie_stmt = conn.prepare(
            "SELECT d.domain_name, c.name, c.path, c.value, c.expires, c.secure, c.http_only, c.updated_at
             FROM cookies c
             JOIN domains d ON c.domain_id = d.id
             WHERE c.domain_id = ?1",
        )?;
        let cookies = cookie_stmt
            .query_map([domain_id], |row| {
                Ok(CookieItem {
                    domain_name: row.get(0)?,
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
            "SELECT d.domain_name, l.key, l.value, l.updated_at
             FROM local_storage l
             JOIN domains d ON l.domain_id = d.id
             WHERE l.domain_id = ?1",
        )?;
        let local_storage = ls_stmt
            .query_map([domain_id], |row| {
                Ok(LocalStorageItem {
                    domain_name: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(DomainSnapshot {
            domain,
            cookies,
            local_storage,
        })
    }
}
