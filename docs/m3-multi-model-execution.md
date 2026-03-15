# 里程碑三：多模型配置与在线执行 — 详细设计文档

## 目的

让平台从"静态编辑器"进化为"可执行的测试环境"。用户第一次能够真正将编写的 Prompt 发送给 AI 模型并看到返回结果，实现"写 Prompt → 跑测试 → 看结果"的核心学习循环。同时引入多模型对比能力，让用户可以在同一测试用例下并行对比不同模型的输出质量、响应速度和 Token 消耗。

### 为什么这个里程碑是关键的

- **核心价值闭环**：M1/M2 只是"编辑"，M3 让平台第一次"活"起来——Prompt 不再是静态文本，而是可以真正发送给 AI 模型并得到反馈
- **多模型对比**：这是 Prompt 工程的核心需求——同一个 Prompt 在不同模型上表现可能截然不同，对比能力让用户快速找到最佳 Prompt-模型组合
- **为后续里程碑奠基**：M4 的断言引擎、M5 的 Trace 追踪、M6 的报告系统，全部建立在 M3 的执行结果之上

---

## 功能清单

| # | 功能 | 说明 |
|---|------|------|
| 1 | AI 提供商配置 | 支持 OpenAI、Anthropic 两种 API 类型，可配置名称、Base URL、API Key |
| 2 | API Key 安全管理 | AES-256-GCM 加密存储、界面脱敏显示 (`sk-****xxxx`) |
| 3 | 模型注册与管理 | 每个 Provider 下注册具体模型名称，支持能力矩阵 (capabilities JSON) 和启用/停用 |
| 4 | 测试用例定义 | 绑定项目，包含用例名称、用户消息、可选配置 JSON |
| 5 | 在线执行 | 合成 Prompt（四层合并 + 变量注入）+ 测试用例 → 发送至选定模型 |
| 6 | 多模型并行执行 | 一次运行选择多个模型，`tokio::spawn` 并发请求，结果并排对比 |
| 7 | SSE 流式输出 | 通过 Server-Sent Events 实时流式推送模型响应到前端 |
| 8 | 执行结果持久化 | 存储响应文本、Token 用量、延迟、原始响应 JSON、错误信息 |

---

## 数据模型

### 数据库 Schema（migration 003）

```sql
-- 提供商表
CREATE TABLE providers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,                                          -- 显示名称，如 "My OpenAI"
    api_type TEXT NOT NULL CHECK(api_type IN ('openai', 'anthropic')),  -- API 协议类型
    base_url TEXT NOT NULL,                                      -- API Base URL
    encrypted_api_key TEXT NOT NULL,                              -- AES-256-GCM 加密后的 API Key
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 模型表
CREATE TABLE models (
    id TEXT PRIMARY KEY NOT NULL,
    provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    model_name TEXT NOT NULL,              -- 如 "gpt-4o", "claude-sonnet-4-20250514"
    capabilities TEXT NOT NULL DEFAULT '{}',  -- JSON: {"tool_call":true, "streaming":true, "json_mode":false}
    is_active INTEGER NOT NULL DEFAULT 1,  -- 启用/停用开关
    created_at TEXT NOT NULL
);

-- 测试用例表
CREATE TABLE test_cases (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                    -- 用例名称
    user_message TEXT NOT NULL,            -- 发送给模型的用户消息
    config TEXT NOT NULL DEFAULT '{}',     -- 可选配置 JSON (max_tokens, temperature 等)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 运行记录表
CREATE TABLE runs (
    id TEXT PRIMARY KEY NOT NULL,
    test_case_id TEXT NOT NULL REFERENCES test_cases(id) ON DELETE CASCADE,
    model_id TEXT NOT NULL REFERENCES models(id),
    status TEXT NOT NULL DEFAULT 'pending',   -- pending | running | completed | failed
    system_prompt TEXT NOT NULL,              -- 执行时的合成 Prompt 快照
    started_at TEXT,
    finished_at TEXT,
    created_at TEXT NOT NULL
);

-- 运行结果表
CREATE TABLE run_results (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    response_text TEXT NOT NULL DEFAULT '',   -- 模型返回的文本
    token_usage TEXT NOT NULL DEFAULT '{}',   -- JSON: {"prompt_tokens":N, "completion_tokens":N, "total_tokens":N}
    latency_ms INTEGER,                      -- 请求耗时（毫秒）
    raw_response TEXT NOT NULL DEFAULT '{}',  -- 完整 API 原始响应 JSON
    error_message TEXT,                      -- 失败时的错误信息
    created_at TEXT NOT NULL
);
```

