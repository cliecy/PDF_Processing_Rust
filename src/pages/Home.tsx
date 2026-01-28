import { PageType } from '../App';
import {
  Merge,
  Scissors,
  Trash2,
  FileOutput,
  Minimize2,
  RotateCw,
  FileImage,
  Image,
  Wrench,
  ScanLine,
  FileType,
  FileSpreadsheet,
  Presentation,
  Globe,
} from 'lucide-react';

interface HomeProps {
  onNavigate: (page: PageType) => void;
}

interface ToolCard {
  id: PageType | string;
  title: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  bgColor: string;
  available: boolean;
}

const editTools: ToolCard[] = [
  {
    id: 'merge',
    title: 'PDF 合并',
    description: '将多个 PDF 文件合并为一个',
    icon: <Merge size={24} />,
    color: 'text-orange-400',
    bgColor: 'bg-orange-500/10',
    available: true,
  },
  {
    id: 'split',
    title: 'PDF 分割',
    description: '将 PDF 分割为多个文件',
    icon: <Scissors size={24} />,
    color: 'text-red-400',
    bgColor: 'bg-red-500/10',
    available: true,
  },
  {
    id: 'delete',
    title: '删除页面',
    description: '从 PDF 中删除指定页面',
    icon: <Trash2 size={24} />,
    color: 'text-rose-400',
    bgColor: 'bg-rose-500/10',
    available: true,
  },
  {
    id: 'extract',
    title: '提取页面',
    description: '从 PDF 中提取指定页面',
    icon: <FileOutput size={24} />,
    color: 'text-amber-400',
    bgColor: 'bg-amber-500/10',
    available: true,
  },
  {
    id: 'rotate',
    title: '旋转页面',
    description: '旋转 PDF 页面方向',
    icon: <RotateCw size={24} />,
    color: 'text-blue-400',
    bgColor: 'bg-blue-500/10',
    available: true,
  },
  {
    id: 'organize',
    title: 'PDF 编排',
    description: '重新排序 PDF 页面',
    icon: <Wrench size={24} />,
    color: 'text-purple-400',
    bgColor: 'bg-purple-500/10',
    available: false,
  },
];

const optimizeTools: ToolCard[] = [
  {
    id: 'compress',
    title: 'PDF 压缩',
    description: '减小 PDF 文件体积',
    icon: <Minimize2 size={24} />,
    color: 'text-green-400',
    bgColor: 'bg-green-500/10',
    available: true,
  },
  {
    id: 'repair',
    title: 'PDF 修复',
    description: '修复损坏的 PDF 文件',
    icon: <Wrench size={24} />,
    color: 'text-yellow-400',
    bgColor: 'bg-yellow-500/10',
    available: false,
  },
  {
    id: 'ocr',
    title: 'OCR 识别',
    description: '识别 PDF 中的文字',
    icon: <ScanLine size={24} />,
    color: 'text-cyan-400',
    bgColor: 'bg-cyan-500/10',
    available: false,
  },
];

const convertToPdfTools: ToolCard[] = [
  {
    id: 'images-to-pdf',
    title: '图片转 PDF',
    description: 'JPEG、PNG 转换为 PDF',
    icon: <FileImage size={24} />,
    color: 'text-purple-400',
    bgColor: 'bg-purple-500/10',
    available: true,
  },
  {
    id: 'word-to-pdf',
    title: 'Word 转 PDF',
    description: 'DOC、DOCX 转换为 PDF',
    icon: <FileType size={24} />,
    color: 'text-blue-500',
    bgColor: 'bg-blue-500/10',
    available: false,
  },
  {
    id: 'excel-to-pdf',
    title: 'Excel 转 PDF',
    description: 'XLS、XLSX 转换为 PDF',
    icon: <FileSpreadsheet size={24} />,
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    available: false,
  },
  {
    id: 'ppt-to-pdf',
    title: 'PPT 转 PDF',
    description: 'PPT、PPTX 转换为 PDF',
    icon: <Presentation size={24} />,
    color: 'text-orange-500',
    bgColor: 'bg-orange-500/10',
    available: false,
  },
  {
    id: 'html-to-pdf',
    title: 'HTML 转 PDF',
    description: '网页转换为 PDF',
    icon: <Globe size={24} />,
    color: 'text-teal-400',
    bgColor: 'bg-teal-500/10',
    available: false,
  },
];

