import { useState } from 'react';
import { RotateCw, FileText, RotateCcw } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult, PdfInfo } from '../hooks/useTauri';

export default function RotatePdf() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [selectedPages, setSelectedPages] = useState<Set<number>>(new Set());
  const [rotation, setRotation] = useState<90 | 180 | 270>(90);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const { selectPdfFiles, getPdfInfo, rotatePages, selectOutputFile } = useTauri();

  const handleSelectFile = async () => {
    try {
      const paths = await selectPdfFiles(false);
      if (paths && paths.length > 0) {
        const path = paths[0];
        setFilePath(path);
        setFileName(path.split(/[/\\]/).pop() || path);
        const info = await getPdfInfo(path);
        setPdfInfo(info);
        // Select all pages by default
        setSelectedPages(
          new Set(Array.from({ length: info.page_count }, (_, i) => i + 1))
        );
      }
    } catch (error) {
      console.error('Error selecting file:', error);
    }
  };

  const togglePage = (page: number) => {
    const newSelected = new Set(selectedPages);
    if (newSelected.has(page)) {
      newSelected.delete(page);
    } else {
      newSelected.add(page);
    }
    setSelectedPages(newSelected);
  };

  const selectAll = () => {
    setSelectedPages(
      new Set(Array.from({ length: pdfInfo?.page_count || 0 }, (_, i) => i + 1))
    );
  };

  const handleRotate = async () => {
    if (!filePath || selectedPages.size === 0) return;

    try {
      const outputPath = await selectOutputFile('rotated.pdf');
      if (!outputPath) return;

      setLoading(true);
      const res = await rotatePages(
        filePath,
        Array.from(selectedPages),
        rotation,
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
    setFilePath(null);
    setFileName('');
    setPdfInfo(null);
    setSelectedPages(new Set());
    setRotation(90);
    setResult(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="旋转页面"
          description="旋转 PDF 页面方向"
          icon={<RotateCw size={28} />}
          iconColor="bg-blue-500/10 text-blue-400"
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
        title="旋转页面"
        description="旋转 PDF 页面方向"
        icon={<RotateCw size={28} />}
        iconColor="bg-blue-500/10 text-blue-400"
      />

      {!filePath ? (
        <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-12 text-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
            <FileText className="text-zinc-500" size={28} />
          </div>
          <p className="text-zinc-400 mb-4">选择要处理的 PDF 文件</p>
          <Button variant="primary" onClick={handleSelectFile}>
            选择 PDF 文件
          </Button>
        </div>
      ) : (
        <>
          {/* Selected File Info */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-4 mb-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center">
                <FileText className="text-blue-400" size={24} />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-white font-medium truncate">{fileName}</p>
                <p className="text-sm text-zinc-500">
                  共 {pdfInfo?.page_count} 页 ·{' '}
                  {((pdfInfo?.file_size || 0) / 1024).toFixed(1)} KB
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={handleSelectFile}>
                更换文件
              </Button>
            </div>
          </div>

          {/* Rotation Options */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-6 mb-6">
            <h3 className="text-white font-medium mb-4">旋转角度</h3>
            <div className="flex gap-3">
              {[90, 180, 270].map((angle) => (
                <button
                  key={angle}
                  onClick={() => setRotation(angle as 90 | 180 | 270)}
                  className={`flex-1 flex flex-col items-center gap-2 p-4 rounded-xl transition-all ${
                    rotation === angle
                      ? 'bg-blue-500/20 border border-blue-500/40'
                      : 'bg-[#22222b] border border-transparent hover:border-[#3e3e48]'
                  }`}
                >
                  {angle === 90 && <RotateCw className="text-blue-400" size={24} />}
                  {angle === 180 && <RotateCw className="text-blue-400" size={24} />}
                  {angle === 270 && <RotateCcw className="text-blue-400" size={24} />}
                  <span
                    className={`text-sm font-medium ${
                      rotation === angle ? 'text-blue-400' : 'text-zinc-400'
                    }`}
                  >
                    {angle}°
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Page Selection */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-white font-medium">选择要旋转的页面</h3>
              <Button variant="ghost" size="sm" onClick={selectAll}>
                全选
              </Button>
            </div>
            <div className="grid grid-cols-8 gap-2">
              {Array.from({ length: pdfInfo?.page_count || 0 }, (_, i) => i + 1).map(
                (page) => (
                  <button
                    key={page}
                    onClick={() => togglePage(page)}
                    className={`aspect-square rounded-lg flex items-center justify-center text-sm font-medium transition-all ${
                      selectedPages.has(page)
                        ? 'bg-blue-500 text-white'
                        : 'bg-[#22222b] text-zinc-400 hover:bg-[#2a2a35] hover:text-white'
                    }`}
                  >
                    {page}
                  </button>
                )
              )}
            </div>
            <p className="text-sm text-zinc-500 mt-3">
              已选择 {selectedPages.size} 页
            </p>
          </div>

          {/* Action */}
          <Button
            variant="primary"
            onClick={handleRotate}
            loading={loading}
            disabled={selectedPages.size === 0}
            icon={<RotateCw size={18} />}
          >
            旋转 {rotation}°
          </Button>
        </>
      )}
    </div>
  );
}
