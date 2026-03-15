# System Prompt 学习平台 — 里程碑分解

## 项目概述

一个面向"学习 System Prompt"的 Web 平台。用户通过**写 Prompt → 跑测试 → 看结果 → 改 Prompt** 的循环来学习 Prompt 编写技巧。

**技术栈**：React + TypeScript（前端）| Rust + Axum（后端）| SQLite（数据库）

---

## 里程碑依赖关系

```
M1 (骨架 + Prompt CRUD)
 └──→ M2 (分层编辑 + 版本管理)
       └──→ M3 (多模型配置 + 在线执行)
             ├──→ M4 (断言引擎 + Fixture 回放)
             │     └──→ M6 (报告 + CI + 安全) ←─┐
             └──→ M5 (Trace 追踪 + 可视化) ────┘
```

## 数据模型演进

| 里程碑 | 新增表 |
|--------|--------|
| M1 | `prompts` |
| M2 | `projects`, `prompt_layers`, `prompt_versions` |
| M3 | `providers`, `models`, `test_cases`, `runs`, `run_results` |
| M4 | `assertions`, `assertion_results`, `fixtures`, `baselines` |
| M5 | `trace_spans` |
| M6 | `run_reports`, `audit_logs` |

---

## 里程碑一：项目骨架搭建与 Prompt 基础管理

### 目的

建立 React + Rust/Axum + SQLite 的端到端开发环境，验证技术栈可行性。让开发者从第一天就能看到"前端发请求 → 后端处理 → 数据库存取 → 页面展示"的完整闭环，建立信心并形成开发节奏。

### 包含功能

1. 前后端项目脚手架（monorepo 结构）
2. SQLite 数据库初始化与迁移机制
3. System Prompt 的 CRUD 操作（创建、读取、更新、删除）
4. 单层 Prompt 编辑器（纯文本，暂不支持分层）
5. 基础页面路由与布局框架（侧边栏 + 主内容区）
6. API 健康检查端点

### 实现方式

**后端（Rust）：**
- `axum` 作为 HTTP 框架，`tokio` 作为异步运行时
- `sqlx` 连接 SQLite，`sqlx::migrate!` 管理数据库迁移
- `serde` / `serde_json` 做序列化反序列化
- `tower-http` 提供 CORS 中间件
- 数据模型：`prompts` 表（id, name, content, created_at, updated_at）

**前端（React）：**
- Vite 初始化 React + TypeScript 项目
- `react-router-dom` 做客户端路由
- `@tanstack/react-query` 管理服务端状态与缓存
- `tailwindcss` 做样式
- 基础页面：Prompt 列表页、Prompt 编辑页

**项目结构：**
```
system-prompt-test/
├── frontend/          # React + Vite
│   ├── src/
│   │   ├── pages/     # PromptList, PromptEditor
│   │   ├── api/       # HTTP client 封装
│   │   └── components/
│   └── package.json
├── backend/           # Rust + Axum
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/
│   │   ├── models/
│   │   └── db/
│   └── Cargo.toml
└── docs/
```

### 测试方式

**手动测试：**
1. 启动后端，访问 `GET /api/health` 返回 200
2. 在前端创建一条 Prompt，刷新页面后仍然存在
3. 编辑 Prompt 内容，保存后重新加载确认更新生效
4. 删除 Prompt，确认列表中消失

**自动化测试：**
- 后端：`#[tokio::test]` 集成测试，`sqlx::test` 提供测试数据库隔离
- 前端：Vitest + React Testing Library 组件渲染测试
- 可选：`insta` crate 做 API 快照测试

### 交付物

- 可运行的前后端项目（`cargo run` + `npm run dev`）
- SQLite 数据库 schema（v1 迁移脚本）
- Prompt CRUD 的 REST API（4 个端点）
- 基础 UI：Prompt 列表页 + 编辑页
- README 包含本地启动说明

### 依赖关系

无（第一个里程碑）

---

## 里程碑二：分层 Prompt 编辑与版本管理

### 目的

实现 Prompt 管理的核心差异化能力——分层组合（global/project/provider/model）和版本管理。这是整个平台的基石功能，让用户能够像管理代码一样管理 Prompt：有层级继承、有版本历史、可回滚、可对比差异。

### 包含功能

