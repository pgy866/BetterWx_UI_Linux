#!/bin/bash
# 生成AppImage格式安装包
npm run build -- --linux AppImage

# 添加桌面文件集成
install -Dm644 assets/linux/betterwx.desktop \
    dist/BetterWx-Linux.AppDir/usr/share/applications/betterwx.desktop

# 生成更新信息
ARCH=x86_64 ./appimagetool-x86_64.AppImage dist/BetterWx-Linux.AppDir