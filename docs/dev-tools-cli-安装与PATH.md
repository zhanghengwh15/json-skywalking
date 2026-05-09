# dev-tools 命令行：双二进制架构与 PATH 配置

本项目目前提供 **两个二进制**，同源同套子命令，区别只在 Windows 子系统：

| 文件 | 子系统 | 适用场景 |
|------|--------|----------|
| `dev-tools.exe` | GUI（`windows_subsystem = "windows"`） | 双击启动主程序窗口；也可在 **cmd** / **git bash** 等会等待 GUI 程序结束的终端里跑子命令 |
| `dev-tools-cli.exe` | Console | **PowerShell** / cmd / 任意 shell 下的命令行调用，输出能被父进程稳定接管 |

之所以拆开：Windows 把可执行文件按子系统分两类，PowerShell 启动 GUI 子系统的程序时**不等待、也不接管 stdout**，导致 `dev-tools --help` 在 PowerShell 里看起来"没反应"。`dev-tools-cli.exe` 是 console 子系统，PowerShell 会正常等待并显示输出。

NSIS / MSI / DMG 安装包里**两份都装**，子命令完全一致，按习惯挑一个用即可。

---

## 0. 已通过 DMG（macOS）或 NSIS 安装包（Windows）安装 — 最常见

### 0.1 Windows：安装目录里有什么

NSIS `perMachine` 模式默认装到 `C:\Program Files\<应用名>\`（本项目里是 `C:\Program Files\dev-tools\` 或你指定的目录，比如 `D:\Program Files\dev-tools\`）。安装后该目录下会同时有：

```text
<install-dir>\dev-tools.exe          ← GUI 主程序
<install-dir>\dev-tools-cli.exe      ← 控制台 CLI（PowerShell 用这个）
```

> 验证：开始菜单里"Dev Tools"快捷方式 → 右键 → 打开文件所在位置；同目录下应该能看到 `dev-tools-cli.exe`。如果**只有** `dev-tools.exe`，说明用的是旧版安装包，重新跑 `npx tauri build --bundles nsis` 出新的 setup.exe，再重装。

### 0.2 PowerShell 用法

把安装目录加入用户 `Path`（环境变量 → 用户变量 `Path` → 新建 → 填入 `dev-tools.exe` 所在的**文件夹**，不要填 exe 本身），新开一个 PowerShell 终端：

```powershell
dev-tools-cli --help
dev-tools-cli task-branch-group list
dev-tools-cli domain list
```

✅ 输出能正常显示。

如果一定要敲 `dev-tools`（GUI 二进制）在 PowerShell 跑子命令，可以套一层 `cmd /c`，cmd 启动 GUI 程序会等待结束并接管控制台：

```powershell
cmd /c dev-tools --help
cmd /c dev-tools task-branch-group list
```

或者在 `$PROFILE` 里写一个别名函数：

```powershell
function dev-tools { cmd /c dev-tools.exe @args }
```

但更建议直接用 `dev-tools-cli`。

### 0.3 cmd / git bash 用法

cmd 和 git bash（mintty）都会等 GUI 程序结束并显示输出，所以两个二进制都能用：

```bash
dev-tools task-branch-group list      # 也行
dev-tools-cli task-branch-group list  # 也行
```

### 0.4 macOS：从 .app 包里找到 `dev-tools`

DMG 安装的应用：

1. **访达 → 应用程序**，找到从 DMG 安装的应用。
2. 右键 → **显示包内容**。
3. 进入 **`Contents/MacOS/`**，里面有 **`dev-tools`**（无扩展名）。

路径形如：

```text
/Applications/<你的应用名>.app/Contents/MacOS/dev-tools
```

> macOS 没有"GUI 子系统不被等待"的问题，Terminal 直接调用即可，**不需要** `dev-tools-cli`。当前版本 macOS 的 DMG 也只装 `dev-tools`，Windows 才双发。

可选：以后想直接敲 `dev-tools`：

```bash
sudo ln -sf "/Applications/<你的应用名>.app/Contents/MacOS/dev-tools" /usr/local/bin/dev-tools
```

或写 alias：

```bash
alias dev-tools='"/Applications/<你的应用名>.app/Contents/MacOS/dev-tools"'
```

---

## 1. 从源码构建：两个 bin 在哪

仓库根：

```bash
cd src-tauri
cargo build --release        # 同时编译 dev-tools 与 dev-tools-cli
```

| 平台 | GUI 调试版 | GUI 发布版 | CLI 发布版（仅 Windows 需要） |
|------|------------|------------|--------------------------------|
| macOS / Linux | `src-tauri/target/debug/dev-tools` | `src-tauri/target/release/dev-tools` | `src-tauri/target/release/dev-tools-cli`（一般用不上）|
| Windows | `src-tauri\target\debug\dev-tools.exe` | `src-tauri\target\release\dev-tools.exe` | `src-tauri\target\release\dev-tools-cli.exe` |

打 NSIS 安装包：

```bash
npx tauri build --bundles nsis
# → src-tauri/target/release/bundle/nsis/dev-tools_<version>_x64-setup.exe
```

> 实现：`src-tauri/installer.nsh` 是 Tauri 的 NSIS installer hook，在打包阶段把 `dev-tools-cli.exe` 通过 `File` 指令塞进安装器，安装时复制到 `$INSTDIR`，卸载时一并删除。无需手工拷贝。

---

## 2. 不修改 PATH：本机开发常用

### 2.1 `cargo run`（仅在已克隆仓库、装好 Rust 时）

注意 `--` 后才是程序参数，并显式指定 `--bin`：

```bash
# GUI 二进制（Windows GUI 子系统；PowerShell 仍然不会显示输出）
cargo run --release --manifest-path src-tauri/Cargo.toml --bin dev-tools -- task-branch-group list

