#!/bin/bash
set -e

# 包信息配置
APP_NAME="betterwx-linux"
VERSION=$(node -p "require('./package.json').version")
ARCH=$(dpkg --print-architecture)
DEB_DIR="dist/deb"
BUILD_DIR="$DEB_DIR/build"
INSTALL_DIR="$BUILD_DIR/usr/lib/$APP_NAME"
BIN_DIR="$BUILD_DIR/usr/bin"
SHARE_DIR="$BUILD_DIR/usr/share"

# 清理旧构建
rm -rf "$DEB_DIR"
mkdir -p "$INSTALL_DIR" "$BIN_DIR"

# 构建应用
npm run build:prod

# 复制Electron构建结果
cp -r dist/linux-unpacked/* "$INSTALL_DIR"

# 创建启动脚本
cat << EOF > "$BIN_DIR/$APP_NAME"
#!/bin/sh
exec /usr/lib/$APP_NAME/$APP_NAME --no-sandbox "\$@"
EOF
chmod +x "$BIN_DIR/$APP_NAME"

# 准备桌面文件
mkdir -p "$SHARE_DIR/applications"
cp assets/linux/betterwx.desktop "$SHARE_DIR/applications/"

# 准备图标资源
ICON_DIR="$SHARE_DIR/icons/hicolor"
for size in 16x16 32x32 48x48 256x256; do
    mkdir -p "$ICON_DIR/$size/apps"
    cp "assets/icons/$size.png" "$ICON_DIR/$size/apps/betterwx.png"
done

# 创建系统集成目录
mkdir -p "$BUILD_DIR/usr/share/dbus-1/services"
cat << EOF > "$BUILD_DIR/usr/share/dbus-1/services/com.betterwx.Linux.service"
[D-BUS Service]
Name=com.betterwx.Linux
Exec=/usr/lib/$APP_NAME/$APP_NAME --dbus
EOF

# 创建control文件
mkdir -p "$BUILD_DIR/DEBIAN"
cat << EOF > "$BUILD_DIR/DEBIAN/control"
Package: $APP_NAME
Version: $VERSION
Section: net
Priority: optional
Architecture: $ARCH
Depends: libappindicator3-1, libgtk-3-0, libnotify4, libxss1, libsecret-1-0, libatomic1
Maintainer: BetterWx Team <team@betterwx.org>
Description: Enhanced WeChat Client for Linux
 Features:
  - Multiple account support
  - Message recall prevention
  - Native Linux integration
Homepage: https://github.com/betterwx-linux
EOF

# 创建postinst脚本
cat << EOF > "$BUILD_DIR/DEBIAN/postinst"
#!/bin/sh
set -e

# 更新图标缓存
if [ -x "\$(command -v update-desktop-database)" ]; then
    update-desktop-database -q /usr/share/applications
fi

# 图标缓存更新
for size in 16 32 48 256; do
    xdg-icon-resource forceupdate --theme hicolor --size \$size
done

exit 0
EOF
chmod +x "$BUILD_DIR/DEBIAN/postinst"

# 构建deb包
dpkg-deb --build "$BUILD_DIR" "$DEB_DIR/${APP_NAME}_${VERSION}_${ARCH}.deb"

echo "Package built: $DEB_DIR/${APP_NAME}_${VERSION}_${ARCH}.deb"