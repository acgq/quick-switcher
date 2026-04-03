# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

跨平台窗口快速切换工具，类似 Alfred/Raycast/Listary，支持：
- 全局快捷键 `Alt + Ctrl + Space` 呼出
- 实时搜索过滤窗口
- 拼音搜索（首字母 + 全拼）
- 跨平台（Windows / macOS / Linux）

## 技术栈

- **后端**: Rust + Tauri
- **前端**: React + TypeScript
- **样式**: TailwindCSS
- **状态管理**: Zustand

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
src/
├── tauri/                # Rust 后端
│   ├── main.rs
│   ├── window_manager/  # 窗口管理模块（跨平台）
│   ├── hotkey/          # 全局快捷键
│   ├── search/          # 搜索与匹配
│   └── models/          # 数据结构
│
├── frontend/            # 前端 UI
│   ├── components/
│   ├── hooks/
│   ├── store/
│   └── pages/
```

## 代码风格

- 实用主义，解决实际问题，不写假想需求的代码
- 函数短小精悍，只做一件事
- 写完代码顺便写测试
- 语言：用中文表达，代码注释用英文

## 开发阶段

1. **Phase 1**: 快捷键呼出 + 窗口列表 + 点击切换
2. **Phase 2**: 输入过滤 + 键盘操作
3. **Phase 3**: 拼音搜索 + 模糊匹配 + LRU 排序
4. **Phase 4**: UI 动画 + 历史权重