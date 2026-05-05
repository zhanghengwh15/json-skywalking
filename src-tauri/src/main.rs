// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dev-tools")]
#[command(about = "Dev Tools - 开发工具箱")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 任务-分支-工程分组管理
    #[command(name = "task-branch-group")]
    TaskBranchGroup {
        #[command(subcommand)]
        action: TaskBranchGroupAction,
    },
}

#[derive(Subcommand)]
enum TaskBranchGroupAction {
    /// 列表查询
    List {
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long)]
        branch_name: Option<String>,
    },
    /// 创建记录
    Create {
        #[arg(long)]
        tb_name: String,
        #[arg(long)]
        task_id: String,
        #[arg(long)]
        branch_name: String,
        #[arg(long, value_parser = clap::value_parser!(i32))]
        group_type: i32,
        #[arg(long, default_value = "0")]
        create_by: i64,
    },
    /// 查询单条
    Get {
        id: i64,
    },
    /// 更新记录
    Update {
        id: i64,
        #[arg(long)]
        tb_name: Option<String>,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long)]
        branch_name: Option<String>,
        #[arg(long)]
        group_type: Option<i32>,
        #[arg(long)]
        rec_status: Option<i32>,
        #[arg(long)]
        modify_by: Option<i64>,
    },
    /// 软删除
    Delete {
        id: i64,
    },
}

fn default_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .or_else(|_| std::env::var("APPDATA"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(local_app_data).join("com.tauri.devtools")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library/Application Support/com.tauri.devtools")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".local/share")
            });
        data_home.join("com.tauri.devtools")
    }
}

fn run_cli(action: TaskBranchGroupAction) {
    let app_data_dir = default_app_data_dir();
    std::fs::create_dir_all(&app_data_dir).expect("create app data dir failed");
    let db_path = app_data_dir.join("data.db");

    let db = tauri_app_lib::cookie_bridge::db::Db::open(&db_path).expect("open db failed");

    match action {
        TaskBranchGroupAction::List { task_id, branch_name } => {
            let items = db
                .task_branch_group_list(None, task_id.as_deref(), branch_name.as_deref())
                .expect("list failed");
            println!("{}", serde_json::to_string_pretty(&items).unwrap());
        }
        TaskBranchGroupAction::Create {
            tb_name,
            task_id,
            branch_name,
            group_type,
            create_by,
        } => {
            let item = tauri_app_lib::task_branch_group::db::CreateTaskBranchGroup {
                tb_name,
                task_id,
                branch_name,
                group_type,
                create_by,
            };
            let id = db.task_branch_group_create(&item).expect("create failed");
            let created = db.task_branch_group_get(id).expect("get failed");
            println!("{}", serde_json::to_string_pretty(&created).unwrap());
        }
        TaskBranchGroupAction::Get { id } => {
            let item = db.task_branch_group_get(id).expect("get failed");
            println!("{}", serde_json::to_string_pretty(&item).unwrap());
        }
        TaskBranchGroupAction::Update {
            id,
            tb_name,
            task_id,
            branch_name,
            group_type,
            rec_status,
            modify_by,
        } => {
            let item = tauri_app_lib::task_branch_group::db::UpdateTaskBranchGroup {
                tb_name,
                task_id,
                branch_name,
                group_type,
                rec_status,
                modify_by,
            };
            let updated = db.task_branch_group_update(id, &item).expect("update failed");
            if updated {
                let result = db.task_branch_group_get(id).expect("get failed");
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                eprintln!("not found or no changes");
                std::process::exit(1);
            }
        }
        TaskBranchGroupAction::Delete { id } => {
            let deleted = db.task_branch_group_delete(id).expect("delete failed");
            if deleted {
                println!("{{\"deleted\":true}}");
            } else {
                eprintln!("not found");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Some(Commands::TaskBranchGroup { action }) = cli.command {
        run_cli(action);
        return;
    }

    tauri_app_lib::run();
}
