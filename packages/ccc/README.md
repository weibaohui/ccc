# @weibaohui/ccc

Claude settings switcher - 跨平台 npm 包

## 安装

```bash
npm install -g @weibaohui/ccc
```

安装时会自动检测你的平台并安装对应版本。

## 跨平台原理

- 主包 `@weibaohui/ccc` 是一个 wrapper
- 安装后执行 `postinstall` 脚本，自动安装对应平台的 binary 包
- Binary 包：`@weibaohui/ccc-linux-x64`、`@weibaohui/ccc-darwin-arm64`、`@weibaohui/ccc-darwin-x64`

## 支持的平台

- Linux x86_64
- macOS ARM64 (Apple Silicon)
- macOS x86_64