### 实体关系

```
providers
└── models (1:N, CASCADE DELETE)

projects (M2)
├── prompt_layers (M2, 用于合成 system_prompt)
└── test_cases (1:N, CASCADE DELETE)
    └── runs (1:N, CASCADE DELETE)
        └── run_results (1:1, CASCADE DELETE)
```

### Rust 数据模型

**Provider 模型** (`backend/src/models/provider.rs`)：

| 结构体 | 用途 |
|--------|------|
| `Provider` | 数据库行映射，含 `encrypted_api_key` |
| `ProviderResponse` | API 响应体，`api_key_masked` 替代原始 Key |
| `CreateProviderRequest` | 创建请求体 (name, api_type, base_url, api_key) |
| `UpdateProviderRequest` | 更新请求体 (均 Optional，api_key 为空表示不更新) |

**Model 模型** (`backend/src/models/model.rs`)：

| 结构体 | 用途 |
|--------|------|
| `AiModel` | 数据库行映射 (id, provider_id, model_name, capabilities JSON, is_active) |
| `CreateModelRequest` | 创建请求体 |
| `UpdateModelRequest` | 更新请求体 |

**Run 模型** (`backend/src/models/run.rs`)：

| 结构体 | 用途 |
|--------|------|
| `Run` | 运行记录（含 status 状态机） |
| `RunResult` | 运行结果（response_text, token_usage, latency_ms, raw_response, error_message） |
| `RunWithResult` | Run + 可选 Result 的聚合视图（`#[serde(flatten)]`） |
| `RunRequest` | 执行请求体 (model_ids[], variables{}) |

---

## 后端实现细节

### 1. Provider 管理模块

**文件**：`backend/src/routes/providers.rs`

#### API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/providers` | 列出所有 Provider（API Key 脱敏返回） |
| POST | `/api/providers` | 创建 Provider（加密 API Key） |
| GET | `/api/providers/{id}` | 获取单个 Provider |
| PUT | `/api/providers/{id}` | 更新 Provider（api_key 为空时保留旧值） |
| DELETE | `/api/providers/{id}` | 删除 Provider（级联删除关联 Models） |

#### 关键实现逻辑

**创建 Provider**：
1. 生成 ULID 作为 ID
2. 调用 `crypto::encrypt(&payload.api_key)` 加密 API Key
3. 插入数据库，返回 `ProviderResponse`（api_key_masked）

**读取 Provider**：
1. 查询数据库获取 `Provider`（含 encrypted_api_key）
2. 调用 `crypto::decrypt()` 解密 → `crypto::mask_api_key()` 脱敏
3. 仅返回 `ProviderResponse`，**永远不返回明文或密文 Key**

**更新 Provider**：
1. 查询现有记录
2. 合并非空字段（`Option::unwrap_or` 保留旧值）
3. 若提供新 api_key 则重新加密；否则保留 encrypted_api_key 不变

### 2. API Key 加密模块

**文件**：`backend/src/crypto.rs`

**算法**：AES-256-GCM（AEAD 认证加密）

**加密流程**：
1. 从环境变量 `ENCRYPTION_KEY` 读取 Base64 编码的 32 字节密钥（开发环境默认全零）
2. 生成 12 字节随机 Nonce
3. AES-256-GCM 加密明文
4. 拼接 Nonce(12B) + Ciphertext → Base64 编码后存储

**解密流程**：
1. Base64 解码
2. 分离前 12 字节（Nonce）和剩余（Ciphertext）
3. AES-256-GCM 解密

**脱敏规则** (`mask_api_key`)：
- Key 长度 <= 8：返回 `****`
- 以 `sk-` 开头：返回 `sk-****{last4}`
- 其他：返回 `****{last4}`

### 3. Model 管理模块

