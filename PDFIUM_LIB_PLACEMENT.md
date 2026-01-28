# PDFium 库文件放置位置

## macOS 应用包结构

构建后的 macOS 应用包位于：
```
src-tauri/target/release/bundle/macos/PDF Toolkit.app/
```

应用包内部结构：
```
PDF Toolkit.app/
└── Contents/
    ├── Info.plist
    ├── MacOS/
    │   └── pdf-toolkit          # 主可执行文件
    └── Frameworks/              # 动态库目录（如果存在）
```

## PDFium 库文件放置位置

### 方法 1：放在 MacOS 目录（推荐）

将 `libpdfium.dylib` 文件复制到：
```
PDF Toolkit.app/Contents/MacOS/libpdfium.dylib
```

这样可执行文件和库文件在同一目录，运行时最容易找到。

### 方法 2：放在 Frameworks 目录

如果 `Frameworks` 目录不存在，先创建：
```bash
mkdir -p "PDF Toolkit.app/Contents/Frameworks"
```

然后将库文件复制到：
```
PDF Toolkit.app/Contents/Frameworks/libpdfium.dylib
```

## Windows 应用结构

构建后的 Windows 应用位于：
```
src-tauri/target/release/bundle/msi/PDF Toolkit/
```

或者：
```
src-tauri/target/release/bundle/nsis/PDF Toolkit/
```

## Windows PDFium 库文件放置位置

将 `pdfium.dll` 文件复制到应用可执行文件所在的目录：
```
PDF Toolkit/
├── pdf-toolkit.exe      # 主可执行文件
└── pdfium.dll           # PDFium 库文件（放在这里）
```

## 如何获取 PDFium 库文件

### macOS

1. **从 pdfium-render 的构建缓存中查找**：
   ```bash
   # 查找已下载的库文件
   find ~/.cargo -name "libpdfium.dylib" 2>/dev/null
   ```

2. **手动下载**：
   - 访问 PDFium 的 GitHub releases
   - 下载对应架构的预编译库（ARM64 或 x86_64）
   - 解压后找到 `libpdfium.dylib`

### Windows

1. **从构建缓存中查找**：
   ```bash
   find target -name "pdfium.dll" 2>/dev/null
   ```

2. **手动下载**：
   - 访问 PDFium 的 GitHub releases
   - 下载 Windows 版本的预编译库
   - 解压后找到 `pdfium.dll`

## 验证库文件是否正确放置

### macOS
```bash
# 检查库文件是否存在
ls -la "PDF Toolkit.app/Contents/MacOS/libpdfium.dylib"

# 检查库文件的架构是否匹配
file "PDF Toolkit.app/Contents/MacOS/libpdfium.dylib"
file "PDF Toolkit.app/Contents/MacOS/pdf-toolkit"
```

### Windows
```bash
# 检查库文件是否存在
dir "PDF Toolkit\pdfium.dll"
```

## 注意事项

1. **架构匹配**：确保库文件的架构与可执行文件匹配
   - macOS ARM64 应用需要 ARM64 库
   - macOS x86_64 应用需要 x86_64 库
   - Windows x64 应用需要 x64 DLL

2. **文件权限**：确保库文件有执行权限（macOS）
   ```bash
   chmod +x "PDF Toolkit.app/Contents/MacOS/libpdfium.dylib"
   ```

3. **库文件大小**：PDFium 库文件通常为 10-20 MB
