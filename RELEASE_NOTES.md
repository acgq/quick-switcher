## Quick Switcher

跨平台窗口快速切换工具，类似 Alfred/Raycast/Listary。

### 功能特性
- **全局快捷键呼出**：默认 `Alt + Ctrl + Space`，可自定义
- **实时搜索过滤**：支持窗口标题和进程名搜索
- **拼音搜索**：支持中文窗口名的拼音首字母和全拼搜索
- **多显示器支持**：窗口显示在鼠标所在的屏幕上
- **KDE Wayland 内置支持**：无需额外依赖

### 安装方式

#### Linux
- **Arch Linux (AUR)**: `yay -S quick-switcher-bin`
- **Debian/Ubuntu**: 下载 `.deb` 包，`sudo apt install ./xxx.deb`
- **Fedora/openSUSE**: 下载 `.rpm` 包，`sudo rpm -i ./xxx.rpm`
- **通用**: 下载 `.AppImage` 文件，直接运行

#### Windows
下载 MSI 或 NSIS 安装包，双击安装

#### macOS
下载 DMG 文件，拖拽到 Applications

### 平台支持

| 平台 | 显示服务器 | 状态 |
|------|-----------|------|
| Linux | X11 | ✅ |
| Linux | KDE Plasma 6 Wayland | ✅ 内置支持 |
| Linux | GNOME/Sway Wayland | ⚠️ 仅 XWayland |
| Windows | - | ✅ |
| macOS | - | ✅ |