**文件**：`backend/src/routes/models_routes.rs`

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/providers/{provider_id}/models` | 列出指定 Provider 下的所有模型 |
| POST | `/api/providers/{provider_id}/models` | 创建模型 |
| PUT | `/api/models/{id}` | 更新模型 |
| DELETE | `/api/models/{id}` | 删除模型 |
| GET | `/api/models` | 列出所有活跃模型（跨 Provider，`is_active = 1`） |

`capabilities` 字段为 JSON 字符串，预留用于表示模型能力矩阵：
```json
{
  "tool_call": true,
  "streaming": true,
  "json_mode": false,
  "mcp": false
}
```

### 4. 测试用例管理模块

**文件**：`backend/src/routes/test_cases.rs`

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/projects/{project_id}/test-cases` | 列出项目下所有测试用例 |
| POST | `/api/projects/{project_id}/test-cases` | 创建测试用例 |
| GET | `/api/test-cases/{id}` | 获取单个测试用例 |
| PUT | `/api/test-cases/{id}` | 更新测试用例 |
| DELETE | `/api/test-cases/{id}` | 删除测试用例（级联删除关联 Runs） |

`config` JSON 预留用于存储测试用例级别的配置：
```json
{
  "max_tokens": 1024,
  "temperature": 0.5
}
```

### 5. 执行引擎

**文件**：`backend/src/routes/runs.rs`

这是 M3 最核心的模块，实现了完整的执行流程。

#### API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | `/api/test-cases/{id}/run` | 创建并执行 Run（支持多模型并行） |
| GET | `/api/test-cases/{id}/runs` | 获取测试用例的所有运行历史 |
| GET | `/api/runs/{id}` | 获取单次运行详情（含 Result） |
| GET | `/api/runs/{id}/stream` | SSE 流式获取运行输出 |

#### 执行流程（`POST /api/test-cases/{id}/run`）

```
前端发起请求
  │
  ├─ 请求体: { model_ids: ["model_1", "model_2"], variables: {"name": "test"} }
  │
  ▼
1. 查询 test_case（获取 project_id, user_message）
  │
  ▼
2. 查询 prompt_layers（按 global → project → provider → model 排序）
  │
  ▼
3. 合成 system_prompt
   ├─ 过滤空层
   ├─ 以 "\n\n" 拼接各层 content
   └─ 正则替换 {{variable}} → 注入变量值
  │
  ▼
4. 为每个 model_id 创建 Run 记录（status = 'running'）
  │
  ▼
5. 为每个 Run 启动 tokio::spawn 异步任务
   ├─ 查询 Model → Provider
   ├─ 解密 API Key
   ├─ 创建 LlmProvider 实例
   ├─ 构造 LlmRequest
   ├─ 调用 provider.complete()
   ├─ 成功：保存 RunResult + 更新 status='completed'
   └─ 失败：保存 error_message + 更新 status='failed'
  │
  ▼
6. 立即返回 Run[] 给前端（不等异步任务完成）
```

**关键设计决策**：
- **快速响应**：`POST /run` 立即返回所有 Run 记录（status=running），前端通过轮询或 SSE 获取最终结果
- **并行执行**：每个模型调用通过独立的 `tokio::spawn` 并行执行，互不阻塞
- **Prompt 快照**：Run 记录中保存 `system_prompt` 字段，确保即使 Prompt 后续被修改，历史运行的执行上下文仍可追溯

#### SSE 流式输出（`GET /api/runs/{id}/stream`）

```
前端连接 EventSource
  │
  ▼
1. 查询 Run → Model → Provider，解密 API Key
  │
  ▼
2. 创建 mpsc::channel<StreamEvent>(100) 缓冲通道
  │
  ▼
3. tokio::spawn 调用 provider.stream(request, tx)
   └─ Provider 内部解析 SSE 流，逐 chunk 发送 StreamEvent 到 tx
  │
  ▼
4. 将 rx 包装为 ReceiverStream → 映射为 axum::response::sse::Event
  │
  ▼
5. 返回 Sse<impl Stream> 给前端

StreamEvent 类型:
  - { event_type: "delta", content: "部分文本" }    -- 增量内容
  - { event_type: "done", token_usage: {...} }       -- 完成，附带 Token 统计
  - { event_type: "error", error: "错误信息" }       -- 出错
```

### 6. LLM Provider 适配层

**文件**：`backend/src/llm/mod.rs`, `openai.rs`, `anthropic.rs`

