import { PageType } from '../App';
import {
  Home,
  Merge,
  Scissors,
  Trash2,
  FileOutput,
  Minimize2,
  RotateCw,
  Image,
  FileImage,
  Settings,
  FileText,
} from 'lucide-react';

interface SidebarProps {
  currentPage: PageType;
  onNavigate: (page: PageType) => void;
}

interface NavItem {
  id: PageType;
  label: string;
  icon: React.ReactNode;
  color: string;
}

const navItems: NavItem[] = [
  { id: 'home', label: '首页', icon: <Home size={20} />, color: 'text-white' },
  { id: 'merge', label: 'PDF 合并', icon: <Merge size={20} />, color: 'text-orange-400' },
  { id: 'split', label: 'PDF 分割', icon: <Scissors size={20} />, color: 'text-red-400' },
  { id: 'delete', label: '删除页面', icon: <Trash2 size={20} />, color: 'text-rose-400' },
  { id: 'extract', label: '提取页面', icon: <FileOutput size={20} />, color: 'text-amber-400' },
  { id: 'compress', label: 'PDF 压缩', icon: <Minimize2 size={20} />, color: 'text-green-400' },
  { id: 'rotate', label: '旋转页面', icon: <RotateCw size={20} />, color: 'text-blue-400' },
  { id: 'images-to-pdf', label: '图片转 PDF', icon: <FileImage size={20} />, color: 'text-purple-400' },
  { id: 'pdf-to-images', label: 'PDF 转图片', icon: <Image size={20} />, color: 'text-pink-400' },
];

export default function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-64 bg-[#16161c] border-r border-[#2e2e38] flex flex-col">
      {/* Logo */}
      <div className="p-6 border-b border-[#2e2e38]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-orange-500 to-red-500 flex items-center justify-center">
            <FileText className="text-white" size={22} />
          </div>
          <div>
            <h1 className="font-semibold text-lg text-white">PDF Toolkit</h1>
            <p className="text-xs text-zinc-500">专业 PDF 工具箱</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 overflow-y-auto">
        <div className="space-y-1 stagger-children">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 group ${
                currentPage === item.id
                  ? 'bg-[#22222b] text-white'
                  : 'text-zinc-400 hover:bg-[#1e1e26] hover:text-white'
              }`}
            >
              <span
                className={`${item.color} transition-transform duration-200 group-hover:scale-110`}
              >
                {item.icon}
              </span>
              <span className="text-sm font-medium">{item.label}</span>
              {currentPage === item.id && (
                <div className="ml-auto w-1.5 h-1.5 rounded-full bg-orange-500" />
              )}
            </button>
          ))}
        </div>
      </nav>

      {/* Settings */}
      <div className="p-4 border-t border-[#2e2e38]">
        <button className="w-full flex items-center gap-3 px-4 py-3 rounded-xl text-zinc-400 hover:bg-[#1e1e26] hover:text-white transition-all duration-200">
          <Settings size={20} />
          <span className="text-sm font-medium">设置</span>
        </button>
      </div>
    </aside>
  );
}
