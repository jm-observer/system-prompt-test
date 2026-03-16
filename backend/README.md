# SP-Magic Backend

这是 System Prompt 学习与测试平台的 Rust 后端部分。

## 🚀 启动指南

确保已安装 Rust 1.75+ 环境。

```bash
cd backend
# 复制并根据需要修改环境配置
cp .env.example .env 
cargo run
```

默认 API 地址: [http://localhost:3000](http://localhost:3000)

## 🏗️ 技术栈

- **Axum**: 轻量、高效、基于 Tower 的异步 HTTP 框架。
- **Tokio**: 工业级异步运行时。
- **SQLx**: 具有编译时 SQL 校验功能的异步数据库驱动。
- **SQLite**: 轻量化存储，支持通过 `migrations/` 自动管理 Schema。
- **Serde**: 绝佳的 Rust 序列化/反序列化库。

## 🛠️ 核心模块

- `src/routes`: 路由定义与请求处理器。
- `src/models`: 数据结构定义与业务逻辑。
- `src/db`: 数据库操作、连接池管理及迁移加载。
- `src/middleware`: 通用的安全、审计与请求追踪中间件。
- `src/providers`: 不同 LLM 提供商（OpenAI, Anthropic 等）的适配层。
- `src/assertions`: 断言引擎实现。

## 🔒 安全说明

- **API Key 加密**: 所有存入数据库的 API Key 均进行 AES-256-GCM 加密。
- **内容脱敏**: 支持全链路渲染脱敏，防止 Trace 信息中泄露敏感 Token 或个人信息。

## 📖 相关文档

- 整体项目说明请参考根目录 `README.md`。
- 详细 API 定义与设计思路请参考 `docs/` 目录。
