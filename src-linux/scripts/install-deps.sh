#!/bin/bash
# 安装构建依赖
sudo apt-get install -y \
    libappindicator3-dev \
    libgtk-3-dev \
    libnotify-dev \
    libdbus-1-dev \
    nodejs \
    npm \
    rpm \
    fakeroot

# 安装运行时依赖
sudo apt-get install -y \
    libxss1 \
    libappindicator3-1 \
    libsecret-1-0