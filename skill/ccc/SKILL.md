---
name: ccc
description: Claude settings changer — manage multiple Claude profiles, verify credentials, and run commands with specific settings without modifying global config. AI agents use this to self-select configurations.
version: 0.3.0
author: weibaohui
license: MIT
metadata:
  hermes:
    tags: [Claude, Settings, Profile, Config, Multi-Provider, AI-Agent]
    related_skills: [claude-code, atomcode, kimi-cli]
---

# CCC — Claude Settings Changer

`ccc` is a CLI tool for managing multiple Claude `settings.json` profiles. AI agents use it to verify and change between different LLM provider configurations without modifying the global `~/.claude/settings.json`.

## Quick Start

```bash
# Install the ccc skill (installs ccc CLI + Claude skill)
ccc skill install

# List all available profiles
ccc list

# View a profile's configuration (token masked)
ccc view minimax

# Verify a profile works (3-step: JSON parse → fields → real API call)
ccc verify minimax

# Run a task with a specific profile (does NOT modify global settings)
ccc run minimax -p "fix the login bug"

# Batch verify all profiles
ccc batch
```

## Profile Storage

Profiles are stored as `~/.claude/settings.json.<suffix>`:

```
~/.claude/
├── settings.json           # Current active settings
├── settings.json.bak-...   # Auto-backups on apply
├── settings.json.minimax   # MiniMax profile
├── settings.json.zai       # GLM/Z.AI profile
├── settings.json.kimi      # Kimi/Moonshot profile
└── ...
```

## Profile Format

```json
{
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "your-token-here",
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_MODEL": "GLM-5.1",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "GLM-5.1",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "GLM-5.1",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "GLM-5.1",
    "ANTHROPIC_REASONING_MODEL": "GLM-5.1"
  },
  "verbose": true,
  "alwaysThinkingEnabled": true,
  "includeCoAuthoredBy": false,
  "skipDangerousModePermissionPrompt": true,
  "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1
}
```

## Commands

### `ccc list`
Lists all available profile suffixes.

### `ccc view [suffix]`
Displays configuration for a profile (or current settings if no suffix). The `ANTHROPIC_AUTH_TOKEN` is masked as `daac...sbCw` (first 4 + last 4 chars).

### `ccc apply <suffix>`
Backs up the current `settings.json` with timestamp (`settings.json.bak-YYYYmmddHHMMSS`), then applies the specified profile as the new global settings.

**Use case**: Human changing profiles permanently.

### `ccc run <suffix> <command...>`
Runs `claude --settings ~/.claude/settings.json.<suffix> [command]`. Does NOT modify the global `settings.json`.

**Use case**: AI agent selecting a profile for one-off tasks.

```bash
# AI agent workflow
ccc view minimax          # Read config
ccc verify minimax         # Verify credentials
ccc run minimax -p "fix the login bug"  # Run task
```

### `ccc verify <suffix>`
3-step verification:
1. JSON parsing — confirms valid JSON
2. Required fields — checks `ANTHROPIC_AUTH_TOKEN` and `ANTHROPIC_BASE_URL`
3. Real API call — makes an actual request to confirm credentials work

Exit code 0 = passed, 1 = failed.

### `ccc batch [suffix...]`
Batch verifies all profiles (or specified ones) in parallel with real-time progress display:
- `[Ns] Progress: M/M | ✅ N | ❌ N | ⏳ N`
- Summary table with pass/fail status and model info

### `ccc skill install`
Extracts the built-in `ccc` Claude skill to `~/.claude/skills/ccc/`. This makes the `ccc` skill available to Claude/AI agents.

```bash
ccc skill install
# Installs to ~/.claude/skills/ccc/SKILL.md
```

## AI Agent Workflow (Recommended)

AI agents should use `ccc run` to select profiles without touching global config:

```bash
# Step 1: Agent reads available profiles
ccc list

# Step 2: Agent checks profile details
ccc view minimax

# Step 3: Agent verifies before use
ccc verify minimax

# Step 4: Agent runs task with selected profile
ccc run minimax -p "implement the user auth feature"
```

## Adding a New Profile

```bash
# Create a new profile from current settings
cp ~/.claude/settings.json ~/.claude/settings.json.newprovider

# Edit with new credentials
# Then verify it works
ccc verify newprovider
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `ccc: command not found` | Install: `cargo install ccc` or use `ccc skill install` |
| `ccc verify` fails with auth error | Check `ANTHROPIC_AUTH_TOKEN` is correct in the profile |
| `ccc run` doesn't use the right model | Confirm the profile has correct `ANTHROPIC_MODEL` value |
| Skill not showing in Claude | Run `ccc skill install` to extract skill to `~/.claude/skills/` |

## Source

- **Repo**: https://github.com/weibaohui/ccc
- **npm packages**: `@weibaohui/ccc`, `@weibaohui/ccc-linux-x64`, `@weibaohui/ccc-darwin-arm64`, `@weibaohui/ccc-darwin-x64`
