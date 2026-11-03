# Dev Tools 文档导航

## 现有对接与操作说明

| 文档 | 内容 |
| --- | --- |
| [Cookie Bridge 对接](cookie-bridge-对接文档.md) | 浏览器扩展推送、HTTP / IPC 查询与本地存储 |
| [域名管理对接](domain-management-对接文档.md) | 域名 CRUD、URL 匹配、关联数据和 CLI |
| [任务分支管理对接](task-branch-group-对接文档.md) | 任务分支关联、HTTP / IPC / CLI 与软删除 |
| [CLI 安装与 PATH](dev-tools-cli-安装与PATH.md) | macOS / Windows 安装、双二进制与命令行使用 |
| [Git 合并远程](git-合并远程.md) | 已有 Git 操作说明 |

## 需求文档

新需求归档到 `docs/产品文档/{需求}/`，通过需求下的 `MOC.md` 关联 PRD、FLD、LLD、发布说明和测试用例。新增需求时在本节追加 MOC 链接，按需创建目录。

现有对接文档保持原路径，接口变动时持续更新。OpenSpec 变更仍在 `openspec/` 管理，由对应需求 MOC 链接。

## 协作规则与技能

- 项目介绍、目录和常用命令：[AGENTS.md](../AGENTS.md)。
- 文档结构、元数据、状态和质量要求：[docs-prd.md](../.codex/rules/common/docs-prd.md)。
- 需求与设计技能位于 `.agents/skills/`，入口见 `AGENTS.md` 的技能表；沿用本项目已有技能目录，不重复存储副本。
- `status: completed` 表示文档交付完成；实施、实测、验收及发布结果单独记录。当前未配置由此状态触发的外部文档同步。
