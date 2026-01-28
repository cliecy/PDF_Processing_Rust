import { useState } from 'react';
import { FileImage, Plus, GripVertical, Image } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult } from '../hooks/useTauri';

interface ImageFile {
  path: string;
  name: string;
}

export default function ImagesToPdf() {
  const [images, setImages] = useState<ImageFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);
  const { selectImageFiles, imagesToPdf, selectOutputFile } = useTauri();

  const handleAddImages = async () => {
    try {
      const paths = await selectImageFiles();
      if (paths) {
        const newImages = paths.map((path) => ({
          path,
          name: path.split(/[/\\]/).pop() || path,
        }));
        setImages([...images, ...newImages]);
      }
    } catch (error) {
      console.error('Error selecting images:', error);
    }
  };

  const handleRemoveImage = (index: number) => {
    setImages(images.filter((_, i) => i !== index));
  };

  const handleConvert = async () => {
    if (images.length === 0) return;

    try {
      const outputPath = await selectOutputFile('output.pdf');
      if (!outputPath) return;

      setLoading(true);
      const res = await imagesToPdf(
        images.map((img) => img.path),
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
    setImages([]);
    setResult(null);
  };

  const moveImage = (from: number, to: number) => {
    const newImages = [...images];
    const [moved] = newImages.splice(from, 1);
    newImages.splice(to, 0, moved);
    setImages(newImages);
  };

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;
    setDragOverIndex(index);
  };

  const handleDragLeave = () => {
    setDragOverIndex(null);
  };

  const handleDrop = (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex) {
      setDraggedIndex(null);
      setDragOverIndex(null);
      return;
    }

    const newImages = [...images];
    const [moved] = newImages.splice(draggedIndex, 1);
    newImages.splice(dropIndex, 0, moved);
    setImages(newImages);
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="图片转 PDF"
          description="将图片转换为 PDF 文件"
          icon={<FileImage size={28} />}
          iconColor="bg-purple-500/10 text-purple-400"
        />
        <ResultCard
          success={result.success}
          message={result.message}
          outputPath={result.output_path || undefined}
          onReset={handleReset}
        />
      </div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto">
      <PageHeader
        title="图片转 PDF"
        description="将图片转换为 PDF 文件"
        icon={<FileImage size={28} />}
        iconColor="bg-purple-500/10 text-purple-400"
      />

      {/* Image List */}
      <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] overflow-hidden mb-6">
        {images.length === 0 ? (
          <div className="p-12 text-center">
            <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
              <Image className="text-zinc-500" size={28} />
            </div>
            <p className="text-zinc-400 mb-2">还没有添加任何图片</p>
            <p className="text-zinc-600 text-sm">支持 JPG、PNG、GIF、WebP 格式</p>
          </div>
        ) : (
          <div className="divide-y divide-[#2e2e38]">
            {images.map((image, index) => (
              <div
                key={`${image.path}-${index}`}
                draggable
                onDragStart={() => handleDragStart(index)}
                onDragOver={(e) => handleDragOver(e, index)}
                onDragLeave={handleDragLeave}
                onDrop={(e) => handleDrop(e, index)}
                onDragEnd={handleDragEnd}
                className={`flex items-center gap-4 p-4 transition-all group ${
                  draggedIndex === index
                    ? 'opacity-50 scale-95'
                    : dragOverIndex === index
                    ? 'bg-purple-500/10 border-l-2 border-purple-500'
                    : 'hover:bg-[#22222b]'
                }`}
              >
                <div className="text-zinc-600 hover:text-purple-400 cursor-grab active:cursor-grabbing transition-colors">
                  <GripVertical size={18} />
                </div>
                <span className="w-8 h-8 rounded-lg bg-purple-500/10 text-purple-400 flex items-center justify-center text-sm font-medium">
                  {index + 1}
                </span>
                <div className="w-12 h-12 rounded-lg bg-[#22222b] flex items-center justify-center overflow-hidden">
                  <Image className="text-zinc-500" size={20} />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-white truncate">{image.name}</p>
                  <p className="text-xs text-zinc-600 truncate">{image.path}</p>
                </div>
                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  {index > 0 && (
                    <button
                      onClick={() => moveImage(index, index - 1)}
                      className="p-2 rounded-lg hover:bg-[#2e2e38] text-zinc-400 hover:text-white"
                      title="上移"
                    >
                      ↑
                    </button>
                  )}
                  {index < images.length - 1 && (
                    <button
                      onClick={() => moveImage(index, index + 1)}
                      className="p-2 rounded-lg hover:bg-[#2e2e38] text-zinc-400 hover:text-white"
                      title="下移"
                    >
                      ↓
                    </button>
                  )}
                  <button
                    onClick={() => handleRemoveImage(index)}
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
          onClick={handleAddImages}
          icon={<Plus size={18} />}
        >
          添加图片
        </Button>
        <Button
          variant="primary"
          onClick={handleConvert}
          loading={loading}
          disabled={images.length === 0}
          icon={<FileImage size={18} />}
        >
          转换为 PDF
        </Button>
      </div>
    </div>
  );
}
