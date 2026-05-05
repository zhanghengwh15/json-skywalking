## 1. Backend Foundation

- [x] 1.1 Add `clap` dependency to `src-tauri/Cargo.toml`
- [x] 1.2 Create `src-tauri/src/task_branch_group/mod.rs` with module exports
- [x] 1.3 Create `src-tauri/src/task_branch_group/db.rs` with `TaskBranchGroup` model and CRUD methods
- [x] 1.4 Update `cookie_bridge/db.rs` `Db::open` to create `task_branch_group` table and trigger
- [x] 1.5 Create `src-tauri/src/task_branch_group/commands.rs` with Tauri invoke handlers
- [x] 1.6 Create `src-tauri/src/task_branch_group/http.rs` with Axum router for `/api/task-branch-groups`
- [x] 1.7 Update `cookie_bridge/http.rs` to nest the new router under `/api/task-branch-groups`
- [x] 1.8 Update `src-tauri/src/lib.rs` to register new module and expose commands
- [x] 1.9 Update `src-tauri/src/main.rs` to add CLI mode detection with `clap`

## 2. Frontend Implementation

- [x] 2.1 Create `src/views/TaskBranchGroup.vue` with CRUD UI (table, form modal, filters)
- [x] 2.2 Update `src/router/index.ts` to add `/task-branch-group` route
- [x] 2.3 Update `src/App.vue` to add "任务分支" menu item with icon `account_tree`

## 3. Verification

- [x] 3.1 Run `cargo check` in `src-tauri` to verify Rust compilation
- [x] 3.2 Run frontend dev server and verify page renders correctly
- [x] 3.3 Test CLI commands: `list`, `create`, `update`, `delete`, `get`
- [x] 3.4 Test HTTP API endpoints via curl
- [ ] 3.5 Test Tauri invoke commands via frontend UI
