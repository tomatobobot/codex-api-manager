# Codex API Manager

一个基于 Tauri + Vue 3 的桌面工具，用于管理 [OpenAI Codex CLI](https://github.com/openai/codex) 和 [Claude Code](https://claude.ai/code) 的 API 账号配置，支持多账号切换。

## 功能

- 管理多个 Codex / Claude Code API 账号（名称、API Key、Base URL）
- 一键切换账号，自动写入对应配置文件
- 实时展示当前生效账号（侧边栏 badge）
- 账号列表本地持久化，与配置文件互相独立

## 工作原理

| 账号类型 | 写入文件 |
|---------|---------|
| Codex | `~/.codex/auth.json`（API Key）、`~/.codex/config.toml`（Base URL） |
| Claude Code | `~/.claude/settings.json`（`env.ANTHROPIC_AUTH_TOKEN` + `env.ANTHROPIC_BASE_URL`） |

账号列表保存在应用配置目录（由 Tauri 管理），与 Codex / Claude 配置文件彼此独立。**保存账号只更新本地列表，切换账号后才会同步到配置文件。**

## 技术栈

- [Tauri 2](https://tauri.app/) — 跨平台桌面框架
- [Vue 3](https://vuejs.org/) + TypeScript — 前端
- [Vite](https://vitejs.dev/) — 构建工具
- [Tailwind CSS v4](https://tailwindcss.com/) — 样式
- Rust — 后端逻辑（配置文件读写）

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri 系统依赖（见 [Tauri 文档](https://tauri.app/start/prerequisites/)）

### 启动开发环境

```bash
pnpm install
pnpm tauri:dev
```

### 运行测试

```bash
# 前端测试
pnpm test

# Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml
```

### 构建

```bash
pnpm tauri:build
```

## 项目结构

```
src/
  components/
    ProfileManager.vue   # 主界面组件
  services/
    manager.ts           # Tauri 命令封装（前端 API）
  styles.css             # 全局样式与设计 token
src-tauri/
  src/
    lib.rs               # Tauri 命令注册
    codex_manager.rs     # 核心逻辑：账号管理、配置文件读写
```
