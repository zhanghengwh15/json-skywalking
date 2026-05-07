# dev-tools 命令行：为何找不到命令，以及 macOS / Windows 怎么处理

本项目的**主程序与命令行入口是同一个可执行文件**，包名 **`dev-tools`**（见 `src-tauri/Cargo.toml`）。  
无论来源是 **DMG / Windows 安装包**，还是 **`cargo build`**，系统都**不会自动**在终端里注册 `dev-tools` 这个名字，所以直接输入常出现：

```text
zsh: command not found: dev-tools
```

这不是「没装 CLI」，而是 **CLI 没有出现在 PATH 里**。下面按使用场景说明怎么处理。

---

## 0. 已通过 DMG（macOS）或安装程序（Windows）安装 — 最常见

安装包会把应用装到「应用程序」或「Program Files」里，**不会**像 Homebrew 那样往 `/usr/local/bin` 或用户 PATH 里放同名命令，因此终端里仍要：**写全路径**，或 **自己加 PATH / 做符号链接**。

### 0.1 macOS：从 .app 包里找到真正的 `dev-tools`

1. 打开 **访达 → 应用程序**，找到从 DMG 安装的应用（名称以你机器上为准，例如带 **Dev Tools**、**dev-tools** 等字样）。
2. **右键该应用 → 显示包内容**。
3. 进入 **`Contents/MacOS/`** 目录。
4. 里面通常有一个**无扩展名**的可执行文件，名称一般为 **`dev-tools`**（与工程里 Rust 默认 binary 名一致）。这就是带 `task-branch-group` 子命令的程序本体。

路径形如（**请把中间的 .app 名称改成你本机实际名称**）：

```text
/Applications/<你的应用名>.app/Contents/MacOS/dev-tools
```

若应用名含空格，在终端里务必加引号，例如：

```bash
"/Applications/Dev Tools.app/Contents/MacOS/dev-tools" task-branch-group list
```

**可选：以后想直接敲 `dev-tools`**（一次配置，升级 DMG 后若 .app 路径不变可继续用）：

```bash
sudo ln -sf "/Applications/<你的应用名>.app/Contents/MacOS/dev-tools" /usr/local/bin/dev-tools
```

或把上面**完整路径**写进 `~/.zshrc` 里的 **alias**（不改系统目录）：

```bash
alias dev-tools='"/Applications/<你的应用名>.app/Contents/MacOS/dev-tools"'
```

执行 `source ~/.zshrc` 后，即可使用：

```bash
dev-tools task-branch-group list
```

### 0.2 Windows：从安装目录找到 `dev-tools.exe`

NSIS / MSI 一般会把程序放在 **`C:\Program Files\`** 下某个以应用名命名的文件夹里（本仓库 `tauri.conf.json` 里 NSIS 为 `perMachine`，多为系统级安装目录）。

1. **开始菜单**里找到「Dev Tools」或类似名称的快捷方式，**右键 → 打开文件所在位置**（可能先进快捷方式文件夹，再对快捷方式右键 → 属性，查看「目标」里的 `dev-tools.exe` 全路径）。
2. 确认同目录下有 **`dev-tools.exe`**。

在 **PowerShell** 或 **cmd** 中用**英文引号**包住带空格的路径，例如：

```powershell
& "C:\Program Files\Dev Tools\dev-tools.exe" task-branch-group list
```

**可选：加入用户 PATH**（环境变量 → 用户变量 `Path` → 新建 → 填入 **`dev-tools.exe` 所在文件夹**，例如 `C:\Program Files\Dev Tools`，不要填到 exe 本身）。保存后**新开**终端，再执行：

```powershell
dev-tools task-branch-group list
```

> **说明**：具体文件夹名以安装器为准；若与文档示例不一致，以快捷方式「目标」里的路径为准。

### 0.3 与「从源码编译」的关系

安装包里的可执行文件与 `cargo build` 打出来的是**同一套入口逻辑**（先解析 CLI 子命令，没有子命令再启动窗口）。  
已用 DMG/安装包的用户**不必**再装一份 `cargo build`，只要按上面方式**指向已安装的那个二进制**即可。

---

## 1. 从源码构建时，二进制在哪（开发 / CI）

在仓库根目录：

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

| 平台 | 调试版 | 发布版（加 `--release`） |
|------|--------|---------------------------|
| macOS / Linux | `src-tauri/target/debug/dev-tools` | `src-tauri/target/release/dev-tools` |
| Windows | `src-tauri\target\debug\dev-tools.exe` | `src-tauri\target\release\dev-tools.exe` |

---

## 2. 不修改 PATH：本机开发常用

### 2.1 `cargo run`（仅在已克隆仓库、装好 Rust 时）

在**仓库根**执行，注意 `--` 后面才是程序参数：

```bash
cargo run --manifest-path src-tauri/Cargo.toml -- task-branch-group list
```

Windows（PowerShell / cmd）同样写法。

### 2.2 脚本里写相对路径（仅开发机）

```bash
./src-tauri/target/release/dev-tools task-branch-group list
```

```powershell
.\src-tauri\target\release\dev-tools.exe task-branch-group list
```

---

## 3. 长期可用：把 `dev-tools` 放进 PATH（可选）

### 3.1 `cargo install --path src-tauri`（面向开发者）

会把另一份 `dev-tools` 装到 `~/.cargo/bin` 或 `%USERPROFILE%\.cargo\bin`。若与 **DMG 安装包并存**，注意 PATH 里**谁先谁后**会决定终端里执行的是哪一份。

### 3.2 macOS：`/usr/local/bin` 或 `~/bin`

除 **0.1** 的符号链接外，也可复制 `target/release/dev-tools` 到 `~/bin` 并把 `~/bin` 加入 `PATH`（见旧版文档思路即可）。

### 3.3 Windows：自定义目录加入 Path

将 `dev-tools.exe` 复制到例如 `C:\Tools\dev-tools\`，再把该文件夹加入用户 `Path`（不要只加 exe 文件路径）。

---

## 4. 相关文档

- 子命令与 HTTP：[`task-branch-group-对接文档.md`](task-branch-group-对接文档.md)
