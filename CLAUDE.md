# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

跨平台窗口快速切换工具，类似 Alfred/Raycast/Listary，支持：
- 全局快捷键 `Alt + Ctrl + Space` 呼出
- 实时搜索过滤窗口
- 拼音搜索（首字母 + 全拼）
- 跨平台（Windows / macOS / Linux）

## 技术栈

- **后端**: Rust + Tauri 2
- **前端**: React 19 + TypeScript + Vite
- **样式**: CSS（无框架）
- **拼音**: pinyin crate（Rust）

## 常用命令

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build

# 运行前端开发服务器
npm run dev

# Rust 检查
cargo check
cargo clippy

# Rust 测试
cargo test
```

## 架构

```
src-tauri/
├── src/
│   ├── main.rs           # 入口，调用 lib.rs
│   ├── lib.rs            # 主逻辑 + 平台模块（platform 模块内含 Windows/macOS/Linux 实现）
│   └── search.rs         # 搜索匹配（拼音、模糊匹配、评分）

src/                      # 前端
├── App.tsx               # 主窗口组件（窗口列表、搜索、键盘导航）
├── App.css               # 主窗口样式
├── Settings.tsx          # 设置页面（快捷键配置）
└── Settings.css          # 设置页面样式
```

### 关键设计

- **平台模块**: `lib.rs` 内嵌 `platform` 模块，通过 `#[cfg(target_os)]` 条件编译实现跨平台
- **搜索逻辑**: `search.rs` 支持直接匹配、拼音全拼、拼音首字母、模糊匹配，按评分排序
- **前端通信**: 通过 Tauri `invoke` 调用后端命令（`get_windows`, `search_windows`, `switch_window`）

## 代码风格

- 实用主义，解决实际问题，不写假想需求的代码
- 函数短小精悍，只做一件事
- 语言：用中文表达，代码注释用英文