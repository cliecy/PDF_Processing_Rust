import { useState } from 'react';
import { ArrowDown, ArrowUp, FileText, ListOrdered, RotateCcw } from 'lucide-react';
import Button from '../components/Button';
import PageHeader from '../components/PageHeader';
import ResultCard from '../components/ResultCard';
import { PdfInfo, ProcessResult, useTauri } from '../hooks/useTauri';

export default function ReorderPages() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [pageOrder, setPageOrder] = useState<number[]>([]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const {
    selectPdfFiles,
    getPdfInfo,
    reorderPages,
    selectOutputFile,
    openFolder,
    copyToClipboard,
    formatFileSize,
  } = useTauri();

  const originalOrder = (pageCount: number) =>
    Array.from({ length: pageCount }, (_, index) => index + 1);

  const handleSelectFile = async () => {
    try {
      const paths = await selectPdfFiles(false);
      if (!paths?.length) return;

      const path = paths[0];
      const info = await getPdfInfo(path);
      setFilePath(path);
      setFileName(path.split(/[/\\]/).pop() || path);
      setPdfInfo(info);
      setPageOrder(originalOrder(info.page_count));
    } catch (error) {
      setResult({
        success: false,
        message: `无法读取 PDF 文件：${String(error)}`,
        output_path: null,
      });
    }
  };

  const movePage = (index: number, offset: -1 | 1) => {
    const target = index + offset;
    if (target < 0 || target >= pageOrder.length) return;

    const nextOrder = [...pageOrder];
    [nextOrder[index], nextOrder[target]] = [nextOrder[target], nextOrder[index]];
    setPageOrder(nextOrder);
  };

  const handleReorder = async () => {
    if (!filePath || pageOrder.length === 0) return;

    try {
      const outputPath = await selectOutputFile('reordered.pdf');
      if (!outputPath) return;

      setLoading(true);
      setResult(await reorderPages(filePath, pageOrder, outputPath));
    } catch (error) {
      setResult({ success: false, message: String(error), output_path: null });
    } finally {
      setLoading(false);
    }
  };

  const handleReset = () => {
    setFilePath(null);
    setFileName('');
    setPdfInfo(null);
    setPageOrder([]);
    setResult(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="页面重排"
          description="调整 PDF 页面的先后顺序"
          icon={<ListOrdered size={28} />}
          iconColor="bg-cyan-500/10 text-cyan-400"
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
        title="页面重排"
        description="调整 PDF 页面的先后顺序"
        icon={<ListOrdered size={28} />}
        iconColor="bg-cyan-500/10 text-cyan-400"
      />

      {!filePath ? (
        <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-12 text-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
            <FileText className="text-zinc-500" size={28} />
          </div>
          <p className="text-zinc-400 mb-4">选择要重新排序的 PDF 文件</p>
          <Button onClick={handleSelectFile}>选择 PDF 文件</Button>
        </div>
      ) : (
        <>
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-4 mb-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-xl bg-cyan-500/10 flex items-center justify-center">
                <FileText className="text-cyan-400" size={24} />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-white font-medium truncate">{fileName}</p>
                <p className="text-sm text-zinc-500">
                  {pdfInfo?.page_count} 页 · {formatFileSize(pdfInfo?.file_size || 0)} · PDF{' '}
                  {pdfInfo?.pdf_version}
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={handleSelectFile}>
                更换文件
              </Button>
            </div>
          </div>

          <div className="mb-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-white font-medium">当前页面顺序</h3>
              <Button
                variant="ghost"
                size="sm"
                icon={<RotateCcw size={16} />}
                onClick={() => setPageOrder(originalOrder(pdfInfo?.page_count || 0))}
              >
                恢复原顺序
              </Button>
            </div>
            <div className="space-y-2 max-h-[420px] overflow-y-auto pr-1">
              {pageOrder.map((page, index) => (
                <div
                  key={page}
                  className="flex items-center gap-3 bg-[#1a1a21] border border-[#2e2e38] rounded-xl px-4 py-3"
                >
                  <span className="w-8 h-8 rounded-lg bg-cyan-500/10 text-cyan-400 flex items-center justify-center text-sm font-medium">
                    {index + 1}
                  </span>
                  <span className="text-zinc-400 flex-1">
                    原 PDF 第 <span className="text-white font-medium">{page}</span> 页
                  </span>
                  <button
                    onClick={() => movePage(index, -1)}
                    disabled={index === 0}
                    className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-[#2e2e38] disabled:opacity-30 disabled:cursor-not-allowed"
                    title="上移"
                  >
                    <ArrowUp size={17} />
                  </button>
                  <button
                    onClick={() => movePage(index, 1)}
                    disabled={index === pageOrder.length - 1}
                    className="p-2 rounded-lg text-zinc-400 hover:text-white hover:bg-[#2e2e38] disabled:opacity-30 disabled:cursor-not-allowed"
                    title="下移"
                  >
                    <ArrowDown size={17} />
                  </button>
                </div>
              ))}
            </div>
          </div>

          <Button
            onClick={handleReorder}
            loading={loading}
            disabled={pageOrder.length === 0}
            icon={<ListOrdered size={18} />}
          >
            保存新顺序
          </Button>
        </>
      )}
    </div>
  );
}
