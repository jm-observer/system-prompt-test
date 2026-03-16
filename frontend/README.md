# SP-Magic Frontend

这是 System Prompt 学习与测试平台的 React 前端部分。

## 🚀 开发环境启动

确保已进入 `frontend` 目录：

```bash
npm install
npm run dev
```

默认访问地址: [http://localhost:5173](http://localhost:5173)

## 🏗️ 技术架构

- **React 19**: 最新的 React 特性支持。
- **TypeScript**: 静态类型保证。
- **Tailwind CSS 4**: 高性能原子化 CSS 框架。
- **Monaco Editor**: 提供类似 VS Code 的 Prompt 编辑体验，支持 Diff 模式。
- **TanStack Query**: 优雅的异步状态管理与缓存。
- **Recharts**: 运行报告与性能趋势可视化。

## 📂 目录规范

- `src/api`: 基于 Fetch/Axum 的 API 抽象与类型定义。
- `src/components`: 跨页面复用的 UI 组件（如编辑器封装、瀑布图）。
- `src/pages`: 业务页面（Prompt 列表、配置中心、运行报告）。
- `src/hooks`: 业务逻辑抽离的自定义 Hooks。

## 📖 相关文档

- 整体项目说明请参考根目录 `README.md`。
- 详细开发指南请参考 `docs/milestones.md`。