# CLI 二进制（任意 shell 都能用）
cargo run --release --manifest-path src-tauri/Cargo.toml --bin dev-tools-cli -- task-branch-group list
```

### 2.2 脚本里写相对路径（仅开发机）

```powershell
.\src-tauri\target\release\dev-tools-cli.exe task-branch-group list
```

```bash
./src-tauri/target/release/dev-tools-cli task-branch-group list
```

---

## 3. 长期可用：把 CLI 放进 PATH（可选）

### 3.1 Windows：自定义目录加入 Path

将 `dev-tools-cli.exe`（视情况连同 `dev-tools.exe`）复制到例如 `C:\Tools\dev-tools\`，再把该文件夹加入用户 `Path`（不要只加 exe 文件路径）。

如果是通过 NSIS 安装的，**直接把安装目录**（如 `C:\Program Files\dev-tools`）加进 PATH 即可，两个二进制都在里面。

### 3.2 macOS：`/usr/local/bin` 或 `~/bin`

参见 0.4 的 `ln -sf` 写法，或把 `target/release/dev-tools` 复制到 `~/bin` 并把 `~/bin` 加入 `PATH`。

### 3.3 `cargo install --path src-tauri`（面向开发者）

会把另一份 `dev-tools` 装到 `~/.cargo/bin` 或 `%USERPROFILE%\.cargo\bin`。注意这只装 `default-run` 指定的那一个 bin（即 `dev-tools`，GUI 子系统）。如果想用 `cargo install` 装 CLI 二进制，加 `--bin dev-tools-cli`：

```bash
cargo install --path src-tauri --bin dev-tools-cli
```

---

## 4. 为什么 PowerShell 调 GUI 子系统的程序"没反应"

`src-tauri/src/main.rs` 顶部：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

release 版编译为 GUI 子系统程序。`main.rs` 里虽然 `AttachConsole(ATTACH_PARENT_PROCESS)` 试图附回父控制台，但 PowerShell 启动 GUI 程序时的策略是「立即 fork、立即返回」——它**不等待**、**不接管 stdout**，所以你看到的就是新一行提示符，输出被丢掉了。

cmd / git bash（mintty + ConPTY）不一样，它们会等 GUI 程序结束并显示输出，这就是为什么之前在 git bash 里能看到 `dev-tools --help` 的输出，PowerShell 里看不到。

`src-tauri/src/bin/dev-tools-cli.rs` 不带 `windows_subsystem` 属性，默认 console 子系统，PowerShell 会等它结束并接管 stdout，从根上避开这个问题。两个 bin 共用 `src-tauri/src/cli.rs` 里的子命令定义，行为完全一致。

---

## 5. 相关文档

- 子命令与 HTTP：[`task-branch-group-对接文档.md`](task-branch-group-对接文档.md)