#### 抽象接口

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 同步请求：发送后等待完整响应返回
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, String>;

    /// 流式请求：通过 mpsc channel 逐步发送 StreamEvent
    async fn stream(
        &self,
        request: &LlmRequest,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<(), String>;
}
```

#### 工厂函数

```rust
pub fn create_provider(api_type: &str, base_url: &str, api_key: &str) -> Box<dyn LlmProvider> {
    match api_type {
        "openai" => Box::new(OpenAiProvider::new(...)),
        "anthropic" => Box::new(AnthropicProvider::new(...)),
        _ => Box::new(OpenAiProvider::new(...)),  // 默认回退 OpenAI 兼容协议
    }
}
```

#### OpenAI 适配器 (`openai.rs`)

**API 端点**：`{base_url}/v1/chat/completions`

**请求格式**：
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "system", "content": "<merged system prompt>"},
    {"role": "user", "content": "<test case user message>"}
  ],
  "max_tokens": 2048,
  "temperature": 0.7
}
```

**认证方式**：`Authorization: Bearer {api_key}`

**同步响应解析**：
- 内容：`response.choices[0].message.content`
- Token：`response.usage.{prompt_tokens, completion_tokens, total_tokens}`

**流式处理**：
- 请求增加 `"stream": true`
- 逐行读取 SSE，解析 `data: {...}` 行
- 增量内容：`data.choices[0].delta.content`
- 结束标记：`data: [DONE]`
- 使用字节流 + 缓冲区逐行解析，处理 chunk 边界分割问题

#### Anthropic 适配器 (`anthropic.rs`)

**API 端点**：`{base_url}/v1/messages`

**请求格式**：
```json
{
  "model": "claude-sonnet-4-20250514",
  "system": "<merged system prompt>",
  "messages": [{"role": "user", "content": "<user message>"}],
  "max_tokens": 2048,
  "temperature": 0.7
}
```

**认证方式**：`x-api-key: {api_key}` + `anthropic-version: 2023-06-01`

**注意与 OpenAI 的差异**：
- system prompt 是顶层字段 `"system"`，而非 messages 数组中的 role
- Token 字段名不同：`input_tokens` / `output_tokens`（vs `prompt_tokens` / `completion_tokens`）
- 内容路径：`response.content[0].text`（vs `choices[0].message.content`）

**流式处理**：
- SSE 格式不同：使用 `event:` 行标记事件类型
- `event: content_block_delta` + `data.delta.text` → 增量内容
- `event: message_delta` + `data.usage.output_tokens` → Token 统计
- `event: message_stop` → 结束标记

---

## 前端实现细节

### 1. Provider 配置页面

**文件**：`frontend/src/pages/ProviderSettings.tsx`

**路由**：`/settings/providers`

**功能**：
- Provider 列表展示（名称、API 类型、Base URL、脱敏 API Key）
- 创建/编辑 Provider 表单（名称、API 类型选择、Base URL、API Key 密码输入框）
- 编辑时 API Key 字段留空表示不更新
- 删除 Provider（级联删除关联模型）
- 每个 Provider 卡片内嵌模型管理
  - 显示已注册模型列表（标签样式）
  - 输入框 + 添加按钮注册新模型
  - 删除单个模型

**状态管理**：
- `useQuery(['providers'])` → 获取 Provider 列表
- `useQuery(['models', providerId])` → 获取每个 Provider 的模型列表
- `useMutation` + `invalidateQueries` → 创建/更新/删除操作后刷新缓存

### 2. 测试用例面板

**文件**：`frontend/src/components/TestCasePanel.tsx`

**位置**：ProjectEditor 页面的 Test 标签页内

**功能**：
- 测试用例列表（点击选中）
- 新建/编辑测试用例表单（名称 + 用户消息 textarea）
- 删除测试用例
- 选中状态高亮

**交互**：
- 选中测试用例 → 触发 `onSelect(caseId)` → RunPanel 和 RunResultsView 联动显示

### 3. 运行面板

**文件**：`frontend/src/components/RunPanel.tsx`

**功能**：
- 获取所有活跃模型 (`useQuery(['allModels'])`)
- 多选模型（checkbox 标签式 UI）
- 点击 "Run" 按钮触发 `createRun(testCaseId, { model_ids, variables })`
- 运行中状态禁用按钮
- 运行完成后回调 `onRunsCreated(runs)` → 触发 RunResultsView 刷新

### 4. 运行结果视图

**文件**：`frontend/src/components/RunResultsView.tsx`

**功能**：
- 历史运行结果查询 (`useQuery(['runs', testCaseId])`)
- 活跃运行时启用 2 秒轮询 (`refetchInterval: 2000`)
- SSE 流式接收：为每个活跃 Run 创建 `EventSource` 连接
  - 增量内容拼接到 `streamContent[runId]`
  - `done` 或 `error` 事件关闭连接
