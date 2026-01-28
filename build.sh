#!/bin/bash

# PDF Toolkit 构建脚本
# 用于构建 macOS 和 Windows 版本

set -e

echo "开始构建 PDF Toolkit..."

# 构建 macOS 版本
echo ""
echo "=========================================="
echo "构建 macOS 版本 (ARM64)"
echo "=========================================="
npm run tauri build -- --target aarch64-apple-darwin

# 构建 Windows 版本（如果当前是 macOS，需要交叉编译工具链）
echo ""
echo "=========================================="
echo "构建 Windows 版本 (x64)"
echo "=========================================="
# 注意：Windows 构建需要在 Windows 系统上，或者使用交叉编译
# 如果当前是 macOS，可以尝试：
# rustup target add x86_64-pc-windows-msvc
# npm run tauri build -- --target x86_64-pc-windows-msvc
echo "Windows 版本需要在 Windows 系统上构建，或配置交叉编译工具链"

echo ""
echo "=========================================="
echo "构建完成！"
echo "=========================================="
echo ""
echo "构建产物位置："
echo "  macOS: src-tauri/target/aarch64-apple-darwin/release/bundle/"
echo "  Windows: src-tauri/target/x86_64-pc-windows-msvc/release/bundle/"
echo ""
echo "如果 PDFium 库未自动包含，请手动将库文件复制到应用包中："
echo "  macOS: .app/Contents/MacOS/ 或 .app/Contents/Frameworks/"
echo "  Windows: 应用目录/"
echo ""