1. 分层 Prompt 编辑器（四层：全局 → 项目 → 提供商 → 模型）
2. 层级合并预览（实时展示最终合成的 Prompt 内容）
3. 版本管理：每次保存自动生成版本快照
4. 版本历史列表与版本间 diff 对比
5. 版本回滚功能
6. 变量注入系统（`{{variable_name}}` 模板语法）
7. 项目（Project）概念的 CRUD

### 实现方式

**后端：**
- 新增数据模型：
  - `projects` 表（id, name, description, created_at）
  - `prompt_layers` 表（id, project_id, layer_type, target_ref, content, version, created_at）
  - `prompt_versions` 表（id, prompt_layer_id, version, content, created_at）
- 层级合并逻辑：按 global → project → provider → model 顺序拼接，支持覆盖与追加
- 变量注入：使用 `handlebars` crate 或自定义正则替换 `{{...}}`
- diff 生成：使用 `similar` crate

**前端：**
- `@monaco-editor/react`（Monaco Editor）作为 Prompt 编辑器
- 分层 UI：标签页切换不同层级，右侧面板展示合并预览
- 版本 diff：Monaco Editor 内置 diff 视图
- 变量面板：独立区域管理变量键值对

### 测试方式

**手动测试：**
1. 创建项目，分别在 global 和 project 层编写 Prompt
2. 查看合并预览，确认两层内容正确拼接
3. 修改内容后查看版本历史，确认新版本出现
4. 选两个版本做 diff 对比，确认差异高亮正确
5. 回滚到旧版本，确认内容恢复
6. 插入 `{{user_name}}`，设置变量值，确认预览中被替换

**自动化测试：**
- 层级合并逻辑的单元测试（各种组合场景）
- 版本创建与回滚的集成测试
- 变量注入边界测试（未定义变量、特殊字符）

### 交付物

- 分层 Prompt 编辑器 UI（含 Monaco Editor）
- 版本管理系统（历史、diff、回滚）
- 变量注入引擎
- 项目管理 CRUD
- 数据库迁移脚本（v2）

### 依赖关系

依赖里程碑一的项目骨架、数据库基础设施、前端路由与布局

---

## 里程碑三：多模型配置与在线执行

### 目的

让平台从"静态编辑器"进化为"可执行的测试环境"。用户第一次能够真正将编写的 Prompt 发送给 AI 模型并看到返回结果，实现"写 Prompt → 跑测试 → 看结果"的核心学习循环。同时引入多模型对比能力。

### 包含功能

1. AI 提供商配置（OpenAI、Anthropic、Azure 等）
2. API Key 管理（加密存储、界面脱敏显示）
3. 模型能力矩阵配置（tool_call / streaming / json_mode / mcp）
4. 测试用例定义（case name + user message + 期望配置）
5. 在线执行：将合成 Prompt + 测试用例发送至选定模型
6. 多模型并行执行，结果并排对比展示
7. 执行结果持久化存储

### 实现方式

**后端：**
- 新增数据模型：
  - `providers` 表（id, name, api_type, base_url, encrypted_api_key, created_at）
  - `models` 表（id, provider_id, model_name, capabilities JSON, is_active）
  - `test_cases` 表（id, project_id, name, user_message, config JSON, created_at）
  - `runs` 表（id, test_case_id, prompt_version_id, model_id, status, started_at, finished_at）
  - `run_results` 表（id, run_id, response_text, token_usage JSON, latency_ms, raw_response JSON）
- API Key 加密：`aes-gcm` crate 进行 AES-256-GCM 加密，密钥从环境变量读取
- HTTP 客户端：`reqwest` 调用各 AI 提供商 API
- 多模型并行：`tokio::join!` 或 `futures::join_all` 并发请求
- 提供商适配层：定义 `trait LlmProvider`，为 OpenAI/Anthropic 分别实现
- 流式响应：SSE，通过 `axum::response::Sse` 返回

**前端：**
- Provider 配置页面：表单管理提供商和模型信息
- 测试用例编辑页面
- 执行面板：选择 case + 选择模型（可多选）→ 点击"运行"
- 结果对比视图：左右分栏或网格布局
- `EventSource` 或 `fetch` + `ReadableStream` 接收 SSE 流式输出

### 测试方式

**手动测试：**
1. 配置 OpenAI 和 Anthropic 提供商，添加模型
2. 创建测试用例（如"请解释什么是递归"）
3. 选择两个模型并行执行，观察流式输出实时显示
4. 对比两个模型的输出内容和响应时间
5. 刷新页面，确认历史执行结果仍可查看
6. 验证 API Key 显示为 `sk-****xxxx` 脱敏形式

