# MOC - cURL 格式化

## 文档列表

| 类型 | 文档 | 状态 |
| --- | --- | --- |
| PRD | [cURL 格式化需求](prd/PRD-cURL格式化.md) | completed |
| FLD | [前端设计](design/fld/FLD-cURL格式化.md) | completed |
| 测试 | [兼容转换测试](testcase/testcase-兼容转换-2026-11-03.md) | completed |

## 关联信息

- 模块：`curl-formatter`，纯前端本地转换。
- 页面：[CurlFormatter.vue](../../../src/views/CurlFormatter.vue)。
- 转换器：[curlFormatter.ts](../../../src/utils/curlFormatter.ts)。
- 自动化验证：[curlFormatter.test.mjs](../../../tests/curlFormatter.test.mjs)，执行 `npm run test:curl`。
- 原始输入：用户本轮需求与附件；附件含认证数据，不复制到仓库，测试采用虚构样例。
- 文档规则：[docs-prd.md](../../../.codex/rules/common/docs-prd.md)。

## 变更日志

- 2026-11-03 新增需求、前端设计、兼容转换菜单和自动化验证；目标版本经用户确认为 cURL 7.29。
- 2026-11-03 完成本机构建、14 组自动化和浏览器操作验证；保留目标 Linux 与 Tauri 真机未测范围。
