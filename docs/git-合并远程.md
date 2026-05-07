# Git：合并远程 main（简洁说明）

## 本次已执行的命令

在仓库根目录 `tauri-app` 下：

```bash
cd /Users/zhangheng/frontCode/Tauri_app/tauri-app

git status
git remote -v
git branch -vv

git pull origin main --no-rebase --no-edit
```

说明：`--no-rebase` 表示用**合并**把 `origin/main` 合进当前分支（适合已分叉、要保留合并提交的场景）；`--no-edit` 使用默认合并说明，不打开编辑器。

若 Git 提示必须先指定合并方式，可任选其一作为默认（只需配置一次）：

```bash
# 以后 pull 默认用合并（与上面 pull 一致）
git config pull.rebase false

# 或：以后 pull 默认用变基（线性历史，需熟悉 rebase）
git config pull.rebase true
```

## 日常可复用写法

```bash
git fetch origin
git merge origin/main --no-edit
```

或一步：

```bash
git pull origin main --no-rebase --no-edit
```

有冲突时：按提示改文件 → `git add` → `git commit`（完成合并提交）。

## 合并后可选自检

```bash
cargo check --manifest-path src-tauri/Cargo.toml
npm run build
```

（视本机网络与依赖而定。）