**自动化测试：**
- Provider trait 的 mock 测试（不依赖真实 API）
- API Key 加密/解密往返测试
- 并发执行逻辑测试（mock provider）
- 集成测试：fixture 数据模拟 AI 响应

### 交付物

- Provider/Model 配置管理系统
- 测试用例 CRUD
- 在线执行引擎（含多模型并行）
- 结果对比 UI
- SSE 流式输出支持
- 数据库迁移脚本（v3）

### 依赖关系

依赖里程碑二的分层 Prompt 合成引擎和项目管理功能

---

## 里程碑四：行为验证与 Fixture 回放

### 目的

将平台从"手动观察输出"提升为"自动化验证"。引入断言系统让用户定义预期行为规则并自动判定通过/失败；引入 Fixture 回放机制使测试可脱离真实 API 调用进行确定性重放，这是实现 CI 集成的基础。

### 包含功能

1. 断言规则定义：
   - `must_call`：输出中必须包含对指定工具的调用
   - `must_not_call`：输出中不得调用指定工具
   - `whitelist`：只允许调用白名单内的工具
   - `keyword_present` / `keyword_absent`：关键词存在/不存在检查
2. 断言执行与结果判定（pass/fail + 证据片段）
3. Fixture 保存：将一次真实执行的完整请求/响应序列化保存
4. Fixture 回放：使用保存的 Fixture 替代真实 API 调用，确定性重放
5. 基线对比：将当前执行结果与标记的基线版本进行差异对比

### 实现方式

**后端：**
- 新增数据模型：
  - `assertions` 表（id, test_case_id, assertion_type, config JSON, created_at）
  - `assertion_results` 表（id, run_id, assertion_id, passed, evidence）
  - `fixtures` 表（id, run_id, request_snapshot JSON, response_snapshot JSON, created_at）
  - `baselines` 表（id, test_case_id, run_id, marked_at）
- 断言引擎：定义 `trait Assertion`，含 `fn evaluate(&self, output: &RunOutput) -> AssertionResult`
  - 为每种类型实现：`MustCallAssertion`, `KeywordAssertion` 等
  - `serde_json` 解析 tool_call 结构检测调用行为
- Fixture：保存时序列化请求/响应；回放时在 Provider trait 层拦截，直接返回存储响应
- 基线对比：`similar` crate 做文本 diff，tool_call 列表做集合对比

**前端：**
- 断言配置 UI：测试用例编辑页增加"断言规则"区域
- 执行结果页增加"断言结果"面板（绿色通过 / 红色失败 + 证据展开）
- Fixture 管理："保存为 Fixture"按钮
- 回放模式切换："在线模式"或"Fixture 回放模式"
- 基线标记："标记为基线"按钮，后续执行自动与基线对比

### 测试方式

**手动测试：**
1. 为测试用例添加 `keyword_present: "递归"` 断言，执行验证
2. 添加 `must_call: "search"` 断言，验证 tool call 检测
3. 保存执行结果为 Fixture
4. Fixture 回放模式重新执行，确认结果一致
5. 标记基线，修改 Prompt 后再次执行，观察基线对比差异

**自动化测试：**
- 每种断言类型的单元测试（正例 + 反例）
- Fixture 回放端到端一致性测试
- 基线 diff 正确性测试
- 边界测试：空输出、超长输出、无 tool_call 等

### 交付物

- 断言规则引擎（4+ 种断言类型）
- Fixture 存储与回放系统
- 基线标记与对比功能
- 断言配置 UI 和结果展示 UI
- 数据库迁移脚本（v4）

### 依赖关系

依赖里程碑三的执行引擎和运行结果存储

---

## 里程碑五：调用链路追踪与可视化

### 目的

为用户提供"白盒透视"能力——完整展示从 Prompt 发送到最终输出的每一步，包括中间的 tool call、tool result、多轮交互。这是理解模型行为和调试 Prompt 的关键能力，让"黑盒"变成"白盒"。

### 包含功能

1. 调用链路数据采集：记录 prompt → model response → tool calls → tool results 序列
2. 时间线可视化：按时间轴展示每个阶段的开始/结束时间和内容
3. 嵌套 span 结构：支持层级展示
4. Span 详情面板：点击节点查看完整请求/响应
5. 耗时分析：各阶段耗时占比可视化（瀑布图样式）
6. 敏感信息自动脱敏

