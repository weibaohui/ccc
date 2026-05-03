# ccc - Claude Config Changer

> 给 AI 用的配置变更工具 · Config changer for AI agents

[![CI](https://github.com/weibaohui/ccc/actions/workflows/ci.yml/badge.svg)](https://github.com/weibaohui/ccc/actions)
[![npm version](https://img.shields.io/npm/v/@weibaohui/ccc.svg)](https://www.npmjs.com/package/@weibaohui/ccc)

---

## 核心理念 · Core Idea

**人类做决策，AI 去执行。**

当 AI 在执行任务时，如果需要切换自己的 Claude Code 配置（比如换模型、换 API 地址、换凭证），不需要人类介入，AI 直接调用 `ccc` 自行完成。

```
人: "帮我用 deepseek 跑这个任务"
AI: ccc apply deepseek → 自动切换 → 开始执行
```

这是一个 **AI 向 AI** 传递配置的方式 —— 人类只需要告诉 AI 用什么配置，AI 自己完成切换。

---

## English

### What is this?

`ccc` (Claude Config Changer) is a CLI tool designed for **AI agents** to change their own Claude Code configuration at runtime. Unlike GUI-based config changers meant for humans, `ccc` is designed to be called programmatically — by another AI, a script, or any automation system.

When an AI agent needs to change its model, API endpoint, or credentials mid-session, it simply runs `ccc apply <profile>` and the change happens automatically with a backup.

### Key philosophy

> **Humans decide, AI executes.**

Humans tell AI which profile to use. The AI calls `ccc` to change to it. No human involvement required for the actual change.

### Install

```bash
npm install -g @weibaohui/ccc
```

### Usage

| Command | Description |
|---|---|
| `ccc list` | List all available profiles |
| `ccc view [suffix]` | View profile details (default: current config) |
| `ccc apply <suffix>` | Change to the specified profile (auto-backup) |
| `ccc run <suffix> <cmd...>` | Run a command with a profile (does not modify global settings) |
| `ccc verify <suffix>` | Verify a profile (JSON parse → required fields → real API call) |
| `ccc batch [suffix...]` | Batch verify all profiles with real-time progress |
| `ccc skill install` | Install the embedded CCC skill to `~/.claude/skills/` |

### Example workflow

```bash
# Human tells AI to use the "deepseek" profile
$ ccc apply deepseek
Backing up current settings to settings.json.bak-20260502180851...
Applying profile 'deepseek'...
Done! Backup: settings.json.bak-20260502180851, Applied: settings.json.deepseek

# AI now runs with the new config automatically
```

### AI Workflow (recommended for agents)

```bash
# Agent reads config first
$ ccc view minimax

# Agent verifies before use (3-step: JSON → fields → API call)
$ ccc verify minimax
[1/2] JSON parsing: OK
[2/2] Required fields: OK (model=MiniMax-M2.7-highspeed)
[3/3] Making API call to verify credentials...
[3/3] API call: OK
Verification PASSED ✅

# Agent runs with that profile (no global modification)
$ ccc run minimax -p "fix the login bug"
# Claude uses minimax profile for this command only

# Batch verify all profiles at once
$ ccc batch
Batch verify: 8 profile(s)
────────────────────────────────────────
  [8s] Progress: 8/8 | ✅ 5 | ❌ 3 | ⏳ 0
════════════════════════════════════════════════════════════
  BATCH VERIFY RESULTS  (8 profiles)
────────────────────────────────────────────────────────────
  minimax              ✅ PASS          model=MiniMax-M2.7-highspeed
  zai                  ✅ PASS          model=GLM-5.1
  kimi                 ❌ FAIL          API error: Failed to authenticate.
════════════════════════════════════════════════════════════
```

### How it works

- Profiles are stored as `~/.claude/settings.json.<suffix>`
- Before every change, the current config is automatically backed up as `settings.json.bak-YYYYmmddHHMMSS`
- Uses `fs::copy` — original file is never modified, only a backup is created

### File structure

```
~/.claude/
├── settings.json              # Currently active config
├── settings.json.deepseek      # AI can change to any of these
├── settings.json.claude
├── settings.json.zai
└── settings.json.bak-*         # Auto-backups (not listed by 'ccc list')
```

---

## 中文

### 这是什么？

`ccc`（Claude Config Changer）是一个专为 **AI Agent** 设计的配置切换工具。在 AI 执行任务的过程中，如果需要切换自己的 Claude Code 配置（如换模型、换 API 地址、换凭证），无需人类介入，AI 直接调用 `ccc apply <profile>` 自动完成。

### 核心理念

> **人类做决策，AI 去执行。**

人类只需告诉 AI 使用哪个配置，AI 自己调用 `ccc` 完成切换。整个过程 AI 自主导，无需人类插手。

### 安装

```bash
npm install -g @weibaohui/ccc
```

### 使用方法

| 命令 | 说明 |
|---|---|
| `ccc list` | 列出所有可用配置 |
| `ccc view [suffix]` | 查看配置详情（默认查看当前配置） |
| `ccc apply <suffix>` | 切换到指定配置（自动备份） |
| `ccc run <suffix> <cmd...>` | 使用指定配置运行命令（不修改全局配置） |
| `ccc verify <suffix>` | 验证配置可用性（JSON解析 → 必填字段 → 真实API调用） |
| `ccc batch [suffix...]` | 批量验证所有配置（实时进度显示） |
| `ccc skill install` | 将内置 CCC skill 安装到 `~/.claude/skills/` |

### 典型工作流

```bash
# 人类告诉 AI 用 deepseek 配置跑任务
$ ccc apply deepseek
Backing up current settings to settings.json.bak-20260502180851...
Applying profile 'deepseek'...
Done! Backup: settings.json.bak-20260502180851, Applied: settings.json.deepseek

# AI 自动以新配置继续执行任务
```

### 工作原理

- 配置以 `~/.claude/settings.json.<suffix>` 文件形式存储
- 每次切换前自动备份当前配置为 `settings.json.bak-YYYYmmddHHMMSS`
- 使用 `fs::copy` — 原文件不会被修改，只新增备份文件

### 文件结构

```
~/.claude/
├── settings.json              # 当前生效的配置
├── settings.json.deepseek      # AI 可以切换到任意一个
├── settings.json.claude
├── settings.json.zai
└── settings.json.bak-*         # 自动备份（不会被 list 列出）
```

---

## Packages

| Package | Platform |
|---|---|
| `@weibaohui/ccc` | Any (wrapper) |
| `@weibaohui/ccc-linux-x64` | Linux x64 |
| `@weibaohui/ccc-darwin-arm64` | macOS ARM64 |
| `@weibaohui/ccc-darwin-x64` | macOS x64 |

安装 `@weibaohui/ccc` 即可，安装脚本自动根据平台拉取对应二进制包。

## License

MIT
