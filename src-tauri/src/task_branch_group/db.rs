use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use crate::cookie_bridge::db::Db;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskBranchGroup {
    pub id: i64,
    pub tb_name: String,
    pub task_id: String,
    pub branch_name: String,
    pub group_type: i32,
    pub create_time: String,
    pub modify_time: String,
    pub rec_status: i32,
    pub create_by: i64,
    pub modify_by: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskBranchGroup {
    pub tb_name: String,
    pub task_id: String,
    pub branch_name: String,
    pub group_type: i32,
    pub create_by: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskBranchGroup {
    pub tb_name: Option<String>,
    pub task_id: Option<String>,
    pub branch_name: Option<String>,
    pub group_type: Option<i32>,
    pub rec_status: Option<i32>,
    pub modify_by: Option<i64>,
}

impl Db {
    pub fn task_branch_group_create(&self, item: &CreateTaskBranchGroup) -> Result<i64> {
        let conn = self.connection();
        conn.execute(
            "INSERT INTO task_branch_group (tb_name, task_id, branch_name, group_type, create_by, modify_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.tb_name,
                item.task_id,
                item.branch_name,
                item.group_type,
                item.create_by,
                item.create_by
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn task_branch_group_list(
        &self,
        keyword: Option<&str>,
        task_id: Option<&str>,
        branch_name: Option<&str>,
    ) -> Result<Vec<TaskBranchGroup>> {
        let mut where_parts: Vec<String> = vec!["rec_status = 1".to_string()];
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        let kw = keyword.map(|s| s.trim()).filter(|s| !s.is_empty());
        if let Some(kw) = kw {
            let pattern = format!("%{}%", kw);
            where_parts.push("(tb_name LIKE ? OR task_id LIKE ?)".to_string());
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern));
        }
        if let Some(tid) = task_id {
            where_parts.push("task_id = ?".to_string());
            params_vec.push(Box::new(tid.to_string()));
        }
        if let Some(bn) = branch_name {
            where_parts.push("branch_name = ?".to_string());
            params_vec.push(Box::new(bn.to_string()));
        }

        let sql = format!(
            "SELECT id, tb_name, task_id, branch_name, group_type, create_time, modify_time, rec_status, create_by, modify_by
             FROM task_branch_group WHERE {} ORDER BY id DESC",
            where_parts.join(" AND ")
        );

        let conn = self.connection();
        let mut stmt = conn.prepare(&sql)?;
        let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(params_ref.as_slice(), Self::row_to_task_branch_group)?;
        rows.collect::<Result<Vec<_>>>()
    }

    pub fn task_branch_group_get(&self, id: i64) -> Result<Option<TaskBranchGroup>> {
        let conn = self.connection();
        let mut stmt = conn.prepare(
            "SELECT id, tb_name, task_id, branch_name, group_type, create_time, modify_time, rec_status, create_by, modify_by
             FROM task_branch_group WHERE id = ?1 AND rec_status = 1"
        )?;
        let mut rows = stmt.query_map([id], Self::row_to_task_branch_group)?;
        Ok(rows.next().transpose()?)
    }

    pub fn task_branch_group_update(&self, id: i64, item: &UpdateTaskBranchGroup) -> Result<bool> {
        let conn = self.connection();
        let mut fields: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(tb_name) = &item.tb_name {
            fields.push("tb_name = ?".to_string());
            params_vec.push(Box::new(tb_name.clone()));
        }
        if let Some(task_id) = &item.task_id {
            fields.push("task_id = ?".to_string());
            params_vec.push(Box::new(task_id.clone()));
        }
        if let Some(branch_name) = &item.branch_name {
            fields.push("branch_name = ?".to_string());
            params_vec.push(Box::new(branch_name.clone()));
        }
        if let Some(group_type) = item.group_type {
            fields.push("group_type = ?".to_string());
            params_vec.push(Box::new(group_type));
        }
        if let Some(rec_status) = item.rec_status {
            fields.push("rec_status = ?".to_string());
            params_vec.push(Box::new(rec_status));
        }
        if let Some(modify_by) = item.modify_by {
            fields.push("modify_by = ?".to_string());
            params_vec.push(Box::new(modify_by));
        }

        if fields.is_empty() {
            return Ok(false);
        }

        let sql = format!(
            "UPDATE task_branch_group SET {} WHERE id = ?",
            fields.join(", ")
        );
        params_vec.push(Box::new(id));
        let params_ref: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let affected = conn.execute(&sql, params_ref.as_slice())?;
        Ok(affected > 0)
    }

    pub fn task_branch_group_delete(&self, id: i64) -> Result<bool> {
        let conn = self.connection();
        let affected = conn.execute(
            "UPDATE task_branch_group SET rec_status = 0 WHERE id = ?1",
            [id],
        )?;
        Ok(affected > 0)
    }

    fn row_to_task_branch_group(row: &rusqlite::Row) -> Result<TaskBranchGroup> {
        Ok(TaskBranchGroup {
            id: row.get(0)?,
            tb_name: row.get(1)?,
            task_id: row.get(2)?,
            branch_name: row.get(3)?,
            group_type: row.get(4)?,
            create_time: row.get(5)?,
            modify_time: row.get(6)?,
            rec_status: row.get(7)?,
            create_by: row.get(8)?,
            modify_by: row.get(9)?,
        })
    }
}