### 实现方式

**后端：**
- 新增：`trace_spans` 表（id, run_id, parent_span_id, span_type, name, start_time, end_time, metadata JSON, content）
- 在 Provider 适配层的请求/响应生命周期中插入 span 记录
- 使用 `tracing` crate 的 span 概念管理层级（存储到 SQLite）
- 解析 OpenAI/Anthropic 的 tool_use / function_call 响应格式
- 脱敏：`regex` crate 定义敏感字段正则，存储前替换

**前端：**
- 自定义 React 组件（SVG）绘制瀑布图
- 颜色编码：蓝色=LLM 请求、绿色=LLM 响应、橙色=tool call、紫色=tool result
- 点击 span 在右侧面板展示详情（格式化 JSON）
- 耗时占比：水平堆叠条形图

### 测试方式

**手动测试：**
1. 执行包含 tool call 的测试用例
2. 打开 Trace 视图，确认时间线展示完整调用链路
3. 验证 span 父子关系正确
4. 点击节点确认详情内容完整
5. 确认敏感信息已脱敏为 `[REDACTED]`

**自动化测试：**
- 模拟完整执行，验证 span 树结构正确
- 脱敏正则匹配测试
- 前端时间线组件渲染测试
- 大量 span（100+）渲染性能测试

### 交付物

- Trace 数据采集与存储系统
- 时间线可视化组件
- Span 详情查看面板
- 敏感信息脱敏引擎
- 数据库迁移脚本（v5）

### 依赖关系

依赖里程碑三的执行引擎，依赖里程碑四的 Fixture 机制（回放模式也需生成 Trace）

---

## 里程碑六：报告系统、CI 集成与安全加固

### 目的

让平台从"开发者工具"升级为"工程化基础设施"。提供结构化运行报告、跨版本趋势分析和 CI 友好的 JSON 导出，使 Prompt 质量管理可纳入 CI/CD 流程。同时完善安全审计功能。

### 包含功能

1. 单次运行报告：延迟、Token 用量、成本估算、断言结果汇总
2. 跨版本趋势分析：不同 Prompt 版本的性能/通过率趋势图
3. 失败分类：自动归类（断言失败、超时、API 错误、模型拒绝等）
4. JSON 报告导出：结构化格式，适配 CI 消费
5. CLI 触发接口：`POST /api/runs/batch` 支持 CI 脚本触发批量执行
6. 审计日志：记录所有关键操作
7. 全链路 Secret 脱敏加固

### 实现方式

**后端：**
- 新增：`run_reports` 表、`audit_logs` 表
- 每次 run 完成后自动聚合生成 report
- 模型定价配置表 + `rust_decimal` 做成本精确计算
- 趋势查询：SQL 按 prompt_version 分组聚合
- JSON 导出：`GET /api/reports/{run_id}/export`
- 批量执行：`POST /api/runs/batch` 接受 case IDs + model IDs
- 审计：Axum middleware（`tower::Layer`）统一拦截记录
- 全链路 sanitization 管道

**前端：**
- 报告仪表板：运行卡片 + `recharts` 趋势图
- 失败分析视图：按类型分组 + 下钻
- 导出按钮：一键下载 JSON
- 审计日志查看页

### 测试方式

**手动测试：**
1. 三个不同 Prompt 版本分别执行同一 case，查看趋势图
2. 导出 JSON 报告验证格式完整
3. `curl` 调用批量执行 API 模拟 CI
4. 检查审计日志覆盖所有操作
5. 搜索报告和日志确认无 API Key 泄露

**自动化测试：**
- 报告各字段计算正确性（特别是成本）
- JSON 导出 schema 验证
- 批量执行 API 集成测试
- 审计 middleware 覆盖测试
- 脱敏完整性测试
- E2E：模拟 CI 完整流程

### 交付物

- 运行报告生成与展示系统
- 跨版本趋势分析图表
- JSON 报告导出 API
- CI 批量执行触发 API
- 审计日志系统
- 全链路敏感信息脱敏
- 数据库迁移脚本（v6）
- CI 集成示例脚本（GitHub Actions 示例）

### 依赖关系

依赖里程碑三（执行数据）、里程碑四（断言结果）、里程碑五（Trace 数据）
