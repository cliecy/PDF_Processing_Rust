# PDF Toolkit

一个跨平台的 PDF 处理工具，基于 Tauri 构建，支持 macOS 和 Windows。

## 功能特性

### PDF 编辑
- **PDF 合并** - 将多个 PDF 文件合并为一个
- **PDF 分割** - 将 PDF 按页面范围分割为多个文件
- **删除页面** - 从 PDF 中删除指定页面
- **提取页面** - 从 PDF 中提取指定页面
- **旋转页面** - 旋转 PDF 页面方向（90°、180°、270°）

### PDF 优化
- **PDF 压缩** - 减小 PDF 文件体积
- PDF 修复（即将推出）
- OCR 识别（即将推出）

### 格式转换
- **图片转 PDF** - 将 JPEG、PNG、GIF、WebP 转换为 PDF
- **PDF 转图片** - 将 PDF 页面转换为 PNG 或 JPG
- Word、Excel、PPT 转换（即将推出）

## 技术栈

- **前端**: React + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri v2
- **PDF 处理**: lopdf

## 开发环境要求

- Node.js >= 18
- Rust >= 1.70
- macOS 10.15+ 或 Windows 10+

## 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在构建时自动安装
```

## 开发

```bash
# 启动开发服务器
npm run tauri dev
```

## 构建

```bash
# 构建生产版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

## 项目结构

```
PDF_app/
├── src/                    # 前端源代码
│   ├── components/         # React 组件
│   ├── pages/             # 页面组件
│   ├── hooks/             # React Hooks
│   ├── App.tsx            # 主应用组件
│   └── main.tsx           # 入口文件
├── src-tauri/             # Tauri 后端源代码
│   ├── src/
│   │   ├── lib.rs         # 库入口
│   │   ├── main.rs        # 应用入口
│   │   └── pdf_operations.rs  # PDF 操作实现
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── package.json           # 前端依赖
└── README.md
```

## 许可证

MIT License
