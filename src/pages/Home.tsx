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
  ListOrdered,
} from 'lucide-react';

interface HomeProps {
  onNavigate: (page: PageType) => void;
}

interface ToolCard {
  id: PageType;
  title: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  bgColor: string;
}

const editTools: ToolCard[] = [
  {
    id: 'merge',
    title: 'PDF 合并',
    description: '将多个 PDF 文件合并为一个',
    icon: <Merge size={24} />,
    color: 'text-orange-400',
    bgColor: 'bg-orange-500/10',
  },
  {
    id: 'split',
    title: 'PDF 分割',
    description: '将 PDF 分割为多个文件',
    icon: <Scissors size={24} />,
    color: 'text-red-400',
    bgColor: 'bg-red-500/10',
  },
  {
    id: 'delete',
    title: '删除页面',
    description: '从 PDF 中删除指定页面',
    icon: <Trash2 size={24} />,
    color: 'text-rose-400',
    bgColor: 'bg-rose-500/10',
  },
  {
    id: 'extract',
    title: '提取页面',
    description: '从 PDF 中提取指定页面',
    icon: <FileOutput size={24} />,
    color: 'text-amber-400',
    bgColor: 'bg-amber-500/10',
  },
  {
    id: 'rotate',
    title: '旋转页面',
    description: '旋转 PDF 页面方向',
    icon: <RotateCw size={24} />,
    color: 'text-blue-400',
    bgColor: 'bg-blue-500/10',
  },
  {
    id: 'reorder',
    title: '页面重排',
    description: '调整 PDF 页面的先后顺序',
    icon: <ListOrdered size={24} />,
    color: 'text-cyan-400',
    bgColor: 'bg-cyan-500/10',
  },
];

const convertTools: ToolCard[] = [
  {
    id: 'compress',
    title: 'PDF 压缩',
    description: '减小 PDF 文件体积',
    icon: <Minimize2 size={24} />,
    color: 'text-green-400',
    bgColor: 'bg-green-500/10',
  },
  {
    id: 'images-to-pdf',
    title: '图片转 PDF',
    description: 'JPEG、PNG 转换为 PDF',
    icon: <FileImage size={24} />,
    color: 'text-purple-400',
    bgColor: 'bg-purple-500/10',
  },
  {
    id: 'pdf-to-images',
    title: 'PDF 转图片',
    description: 'PDF 转换为 JPEG、PNG',
    icon: <Image size={24} />,
    color: 'text-pink-400',
    bgColor: 'bg-pink-500/10',
  },
];

export default function Home({ onNavigate }: HomeProps) {
  const renderToolCard = (tool: ToolCard) => (
    <button
      key={tool.id}
      onClick={() => onNavigate(tool.id)}
      className="group p-5 rounded-2xl border transition-all duration-300 text-left bg-[#1a1a21] border-[#2e2e38] hover:border-[#3e3e48] hover:bg-[#22222b] cursor-pointer"
    >
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
          专业的 PDF 处理工具，支持 macOS、Windows 和 Linux
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

      {/* Convert Tools */}
      <section className="mb-10">
        <div className="flex items-center gap-3 mb-5">
          <div className="w-1 h-6 bg-green-500 rounded-full" />
          <h2 className="text-xl font-semibold text-white">转换与优化</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 stagger-children">
          {convertTools.map(renderToolCard)}
        </div>
      </section>
    </div>
  );
}
