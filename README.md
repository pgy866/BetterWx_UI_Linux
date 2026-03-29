# Better WeChat for Linux
本项目来自分支 https://gitee.com/ChaoYouAi/better-wx-linux-ui

<h3 align="center"><img src="https://raw.githubusercontent.com/afaa1991/BetterWx-UI/refs/heads/1.1.3/src-tauri/icons/128x128@2x.png" width="250px"></h3>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows-green">
  <img src="https://img.shields.io/github/stars/afaa1991/BetterWx-UI">
</p>


wx Linux版 支持4.0.2 双开&防撤回&多账号共存 UI工具
支持平台：linux x64

版本支持：4.0+

根据大佬 Zetaloop 开源 制作的 ui 工具

大佬开源地址 [https://github.com/zetaloop/BetterWX](https://github.com/zetaloop/BetterWX)

UI工具开源地址 [https://github.com/afaa1991/BetterWx-UI](https://github.com/afaa1991/BetterWx-UI)

[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://makeapullrequest.com)

适用于Linux系统的增强版微信客户端，基于Electron和Web技术构建，提供更友好的Linux桌面集成体验。


## ✨ 功能特性

- **多账号支持**  
  通过独立容器实现真正的多开，不同账号完全隔离
- **消息防撤回**  
  实时拦截撤回请求，保留完整聊天记录
- **原生集成**  
  - 系统托盘支持（支持GNOME/KDE等主流DE）
  - DBus接口调用
  - 符合Linux桌面规范的通知系统
- **增强体验**  
  - 全局快捷键（Ctrl+Alt+W唤起窗口）
  - 自定义主题支持
  - 消息加密存储
- **跨架构支持**  
  支持x86_64/ARM64/RISCV架构

## 📥 安装

### 预编译包
```bash
# Ubuntu/Debian
wget https://example.com/betterwx_1.0.0_amd64.deb
sudo apt install ./betterwx_1.0.0_amd64.deb

# Fedora/CentOS
sudo dnf install https://example.com/betterwx-1.0.0.x86_64.rpm

# AppImage
chmod +x betterwx-linux-x86_64.AppImage
./betterwx-linux-x86_64.AppImage
```

### 从源码构建
```bash
git clone https://github.com/your-org/betterwx-linux
cd betterwx-linux

# 安装依赖
npm install

# 开发模式
npm run dev

# 生产构建
npm run build
```

## 🛠️ 使用指南

### 多账号管理
```bash
betterwx --profile=work     # 工作账号
betterwx --profile=personal # 私人账号
```

### 快捷键
| 快捷键          | 功能                |
|-----------------|---------------------|
| Ctrl+Shift+N    | 创建新实例          |
| Ctrl+Alt+W      | 聚焦主窗口          |
| Ctrl+Shift+Del  | 清除当前会话缓存    |

### 配置文件位置
`~/.config/BetterWx/config.json`

## 🧩 插件系统

通过扩展接口实现功能增强：
```javascript
// plugins/anti-recall.js
module.exports = {
  onMessage: (msg) => {
    if (msg.isRevoke) {
      logRecalledMessage(msg)
      return false // 阻止撤回
    }
  }
}
```

## 🛡️ 安全性

- 使用Electron沙箱机制隔离渲染进程
- 消息存储采用AES-256-GCM加密
- 支持硬件安全模块（HSM）集成
- 自动安全更新机制

## 🤝 参与贡献

### 分支策略
- `main`：稳定版本
- `dev`：开发分支
- `feat/*`：功能开发分支
- `fix/*`：问题修复分支

### 代码规范
1. 遵循Electron安全实践
2. 所有API调用需通过preload脚本
3. 重要功能需要包含单元测试
4. 提交信息格式遵循[Conventional Commits](https://www.conventionalcommits.org)

### 开发工作流
```bash
# 创建新功能分支
git checkout -b feat/awesome-feature

# 提交变更
git commit -s -m "feat: 添加神奇的新功能"

# 推送并创建PR
git push origin feat/awesome-feature
```

