#!/bin/bash

# 查找 PDFium 库文件的脚本

echo "查找 PDFium 库文件..."
echo ""

# 在 Cargo 缓存中查找
echo "1. 在 Cargo 缓存中查找："
find ~/.cargo -name "*pdfium*.dylib" -o -name "*pdfium*.dll" -o -name "*pdfium*.so" 2>/dev/null | head -10

echo ""
echo "2. 在构建目录中查找："
find src-tauri/target -name "*pdfium*.dylib" -o -name "*pdfium*.dll" -o -name "*pdfium*.so" 2>/dev/null | head -10

echo ""
echo "3. 检查 pdfium-render 的构建输出："
find src-tauri/target -name "*.dylib" -type f 2>/dev/null | grep -i pdfium | head -10

echo ""
echo "4. 检查系统库路径："
ls -la /usr/local/lib/libpdfium* 2>/dev/null || echo "未找到"
ls -la /opt/homebrew/lib/libpdfium* 2>/dev/null || echo "未找到"

echo ""
echo "提示：如果找不到库文件，pdfium-render 的 bindings feature 会在运行时自动下载。"
echo "但为了确保打包时包含，可能需要手动下载并复制库文件。"
