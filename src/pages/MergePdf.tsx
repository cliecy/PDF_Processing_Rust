import { useRef, useState } from 'react';
import { Merge, Plus, GripVertical, Trash2 } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult } from '../hooks/useTauri';

interface FileItem {
  path: string;
  name: string;
}

export default function MergePdf() {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const [dragEnabledIndex, setDragEnabledIndex] = useState<number | null>(null);
  const dragEnabledRef = useRef<number | null>(null);
  const { selectPdfFiles, mergePdfs, selectOutputFile, openFolder, copyToClipboard } = useTauri();

  const handleAddFiles = async () => {
    try {
      const paths = await selectPdfFiles(true);
      if (paths) {
        const newFiles = paths.map((path) => ({
          path,
          name: path.split(/[/\\]/).pop() || path,
        }));
        setFiles([...files, ...newFiles]);
      }
    } catch (error) {
      console.error('Error selecting files:', error);
    }
  };

  const handleRemoveFile = (index: number) => {
    setFiles(files.filter((_, i) => i !== index));
  };

  const handleMerge = async () => {
    if (files.length < 2) return;

    try {
      const outputPath = await selectOutputFile('merged.pdf');
      if (!outputPath) return;

      setLoading(true);
      const res = await mergePdfs(
        files.map((f) => f.path),
        outputPath
      );
      setResult(res);
    } catch (error) {
      setResult({
        success: false,
        message: String(error),
        output_path: null,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setFiles([]);
    setResult(null);
  };

  const moveFile = (from: number, to: number) => {
    const newFiles = [...files];
    const [moved] = newFiles.splice(from, 1);
    newFiles.splice(to, 0, moved);
    setFiles(newFiles);
  };

  const handleDragHandlePointerDown = (index: number) => {
    dragEnabledRef.current = index;
    setDragEnabledIndex(index);
  };

  const handleDragStart = (e: React.DragEvent<HTMLDivElement>, index: number) => {
    if (dragEnabledRef.current !== index) {
      e.preventDefault();
      return;
    }

    setDraggedIndex(index);
    setDragOverIndex(index);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.dropEffect = 'move';
    e.dataTransfer.setData('text/plain', String(index));
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    if (draggedIndex === null || draggedIndex === index) return;
    setDragOverIndex(index);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    const nextTarget = e.relatedTarget;
    if (nextTarget instanceof Node && e.currentTarget.contains(nextTarget)) {
      return;
    }

    setDragOverIndex((current) => (current === draggedIndex ? current : null));
  };

  const handleDrop = (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex) {
      setDraggedIndex(null);
      setDragOverIndex(null);
      return;
    }

    const newFiles = [...files];
    const [moved] = newFiles.splice(draggedIndex, 1);
    newFiles.splice(dropIndex, 0, moved);
    setFiles(newFiles);
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
    setDragOverIndex(null);
    setDragEnabledIndex(null);
    dragEnabledRef.current = null;
  };

  const handleClearAll = () => {
    setFiles([]);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="PDF 合并"
          description="将多个 PDF 文件合并为一个"
          icon={<Merge size={28} />}
          iconColor="bg-orange-500/10 text-orange-400"
        />
        <ResultCard
          success={result.success}
          message={result.message}
          outputPath={result.output_path || undefined}
          onReset={handleReset}
          onOpenFolder={result.output_path ? () => openFolder(result.output_path!) : undefined}
          onCopyPath={result.output_path ? () => copyToClipboard(result.output_path!) : undefined}
        />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto">
      <PageHeader
        title="PDF 合并"
        description="将多个 PDF 文件合并为一个"
        icon={<Merge size={28} />}
        iconColor="bg-orange-500/10 text-orange-400"
      />

      {/* File List */}
      <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] overflow-hidden mb-6">
        {files.length > 0 && (
          <div className="flex items-center justify-between px-4 py-3 border-b border-[#2e2e38]">
            <span className="text-sm text-zinc-400">
              已添加 <span className="text-white font-medium">{files.length}</span> 个文件
            </span>
            <button
              onClick={handleClearAll}
              className="text-xs text-zinc-500 hover:text-red-400 transition-colors flex items-center gap-1"
            >
              <Trash2 size={12} />
              清空全部
            </button>
          </div>
        )}
        {files.length === 0 ? (
          <div className="p-12 text-center">
            <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
              <Merge className="text-zinc-500" size={28} />
            </div>
            <p className="text-zinc-400 mb-2">还没有添加任何文件</p>
            <p className="text-zinc-600 text-sm">点击下方按钮选择 PDF 文件</p>
          </div>
        ) : (
          <div className="divide-y divide-[#2e2e38]">
            {files.map((file, index) => (
              <div
                key={`${file.path}-${index}`}
                draggable
                onDragStart={(e) => handleDragStart(e, index)}
                onDragOver={(e) => handleDragOver(e, index)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, index)}
                onDragEnd={handleDragEnd}
                className={`flex items-center gap-4 p-4 transition-all group ${
                  draggedIndex === index
                    ? 'opacity-50 scale-95'
                    : dragOverIndex === index
                    ? 'bg-orange-500/10 border-l-2 border-orange-500'
                    : 'hover:bg-[#22222b]'
                }`}
              >
                <button
                  type="button"
                  onPointerDown={() => handleDragHandlePointerDown(index)}
                  onPointerUp={() => {
                    if (draggedIndex === null) {
                      setDragEnabledIndex(null);
                      dragEnabledRef.current = null;
                    }
                  }}
                  onPointerCancel={() => {
                    setDragEnabledIndex(null);
                    dragEnabledRef.current = null;
                  }}
                  className={`text-zinc-600 transition-colors ${
                    dragEnabledIndex === index
                      ? 'text-orange-400 cursor-grabbing'
                      : 'hover:text-orange-400 cursor-grab active:cursor-grabbing'
                  }`}
                  title="拖动调整顺序"
                >
                  <GripVertical size={18} />
                </button>
                <span className="w-8 h-8 rounded-lg bg-orange-500/10 text-orange-400 flex items-center justify-center text-sm font-medium">
                  {index + 1}
                </span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-white truncate">{file.name}</p>
                  <p className="text-xs text-zinc-600 truncate">{file.path}</p>
                </div>
                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  {index > 0 && (
                    <button
                      onClick={() => moveFile(index, index - 1)}
                      className="p-2 rounded-lg hover:bg-[#2e2e38] text-zinc-400 hover:text-white"
                      title="上移"
                    >
                      ↑
                    </button>
                  )}
                  {index < files.length - 1 && (
                    <button
                      onClick={() => moveFile(index, index + 1)}
                      className="p-2 rounded-lg hover:bg-[#2e2e38] text-zinc-400 hover:text-white"
                      title="下移"
                    >
                      ↓
                    </button>
                  )}
                  <button
                    onClick={() => handleRemoveFile(index)}
                    className="p-2 rounded-lg hover:bg-red-500/10 text-zinc-400 hover:text-red-400"
                    title="删除"
                  >
                    ✕
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex gap-4">
        <Button
          variant="secondary"
          onClick={handleAddFiles}
          icon={<Plus size={18} />}
        >
          添加 PDF 文件
        </Button>
        <Button
          variant="primary"
          onClick={handleMerge}
          loading={loading}
          disabled={files.length < 2}
          icon={<Merge size={18} />}
        >
          合并 PDF
        </Button>
      </div>

      {files.length > 0 && files.length < 2 && (
        <p className="text-amber-500 text-sm mt-4">
          请至少添加 2 个 PDF 文件才能进行合并
        </p>
      )}
    </div>
  );
}
