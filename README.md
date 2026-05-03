# ccc - Claude Settings Switcher

Claude Code 配置文件切换工具，在多个配置之间轻松切换。

## 功能

- `ccc list` - 列出所有可用配置
- `ccc view [suffix]` - 查看配置详情（默认查看当前配置）
- `ccc apply <suffix>` - 切换到指定配置（自动备份）

## 安装

```bash
cargo build --release
cp target/release/ccc ~/bin/ccc
```

## 使用示例

### 列出所有配置

```bash
$ ccc list
a7m
atm.6
kimi
minimax
xiaomo
zai
```

### 查看当前配置

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

### 查看指定配置

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

### 切换配置

```bash
$ ccc apply zai
Backing up current settings to settings.json.bak-20260502180851...
Applying profile 'zai'...
Done! Backup: settings.json.bak-20260502180851, Applied: settings.json.zai
```

## 工作原理

- 配置文件存储在 `~/.claude/settings.json.*`
- 切换前自动备份当前配置为 `settings.json.bak-YYYYmmddHHMMSS`
- 使用 `fs::copy` 确保原文件不被修改，只新增备份文件

## 文件结构

```
~/.claude/
├── settings.json          # 当前生效的配置
├── settings.json.a7m      # 各配置副本
├── settings.json.minimax
├── settings.json.zai
└── settings.json.bak-*    # 自动备份（不会被 list 列出）
```