- 网格布局并排展示（`grid-cols-1 md:grid-cols-2`）
- 每个结果卡片包含：
  - 模型 ID
  - 状态徽章（running=黄色, completed=绿色, failed=红色）
  - 响应文本（monospace, 可滚动）
  - 错误信息（红色高亮）
  - 延迟和 Token 统计

### 5. SSE 客户端工具

**文件**：`frontend/src/api/sse.ts`

```typescript
export function streamRun(
  runId: string,
  onEvent: (event: StreamEvent) => void,
  onError?: (error: Error) => void,
): () => void {
  const eventSource = new EventSource(`/api/runs/${runId}/stream`)
  // onmessage → JSON.parse → onEvent callback
  // done/error → eventSource.close()
  return () => eventSource.close()  // 返回清理函数
}
```

- 使用浏览器原生 `EventSource` API
- 返回清理函数供 React `useEffect` 的 cleanup 使用
- 自动处理连接错误和关闭

### 6. API 客户端层

**Provider API** (`frontend/src/api/providers.ts`)：
- `fetchProviders`, `createProvider`, `updateProvider`, `deleteProvider`
- `fetchModels`, `fetchAllModels`, `createModel`, `updateModel`, `deleteModel`

**TestCase & Run API** (`frontend/src/api/testCases.ts`)：
- `fetchTestCases`, `createTestCase`, `updateTestCase`, `deleteTestCase`
- `createRun`, `fetchRuns`, `fetchRun`

所有 API 函数使用 `fetch` + 统一的 `handleResponse<T>` 错误处理。

---

## 页面布局与交互流程

### ProjectEditor 测试标签页布局

```
┌──────────────────────────────────────────────────────────┐
│  [Editor] [Test]                                         │  ← 标签页切换
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌── Test Cases ──────────────────────────────────────┐  │
│  │  [+ New]                                           │  │
│  │  ┌─────────────────────────────────────────────┐   │  │
│  │  │ ● 测试用例1: "请解释递归"         [Edit][Del] │   │  │
│  │  │   测试用例2: "写一首诗"           [Edit][Del] │   │  │
│  │  └─────────────────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌── Select Models to Run ────────────────────────────┐  │
│  │  [✓ gpt-4o] [✓ claude-sonnet] [  gpt-3.5-turbo]   │  │
│  │  [ Run (2 models) ]                                │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌── Run Results ─────────────────────────────────────┐  │
│  │  ┌─────────────────┐  ┌─────────────────┐         │  │
│  │  │ gpt-4o      ✅  │  │ claude-sonnet ✅ │         │  │
│  │  │                 │  │                  │         │  │
│  │  │ 递归是一种函数  │  │ 递归（Recursion）│         │  │
│  │  │ 调用自身的编程  │  │ 是指在函数的定义 │         │  │
│  │  │ 技术...         │  │ 中使用函数自身... │         │  │
│  │  │                 │  │                  │         │  │
│  │  │ 1200ms · 350tok │  │ 980ms · 420tok   │         │  │
│  │  └─────────────────┘  └─────────────────┘         │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### 完整交互流程

```
1. 用户进入 Settings → Providers
   ├─ 添加 OpenAI Provider (名称, base_url, api_key)
   ├─ 在 Provider 下添加模型 "gpt-4o"
   ├─ 添加 Anthropic Provider
   └─ 添加模型 "claude-sonnet-4-20250514"

2. 用户进入项目 → Test 标签页
   ├─ 创建测试用例 (名称: "递归解释", 消息: "请解释什么是递归")
   └─ 选中该测试用例

3. 在 RunPanel 中选择模型
   ├─ 勾选 gpt-4o
   ├─ 勾选 claude-sonnet
   └─ 点击 "Run (2 models)"

4. 前端发送 POST /api/test-cases/{id}/run
   ├─ body: { model_ids: [...], variables: {...} }
   └─ 立即收到 Run[] 响应

5. 前端为每个 Run 建立 SSE 连接
   ├─ EventSource("/api/runs/{run1.id}/stream")
   └─ EventSource("/api/runs/{run2.id}/stream")

6. 后端并行执行
   ├─ tokio::spawn → OpenAI gpt-4o → 流式/完整响应
   └─ tokio::spawn → Anthropic claude-sonnet → 流式/完整响应

