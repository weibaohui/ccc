# ccc - Claude Settings Switcher / Claude 配置切换工具

[![CI](https://github.com/weibaohui/ccc/actions/workflows/ci.yml/badge.svg)](https://github.com/weibaohui/ccc/actions)
[![npm version](https://img.shields.io/npm/v/@weibaohui/ccc.svg)](https://www.npmjs.com/package/@weibaohui/ccc)

> English below | 中文说明见下方

---

## English

### Overview

`ccc` (Claude Config Changer) is a CLI tool for switching between multiple Claude Code configuration profiles. It stores each profile as a separate `settings.json.<suffix>` file and makes switching as simple as running one command — with automatic backup before every switch.

### Install

```bash
npm install -g @weibaohui/ccc
```

The postinstall script automatically detects your platform and installs the correct binary package (`@weibaohui/ccc-linux-x64`, `@weibaohui/ccc-darwin-arm64`, or `@weibaohui/ccc-darwin-x64`).

Or install from source:

```bash
git clone https://github.com/weibaohui/ccc.git
cd ccc
cargo build --release
cp target/release/ccc ~/bin/ccc
```

### Usage

| Command | Description |
|---|---|
| `ccc list` | List all available profiles |
| `ccc view [suffix]` | View profile details (default: current config) |
| `ccc apply <suffix>` | Switch to the specified profile (auto-backup) |

#### List all profiles

```bash
$ ccc list
a7m
atm.6
kimi
minimax
xiaomo
zai
```

#### View current config

```bash
$ ccc view
Current settings.json
──────────────────────────────────────────────────
  env:
    ANTHROPIC_AUTH_TOKEN: daac...X0u
    ANTHROPIC_BASE_URL: https://open.bigmodel.cn/api/anthropic
    ANTHROPIC_MODEL: GLM-5.1
    ANTHROPIC_REASONING_MODEL: GLM-5.1
  verbose: true
  alwaysThinkingEnabled: true
  enabledPlugins: 1
    warp@claude-code-warp: true
```

#### View a specific profile

```bash
$ ccc view zai
Profile: settings.json.zai
──────────────────────────────────────────────────
  env:
    ANTHROPIC_AUTH_TOKEN: daac...X0u
    ANTHROPIC_BASE_URL: https://open.bigmodel.cn/api/anthropic
    ANTHROPIC_MODEL: GLM-5.1
  verbose: true
  alwaysThinkingEnabled: true
```

#### Switch profile

```bash
$ ccc apply zai
Backing up current settings to settings.json.bak-20260502180851...
Applying profile 'zai'...
Done! Backup: settings.json.bak-20260502180851, Applied: settings.json.zai
```

### How It Works

- Profile files are stored as `~/.claude/settings.json.<suffix>`
- Before switching, the current config is automatically backed up as `settings.json.bak-YYYYmmddHHMMSS`
- Uses `fs::copy` — original file is never modified, only a backup is created

### File Structure

```
~/.claude/
├── settings.json             # Currently active config
├── settings.json.a7m         # Profile copies
├── settings.json.minimax
├── settings.json.zai
└── settings.json.bak-*       # Auto-backups (not listed by 'ccc list')
```

---

## 中文

### 简介

`ccc`（Claude Config Changer）是一款 Claude Code 配置文件切换 CLI 工具。每个配置存储为独立的 `settings.json.<suffix>` 文件，切换只需一条命令，并自动备份。

### 安装

```bash
npm install -g @weibaohui/ccc
```

安装脚本会自动检测平台，安装对应的二进制包（`@weibaohui/ccc-linux-x64`、`@weibaohui/ccc-darwin-arm64` 或 `@weibaohui/ccc-darwin-x64`）。

或从源码安装：

```bash
git clone https://github.com/weibaohui/ccc.git
cd ccc
cargo build --release
cp target/release/ccc ~/bin/ccc
```

### 使用方法

| 命令 | 说明 |
|---|---|
| `ccc list` | 列出所有可用配置 |
| `ccc view [suffix]` | 查看配置详情（默认查看当前配置） |
| `ccc apply <suffix>` | 切换到指定配置（自动备份） |

#### 列出所有配置

```bash
$ ccc list
a7m
atm.6
kimi
minimax
xiaomo
zai
```

#### 查看当前配置

```bash
$ ccc view
Current settings.json
──────────────────────────────────────────────────
  env:
    ANTHROPIC_AUTH_TOKEN: daac...X0u
    ANTHROPIC_BASE_URL: https://open.bigmodel.cn/api/anthropic
    ANTHROPIC_MODEL: GLM-5.1
    ANTHROPIC_REASONING_MODEL: GLM-5.1
  verbose: true
  alwaysThinkingEnabled: true
  enabledPlugins: 1
    warp@claude-code-warp: true
```

#### 查看指定配置

```bash
$ ccc view zai
Profile: settings.json.zai
──────────────────────────────────────────────────
  env:
    ANTHROPIC_AUTH_TOKEN: daac...X0u
    ANTHROPIC_BASE_URL: https://open.bigmodel.cn/api/anthropic
    ANTHROPIC_MODEL: GLM-5.1
  verbose: true
  alwaysThinkingEnabled: true
```

#### 切换配置

```bash
$ ccc apply zai
Backing up current settings to settings.json.bak-20260502180851...
Applying profile 'zai'...
Done! Backup: settings.json.bak-20260502180851, Applied: settings.json.zai
```

### 工作原理

- 配置文件存储在 `~/.claude/settings.json.<suffix>`
- 切换前自动备份当前配置为 `settings.json.bak-YYYYmmddHHMMSS`
- 使用 `fs::copy` — 原文件不会被修改，只新增备份文件

### 文件结构

```
~/.claude/
├── settings.json             # 当前生效的配置
├── settings.json.a7m          # 各配置副本
├── settings.json.minimax
├── settings.json.zai
└── settings.json.bak-*       # 自动备份（不会被 list 列出）
```

---

## Packages / 包结构

This is a monorepo containing the following npm packages:

| Package | Platform | Description |
|---|---|---|
| `@weibaohui/ccc` | Any | Wrapper package with postinstall |
| `@weibaohui/ccc-linux-x64` | Linux x64 | Linux binary |
| `@weibaohui/ccc-darwin-arm64` | macOS ARM64 (Apple Silicon) | macOS binary |
| `@weibaohui/ccc-darwin-x64` | macOS x64 | macOS binary |

For end users, just install `@weibaohui/ccc` — the correct platform package is pulled in automatically.

## License / 许可证

MIT
