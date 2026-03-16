# System Prompt 学习与测试平台 (SP-Magic)

这是一个面向 "学习 System Prompt" 的端到端 Web 平台。用户可以通过 **编写 Prompt → 运行测试 → 观察结果 → 迭代优化** 的闭环流程，深入理解和掌握 Prompt 编写技巧。

## 🌟 核心特性

- **分层 Prompt 管理**：支持 全局 → 项目 → 提供商 → 模型 四层组合逻辑。
- **版本控制与对比**：自动保存 Prompt 版本历史，支持实时预览合成效果及其版本间的 diff 对比。
- **多模型并行测试**：一键将 Prompt 发送至多个 AI 模型（OpenAI, Anthropic 等），并排对比输出结果。
- **自动化验证（断言引擎）**：支持 `must_call`, `keyword_present` 等多种规则，自动判定测试结果。
- **Fixture 与基线**：支持录制真实交互为 Fixture 进行确定性回放，并支持基线对比监测模型行为漂移。
- **调用链路追踪 (Trace)**：可视化展示从 Prompt 发送到最终输出的完整 Timeline，包含中间的 Tool Call 与结果。
- **工程化支持**：提供运行报告、成本估算，并支持通过 API 触发批量任务，方便集成至 CI 流程。

## 🛠️ 技术栈

- **前端**: React 19 + TypeScript + Vite
  - 样式: Tailwind CSS 4
  - 编辑器: Monaco Editor
  - 状态管理: TanStack Query (React Query)
  - 图表: Recharts
- **后端**: Rust + Axum + Tokio
  - 数据库: SQLite + SQLx (支持自动化迁移)
  - 加密: AES-256-GCM (保护 API Key)
  - 网络: Reqwest (支持 SSE 流式响应)
- **文档**: Markdown + Mermaid (里程碑与架构说明)

## 📂 项目结构

```text
system-prompt-test/
├── frontend/          # React 前端项目
│   ├── src/
│   │   ├── api/       # API 请求分装
│   │   ├── components/# 可复用组件
│   │   └── pages/     # 页面逻辑
├── backend/           # Rust 后端项目
│   ├── src/
│   │   ├── routes/    # 路由处理
│   │   ├── models/    # 数据模型与逻辑
│   │   └── db/        # 数据库与迁移
├── docs/              # 项目开发文档与里程碑
└── AGENTS.md          # 仓库贡献指南与开发规范
```

## 🚀 快速开始

### 1. 前端启动

```bash
cd frontend
npm install
npm run dev
```
访问地址: `http://localhost:5173`

### 2. 后端启动

确保系统已安装 Rust 环境 (MSRV 1.75+)。

```bash
cd backend
# 首次运行会自动创建 SQLite 数据库并运行迁移
cargo run
```
API 地址: `http://localhost:3000`

### 3. 配置环境变量

在 `backend` 目录下创建或修改 `.env` 文件（如果需要）：

```properties
DATABASE_URL=sqlite:data.db
ENCRYPTION_KEY=你的32位加密密钥
```

## 📈 路线图 (Milestones)

- [x] **M1**: 项目骨架搭建与基础 CRUD
- [x] **M2**: 分层编辑与版本管理
- [x] **M3**: 多模型配置与在线执行
- [x] **M4**: 断言引擎与 Fixture 回放
- [ ] **M5**: 调用链路追踪与可视化 (进行中)
- [ ] **M6**: 报告系统、CI 集成与安全加固

详细进度的分级说明请参考 [docs/milestones.md](docs/milestones.md)。

## 🤝 贡献指南

请参考 [AGENTS.md](AGENTS.md) 了解代码风格、目录规范及提交流程。