const convertFromPdfTools: ToolCard[] = [
  {
    id: 'pdf-to-images',
    title: 'PDF 转图片',
    description: 'PDF 转换为 JPEG、PNG',
    icon: <Image size={24} />,
    color: 'text-pink-400',
    bgColor: 'bg-pink-500/10',
    available: true,
  },
  {
    id: 'pdf-to-word',
    title: 'PDF 转 Word',
    description: 'PDF 转换为 DOC、DOCX',
    icon: <FileType size={24} />,
    color: 'text-blue-500',
    bgColor: 'bg-blue-500/10',
    available: false,
  },
  {
    id: 'pdf-to-excel',
    title: 'PDF 转 Excel',
    description: 'PDF 转换为 XLS、XLSX',
    icon: <FileSpreadsheet size={24} />,
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    available: false,
  },
  {
    id: 'pdf-to-ppt',
    title: 'PDF 转 PPT',
    description: 'PDF 转换为 PPT、PPTX',
    icon: <Presentation size={24} />,
    color: 'text-orange-500',
    bgColor: 'bg-orange-500/10',
    available: false,
  },
];

export default function Home({ onNavigate }: HomeProps) {
  const renderToolCard = (tool: ToolCard) => (
    <button
      key={tool.id}
      onClick={() => tool.available && onNavigate(tool.id as PageType)}
      disabled={!tool.available}
      className={`group relative p-5 rounded-2xl border transition-all duration-300 text-left ${
        tool.available
          ? 'bg-[#1a1a21] border-[#2e2e38] hover:border-[#3e3e48] hover:bg-[#22222b] cursor-pointer'
          : 'bg-[#16161c] border-[#22222b] opacity-60 cursor-not-allowed'
      }`}
    >
      {!tool.available && (
        <span className="absolute top-3 right-3 text-[10px] px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-500">
          即将推出
        </span>
      )}
      <div
        className={`w-12 h-12 rounded-xl ${tool.bgColor} flex items-center justify-center mb-4 transition-transform duration-300 group-hover:scale-110`}
      >
        <span className={tool.color}>{tool.icon}</span>
      </div>
      <h3 className="font-semibold text-white mb-1">{tool.title}</h3>
      <p className="text-sm text-zinc-500">{tool.description}</p>
    </button>
  );

  return (
    <div className="max-w-6xl mx-auto">
      {/* Header */}
      <div className="mb-10">
        <h1 className="text-4xl font-bold mb-3">
          <span className="gradient-text">PDF Toolkit</span>
        </h1>
        <p className="text-zinc-400 text-lg">
          专业的 PDF 处理工具，支持 macOS 和 Windows
        </p>
      </div>

      {/* Edit Tools */}
      <section className="mb-10">
        <div className="flex items-center gap-3 mb-5">
          <div className="w-1 h-6 bg-orange-500 rounded-full" />
          <h2 className="text-xl font-semibold text-white">PDF 编辑</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
          {editTools.map(renderToolCard)}
        </div>
      </section>

      {/* Optimize Tools */}
      <section className="mb-10">
        <div className="flex items-center gap-3 mb-5">
          <div className="w-1 h-6 bg-green-500 rounded-full" />
          <h2 className="text-xl font-semibold text-white">PDF 优化</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
          {optimizeTools.map(renderToolCard)}
        </div>
      </section>

      {/* Convert to PDF */}
      <section className="mb-10">
        <div className="flex items-center gap-3 mb-5">
          <div className="w-1 h-6 bg-blue-500 rounded-full" />
          <h2 className="text-xl font-semibold text-white">转换为 PDF</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
          {convertToPdfTools.map(renderToolCard)}
        </div>
      </section>

      {/* Convert from PDF */}
      <section className="mb-10">
        <div className="flex items-center gap-3 mb-5">
          <div className="w-1 h-6 bg-purple-500 rounded-full" />
          <h2 className="text-xl font-semibold text-white">PDF 转换</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
          {convertFromPdfTools.map(renderToolCard)}
        </div>
      </section>
    </div>
  );
}
