#!/usr/bin/env node

// 跨平台安装脚本（用于主包安装）
const os = require('os');
const { execSync } = require('child_process');

const platform = os.platform();
const arch = os.arch();

// 根据系统和架构确定平台包名
function getPackageName() {
  const platformMap = {
    'linux': { 'x64': 'ccc-linux-x64' },
    'darwin': { 'arm64': 'ccc-darwin-arm64', 'x64': 'ccc-darwin-x64' },
    'win32': { 'x64': 'ccc-windows-x64' }
  };

  const p = platformMap[platform];
  if (!p) {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }

  const a = p[arch] || p[Object.keys(p)[0]];
  if (!a) {
    console.error(`Unsupported architecture: ${arch}`);
    process.exit(1);
  }

  return `@weibaohui/${a}`;
}

const pkg = getPackageName();
console.log(`Installing ${pkg}...`);

try {
  execSync(`npm install ${pkg}`, { stdio: 'inherit' });
  console.log('Installation complete!');
} catch (e) {
  console.error('Installation failed');
  process.exit(1);
}
