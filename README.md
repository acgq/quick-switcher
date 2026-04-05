# Quick Switcher

跨平台窗口快速切换工具，类似 Alfred/Raycast/Listary。

## 功能特性

- **全局快捷键呼出**：默认 `Alt + Ctrl + Space`，可自定义
- **实时搜索过滤**：支持窗口标题和进程名搜索
- **拼音搜索**：支持中文窗口名的拼音首字母和全拼搜索
- **键盘导航**：
  - `↑` / `↓` 或 `Ctrl+P` / `Ctrl+N` 选择窗口
  - `Enter` 切换到选中的窗口
  - `Esc` 关闭窗口
- **多显示器支持**：窗口显示在鼠标所在的屏幕上
- **系统托盘**：右键托盘图标可显示窗口、打开设置、退出程序
- **窗口状态保持**：切换窗口时保持最大化/普通状态

## 技术栈

- **后端**：Rust + Tauri 2
- **前端**：React + TypeScript + Vite
- **样式**：CSS

## 平台支持

| 平台 | 显示服务器 | 窗口列表 | 窗口切换 |
|------|-----------|---------|---------|
| Windows | - | ✅ Win32 API | ✅ Win32 API |
| macOS | - | ✅ NSWorkspace + AXUIElement | ✅ NSWorkspace |
| Linux | X11 | ✅ X11 EWMH | ✅ X11 EWMH |
| Linux | Wayland (KDE) | ✅ kdotool | ✅ kdotool |
| Linux | Wayland (GNOME) | ⚠️ 部分 XWayland | ⚠️ 部分 XWayland |
| Linux | Wayland (Sway/Hyprland) | ⚠️ 部分 XWayland | ⚠️ 部分 XWayland |

## 项目结构

```
├── src/                    # 前端源码
│   ├── App.tsx            # 主窗口组件
│   ├── App.css            # 主窗口样式
│   ├── Settings.tsx       # 设置页面组件
│   └── Settings.css       # 设置页面样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── main.rs        # 入口
│   │   ├── lib.rs         # 主逻辑
│   │   └── search.rs      # 搜索匹配模块
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── package.json            # Node 依赖
└── vite.config.ts          # Vite 配置
```

## 开发环境

### 前置要求

- [Node.js](https://nodejs.org/) 20.19+ 或 22.12+
- [Rust](https://www.rust-lang.org/) 1.70+
- [pnpm](https://pnpm.io/) 或 npm

#### Linux 额外要求

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Arch Linux
sudo pacman -S webkit2gtk-4.1 gtk3 libayatana-appindicator librsvg

# Fedora
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```

**Wayland 支持**：对于 KDE Plasma Wayland，需要安装 [kdotool](https://github.com/jinliu/kdotool)：
```bash
# 从 AUR 安装 (Arch Linux)
yay -S kdotool-bin

# 或手动下载
curl -L -o kdotool.tar.gz "https://github.com/jinliu/kdotool/releases/download/v0.2.2/kdotool-0.2.2-x86_64-unknown-linux-gnu.tar.gz"
tar -xzf kdotool.tar.gz
sudo mv kdotool /usr/local/bin/
```

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在首次运行时自动安装
```

### 开发模式

```bash
npm run tauri dev
```

启动后会自动打开开发工具窗口。

### 运行测试

```bash
# Rust 单元测试
cd src-tauri
cargo test
```

## 构建发布

### 开发构建

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### Windows

```bash
npm run tauri build
```

输出文件：
- `src-tauri/target/release/quick-switcher.exe` - 可执行文件
- `src-tauri/target/release/bundle/msi/` - MSI 安装包
- `src-tauri/target/release/bundle/nsis/` - NSIS 安装包

### macOS

```bash
npm run tauri build
```

输出文件：
- `src-tauri/target/release/bundle/dmg/` - DMG 安装包
- `src-tauri/target/release/bundle/macos/` - .app 应用包

### Linux

```bash
npm run tauri build
```

输出文件：
- `src-tauri/target/release/bundle/deb/` - DEB 包（Debian/Ubuntu）
- `src-tauri/target/release/bundle/appimage/` - AppImage（通用）
- `src-tauri/target/release/bundle/rpm/` - RPM 包（Fedora/openSUSE）

**注意**：
- X11 环境直接可用
- Wayland 环境需要安装 `kdotool`（见上方 Linux 额外要求）

**Arch Linux 用户**：可以使用 AUR 包：
```bash
yay -S quick-switcher-bin  # 待发布
```

## 配置

### 快捷键设置

1. 右键系统托盘图标
2. 点击 "Settings"
3. 点击输入框，按下想要的组合键
4. 点击 "Save" 保存

支持的修饰键：`Alt`、`Ctrl`、`Shift`、`Win`

配置文件位置：
- Windows: `%APPDATA%\quick-switcher\config.json`
- macOS: `~/Library/Application Support/quick-switcher/config.json`
- Linux: `~/.config/quick-switcher/config.json`

## 快捷键列表

| 快捷键 | 功能 |
|--------|------|
| `Alt + Ctrl + Space` | 呼出/隐藏窗口（默认，可自定义） |
| `↑` / `↓` | 选择窗口 |
| `Ctrl + P` / `Ctrl + N` | 选择窗口（Emacs 风格） |
| `Enter` | 切换到选中的窗口 |
| `Esc` | 关闭窗口 |

## 常见问题

### Linux Wayland 窗口列表为空或切换不工作？

确保已安装 `kdotool`：
```bash
# 检查是否安装
which kdotool

# 测试是否工作
kdotool search --name ".*" --limit 3
```

如果 `kdotool` 不可用，应用会自动回退到 XWayland 模式（只能看到 X11 应用窗口）。

### 多显示器下窗口位置不对

窗口会显示在鼠标当前所在的显示器上，并自动居中。

### 搜索支持哪些方式？

- 直接匹配：输入 "chrome" 匹配 Chrome 窗口
- 拼音全拼：输入 "weixin" 匹配 "微信"
- 拼音首字母：输入 "wx" 匹配 "微信"

## 许可证

MIT License