7. 前端实时显示
   ├─ delta 事件 → 逐字拼接显示
   ├─ done 事件 → 关闭连接，显示 Token 统计
   └─ 轮询刷新最终持久化结果
```

---

## 依赖的 Crate 和库

### 后端 (Rust)

| Crate | 版本 | 用途 |
|-------|------|------|
| `aes-gcm` | 0.10 | AES-256-GCM 加密 API Key |
| `rand` | (aes-gcm dep) | 生成随机 Nonce |
| `base64` | (aes-gcm dep) | 编解码加密数据 |
| `reqwest` | 0.12 | HTTP 客户端调用 AI 提供商 API |
| `async-trait` | 0.1 | 异步 trait (`LlmProvider`) |
| `tokio` | 1 (full) | 异步运行时 + `spawn` 并发 + `mpsc` 通道 |
| `tokio-stream` | 0.1 | `ReceiverStream` 包装 mpsc Receiver |
| `futures` | 0.3 | `StreamExt` 用于 SSE 流映射 |
| `axum` | 0.8 | `response::sse::Sse` + `Event` |
| `regex` | 1 | 变量注入 `{{...}}` 模式匹配 |

### 前端 (React)

| 包 | 用途 |
|----|------|
| `@tanstack/react-query` | Provider/Model/TestCase/Run 的服务端状态管理 |
| `EventSource` (浏览器原生) | SSE 流式接收 |
| `tailwindcss` | UI 样式 |

---

## 测试方式

### 手动测试

| # | 测试场景 | 预期结果 |
|---|----------|----------|
| 1 | 创建 OpenAI Provider，填入合法 API Key | 创建成功，列表显示脱敏 Key `sk-****xxxx` |
| 2 | 创建 Anthropic Provider | 创建成功，api_type 显示 "anthropic" |
| 3 | 在 Provider 下添加模型 "gpt-4o" | 模型标签出现在 Provider 卡片中 |
| 4 | 编辑 Provider，不填 API Key 保存 | Key 保持不变，其他字段更新 |
| 5 | 编辑 Provider，填入新 API Key | Key 更新为新值，显示新脱敏 Key |
| 6 | 删除 Provider | Provider 和关联模型全部删除 |
| 7 | 创建测试用例 "递归解释" | 列表中出现新用例 |
| 8 | 选中测试用例 + 选择 2 个模型 → Run | 两个结果卡片同时出现，状态从 running → completed |
| 9 | 观察流式输出 | 文本逐字出现（非一次性加载） |
| 10 | 对比两个模型输出 | 结果并排展示，可见不同内容、延迟和 Token |
| 11 | 刷新页面 | 历史运行结果仍可查看 |
| 12 | 使用无效 API Key 执行 | 状态变为 failed，显示错误信息 |
| 13 | 在 Prompt 中使用 `{{name}}`，RunRequest 中传变量 | 合成的 system_prompt 中变量被替换 |

### 自动化测试

#### 后端测试

**单元测试**：

1. **加密模块测试** (`crypto.rs`)
   - 加密/解密往返一致性：`decrypt(encrypt(plaintext)) == plaintext`
   - 不同明文产生不同密文（随机 Nonce）
   - 损坏的密文解密应返回 Err
   - `mask_api_key` 各种长度和前缀的脱敏规则

2. **LLM Provider 适配器测试**
   - Mock HTTP 响应，验证 `complete()` 正确解析 OpenAI 响应格式
   - Mock HTTP 响应，验证 `complete()` 正确解析 Anthropic 响应格式
   - 验证 Token 字段名映射正确（input_tokens → prompt_tokens）
   - HTTP 错误码处理（401, 429, 500）

3. **变量注入测试**
   - 单变量替换
   - 多变量替换
   - 未定义变量保留原样 `{{undefined}}`
   - 无变量时 Prompt 不变

**集成测试** (`#[tokio::test]` + `sqlx::test`)：

4. **Provider CRUD 集成测试**
   - 创建后查询，API Key 已加密存储
   - 更新 api_key 为空时保留旧值
   - 删除 Provider 级联删除 Models

5. **执行引擎集成测试**（Mock LlmProvider）
   - 创建 Run 后 status = 'running'
   - 异步任务完成后 status = 'completed'，RunResult 存在
   - Provider 返回错误时 status = 'failed'，error_message 非空
   - 多模型并行执行，各自独立成功/失败

