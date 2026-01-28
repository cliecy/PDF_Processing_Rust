import { useState } from 'react';
import { Trash2, FileText } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult, PdfInfo } from '../hooks/useTauri';

export default function DeletePages() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [selectedPages, setSelectedPages] = useState<Set<number>>(new Set());
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const { selectPdfFiles, getPdfInfo, deletePages, selectOutputFile } = useTauri();

  const handleSelectFile = async () => {
    try {
      const paths = await selectPdfFiles(false);
      if (paths && paths.length > 0) {
        const path = paths[0];
        setFilePath(path);
        setFileName(path.split(/[/\\]/).pop() || path);
        const info = await getPdfInfo(path);
        setPdfInfo(info);
        setSelectedPages(new Set());
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

  const handleDelete = async () => {
    if (!filePath || selectedPages.size === 0) return;

    try {
      const outputPath = await selectOutputFile('output.pdf');
      if (!outputPath) return;

      setLoading(true);
      const res = await deletePages(
        filePath,
        Array.from(selectedPages),
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
    setResult(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="删除页面"
          description="从 PDF 中删除指定页面"
          icon={<Trash2 size={28} />}
          iconColor="bg-rose-500/10 text-rose-400"
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
        title="删除页面"
        description="从 PDF 中删除指定页面"
        icon={<Trash2 size={28} />}
        iconColor="bg-rose-500/10 text-rose-400"
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
              <div className="w-12 h-12 rounded-xl bg-rose-500/10 flex items-center justify-center">
                <FileText className="text-rose-400" size={24} />
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

          {/* Page Selection */}
          <div className="mb-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-white font-medium">选择要删除的页面</h3>
              <span className="text-sm text-zinc-500">
                已选择 {selectedPages.size} 页
              </span>
            </div>
            <div className="grid grid-cols-8 gap-2">
              {Array.from({ length: pdfInfo?.page_count || 0 }, (_, i) => i + 1).map(
                (page) => (
                  <button
                    key={page}
                    onClick={() => togglePage(page)}
                    className={`aspect-square rounded-lg flex items-center justify-center text-sm font-medium transition-all ${
                      selectedPages.has(page)
                        ? 'bg-rose-500 text-white'
                        : 'bg-[#22222b] text-zinc-400 hover:bg-[#2a2a35] hover:text-white'
                    }`}
                  >
                    {page}
                  </button>
                )
              )}
            </div>
          </div>

          {/* Preview */}
          {selectedPages.size > 0 && (
            <div className="bg-amber-500/5 border border-amber-500/20 rounded-xl p-4 mb-6">
              <p className="text-sm text-amber-400 font-medium mb-2">
                操作预览
              </p>
              <p className="text-xs text-zinc-400 mb-1">
                将删除页码：<span className="text-white font-mono">
                  {Array.from(selectedPages).sort((a, b) => a - b).join(', ')}
                </span>
              </p>
              <p className="text-xs text-zinc-400">
                保留 <span className="text-green-400 font-semibold">
                  {(pdfInfo?.page_count || 0) - selectedPages.size}
                </span> 页
              </p>
            </div>
          )}

          {/* Action */}
          <Button
            variant="danger"
            onClick={handleDelete}
            loading={loading}
            disabled={selectedPages.size === 0}
            icon={<Trash2 size={18} />}
          >
            删除选中的 {selectedPages.size} 页
          </Button>
        </>
      )}
    </div>
  );
}
