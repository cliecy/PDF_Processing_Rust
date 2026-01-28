# PDF Toolkit 构建说明

## 构建 macOS 版本

```bash
# 构建 ARM64 (Apple Silicon) 版本
npm run tauri build -- --target aarch64-apple-darwin

# 构建 x86_64 (Intel) 版本（如果需要）
npm run tauri build -- --target x86_64-apple-darwin
```

构建产物位于：`src-tauri/target/aarch64-apple-darwin/release/bundle/`

## 构建 Windows 版本

**方法 1：在 Windows 系统上构建**
```bash
npm run tauri build
```

**方法 2：在 macOS 上交叉编译（需要配置）**
```bash
# 安装 Windows 目标
rustup target add x86_64-pc-windows-msvc

# 构建
npm run tauri build -- --target x86_64-pc-windows-msvc
```

构建产物位于：`src-tauri/target/x86_64-pc-windows-msvc/release/bundle/`

## 手动添加 PDFium 库文件

如果 PDFium 库未自动包含在应用包中，需要手动添加：

### macOS

1. 找到 PDFium 库文件（通常在构建时下载）：
   ```bash
   # 查找 PDFium 库文件
   find ~/.cargo -name "libpdfium.dylib" 2>/dev/null
   find target -name "libpdfium.dylib" 2>/dev/null
   ```

2. 复制到应用包：
   ```bash
   # 假设应用包路径为 PDF Toolkit.app
   cp libpdfium.dylib "PDF Toolkit.app/Contents/MacOS/"
   # 或者
   cp libpdfium.dylib "PDF Toolkit.app/Contents/Frameworks/"
   ```

### Windows

1. 找到 PDFium 库文件：
   ```bash
   # 查找 PDFium DLL 文件
   find target -name "pdfium.dll" 2>/dev/null
   ```

2. 复制到应用目录：
   ```bash
   # 复制到应用可执行文件所在目录
   cp pdfium.dll "应用目录/"
   ```

## 验证库文件是否包含

### macOS
```bash
# 检查应用包内容
ls -la "PDF Toolkit.app/Contents/MacOS/"
ls -la "PDF Toolkit.app/Contents/Frameworks/"
```

### Windows
```bash
# 检查应用目录
dir "应用目录"
```

## 注意事项

- PDFium 库文件大小约为 10-20 MB
- 确保库文件与可执行文件的架构匹配（ARM64 vs x86_64）
- 库文件需要与可执行文件在同一目录或系统库搜索路径中