#### 前端测试（Vitest + React Testing Library）

6. **ProviderSettings 页面**
   - 渲染 Provider 列表
   - 创建表单提交流程
   - 模型添加/删除交互

7. **TestCasePanel 组件**
   - 渲染测试用例列表
   - 新建/编辑/删除流程
   - 选中状态切换

8. **RunPanel 组件**
   - 模型多选交互
   - Run 按钮禁用/启用状态

9. **RunResultsView 组件**
   - 渲染历史结果
   - 状态徽章颜色正确
   - 流式内容拼接显示

---

## 安全注意事项

1. **API Key 永不明文存储**：数据库中只存加密后的 `encrypted_api_key`
2. **API Key 永不明文返回**：所有 API 响应使用 `ProviderResponse`（含 `api_key_masked`），不含加密值
3. **生产环境必须设置 `ENCRYPTION_KEY`**：默认全零密钥仅用于开发
4. **加密密钥轮换**：当前实现不支持密钥轮换，生产部署前应评估
5. **API Key 输入**：前端使用 `type="password"` 输入框
6. **CORS**：`tower-http` 配置的 CORS 中间件限制跨域请求

---

## 已知限制与后续改进方向

| 限制 | 说明 | 计划解决 |
|------|------|----------|
| 仅支持单轮对话 | 当前 request 只有 system + 1 条 user message | M4/M5 可扩展 |
| 无 tool_call 支持 | 请求体中未包含 tools 定义 | M4 断言引擎需要 |
| 流式结果未持久化 | `stream_run` 的流式输出不写入 run_results | 依赖 `complete()` 的异步任务 |
| 无请求超时 | `reqwest::Client` 未设置超时 | 应添加 timeout 配置 |
| 无速率限制 | 未限制并发请求数 | 生产环境需添加 |
| capabilities 未实际使用 | 模型能力矩阵仅存储，未用于功能开关 | M4+ 使用 |
| config 未实际使用 | 测试用例的 config JSON 未传递给 LLM 请求 | 应传递 max_tokens/temperature |

---

## 文件清单

### 后端

| 文件 | 说明 |
|------|------|
| `backend/migrations/003_m3_providers_testcases_runs.sql` | M3 数据库迁移脚本 |
| `backend/src/crypto.rs` | AES-256-GCM 加密/解密/脱敏 |
| `backend/src/llm/mod.rs` | LlmProvider trait + 工厂函数 |
| `backend/src/llm/openai.rs` | OpenAI 适配器（complete + stream） |
| `backend/src/llm/anthropic.rs` | Anthropic 适配器（complete + stream） |
| `backend/src/models/provider.rs` | Provider 数据模型 |
| `backend/src/models/model.rs` | AiModel 数据模型 |
| `backend/src/models/test_case.rs` | TestCase 数据模型 |
| `backend/src/models/run.rs` | Run / RunResult / RunWithResult 数据模型 |
| `backend/src/routes/providers.rs` | Provider CRUD 路由 |
| `backend/src/routes/models_routes.rs` | Model CRUD 路由 |
| `backend/src/routes/test_cases.rs` | TestCase CRUD 路由 |
| `backend/src/routes/runs.rs` | 执行引擎 + SSE 流式输出路由 |

### 前端

| 文件 | 说明 |
|------|------|
| `frontend/src/pages/ProviderSettings.tsx` | Provider/Model 配置管理页面 |
| `frontend/src/components/TestCasePanel.tsx` | 测试用例 CRUD 面板 |
| `frontend/src/components/RunPanel.tsx` | 模型选择 + 运行触发面板 |
| `frontend/src/components/RunResultsView.tsx` | 运行结果展示 + SSE 流式显示 |
| `frontend/src/api/providers.ts` | Provider/Model API 客户端 |
| `frontend/src/api/testCases.ts` | TestCase/Run API 客户端 |
| `frontend/src/api/sse.ts` | SSE 流式客户端工具 |

---

## 依赖关系

- **依赖 M2**：分层 Prompt 合成引擎（四层合并 + 变量注入）、项目管理功能
- **被 M4 依赖**：执行引擎、Run/RunResult 存储（断言引擎在此基础上判定）
- **被 M5 依赖**：执行引擎（Trace 在 Provider 层拦截记录调用链路）
- **被 M6 依赖**：执行数据（报告系统聚合 Run/RunResult 生成报告）
