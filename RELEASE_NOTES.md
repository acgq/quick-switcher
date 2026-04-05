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
- **AppImage**: 下载 `.AppImage` 文件，`chmod +x && ./xxx.AppImage`

#### Windows
下载 MSI 或 NSIS 安装包，双击安装

#### macOS
下载 DMG 文件，拖拽到 Applications

### 支持架构

| 平台 | 架构 |
|------|------|
| Linux | x86_64, aarch64 |
| Windows | x86_64, aarch64 |
| macOS (Intel) | x86_64 |
| macOS (Apple Silicon) | aarch64 |