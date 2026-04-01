# PDF Toolkit

一个跨平台的 PDF 处理工具，基于 Tauri 构建，支持 macOS 和 Windows。

## 功能特性

### PDF 编辑
- **PDF 合并** - 将多个 PDF 文件合并为一个
- **PDF 分割** - 将 PDF 按页面范围分割为多个文件
- **删除页面** - 从 PDF 中删除指定页面
- **提取页面** - 从 PDF 中提取指定页面
- **旋转页面** - 旋转 PDF 页面方向（90°、180°、270°）

### 转换与优化
- **PDF 压缩** - 减小 PDF 文件体积
- **图片转 PDF** - 将 JPEG、PNG、GIF、WebP 转换为 PDF
- **PDF 转图片** - 将 PDF 页面转换为 PNG 或 JPG（需要安装 Poppler）

## 系统要求

### 运行要求
- macOS 10.13+ 或 Windows 10+
- PDF 转图片功能需要安装 Poppler：
  - macOS: `brew install poppler`
  - Windows: 下载 Poppler for Windows 并添加到 PATH

### 开发要求
- Node.js >= 18
- Rust >= 1.70

## 安装依赖

```bash
# 安装前端依赖
npm install
```

## 开发

```bash
npm run tauri dev
```

## 构建

### 打包成 DMG（分发给别人）

在 **macOS** 上执行：

```bash
# 1. 安装依赖（首次或更新后）
npm install

# 2. 打包（会生成 .app 和 .dmg）
npm run tauri build
```

打包完成后，DMG 文件位置：

- **Apple Silicon (M1/M2/M3)**：`src-tauri/target/release/bundle/dmg/PDF Toolkit_1.0.0_aarch64.dmg`
- **Intel Mac**：先执行 `npm run tauri build -- --target x86_64-apple-darwin`，产物在 `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/`

把生成的 **.dmg** 发给对方即可。对方双击 DMG → 把「PDF Toolkit」拖到「应用程序」→ 完成安装。

### 仅构建当前架构

```bash
npm run tauri build
```

构建产物目录：`src-tauri/target/release/bundle/`
- `macos/`：PDF Toolkit.app
- `dmg/`：PDF Toolkit_1.0.0_aarch64.dmg（或 x86_64）

## 技术栈

- **前端**: React + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri v2
- **PDF 处理**: lopdf（纯 Rust）、Poppler（PDF 转图片）

## 许可证

MIT License
