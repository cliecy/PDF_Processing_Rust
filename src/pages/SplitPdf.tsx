import { useState } from 'react';
import { Scissors, FileText, Plus, Trash2 } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult, PdfInfo } from '../hooks/useTauri';

interface SplitRange {
  start: number;
  end: number;
}

export default function SplitPdf() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [ranges, setRanges] = useState<SplitRange[]>([{ start: 1, end: 1 }]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const { selectPdfFiles, getPdfInfo, splitPdf, selectOutputDir } = useTauri();

  const handleSelectFile = async () => {
    try {
      const paths = await selectPdfFiles(false);
      if (paths && paths.length > 0) {
        const path = paths[0];
        setFilePath(path);
        setFileName(path.split(/[/\\]/).pop() || path);
        const info = await getPdfInfo(path);
        setPdfInfo(info);
        setRanges([{ start: 1, end: info.page_count }]);
      }
    } catch (error) {
      console.error('Error selecting file:', error);
    }
  };

  const handleAddRange = () => {
    const lastEnd = ranges[ranges.length - 1]?.end || 0;
    const maxPage = pdfInfo?.page_count || 1;
    setRanges([
      ...ranges,
      {
        start: Math.min(lastEnd + 1, maxPage),
        end: maxPage,
      },
    ]);
  };

  const handleRemoveRange = (index: number) => {
    if (ranges.length > 1) {
      setRanges(ranges.filter((_, i) => i !== index));
    }
  };

  const handleRangeChange = (
    index: number,
    field: 'start' | 'end',
    value: number
  ) => {
    const newRanges = [...ranges];
    newRanges[index][field] = Math.max(
      1,
      Math.min(value, pdfInfo?.page_count || 1)
    );
    setRanges(newRanges);
  };

  const handleSplit = async () => {
    if (!filePath) return;

    try {
      const outputDir = await selectOutputDir();
      if (!outputDir) return;

      setLoading(true);
      const res = await splitPdf(
        filePath,
        ranges.map((r) => [r.start, r.end] as [number, number]),
        outputDir
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
    setRanges([{ start: 1, end: 1 }]);
    setResult(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="PDF 分割"
          description="将 PDF 分割为多个文件"
          icon={<Scissors size={28} />}
          iconColor="bg-red-500/10 text-red-400"
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
        title="PDF 分割"
        description="将 PDF 分割为多个文件"
        icon={<Scissors size={28} />}
        iconColor="bg-red-500/10 text-red-400"
      />

      {/* File Selection */}
      {!filePath ? (
        <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-12 text-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
            <FileText className="text-zinc-500" size={28} />
          </div>
          <p className="text-zinc-400 mb-4">选择要分割的 PDF 文件</p>
          <Button variant="primary" onClick={handleSelectFile}>
            选择 PDF 文件
          </Button>
        </div>
      ) : (
        <>
          {/* Selected File Info */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-4 mb-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-xl bg-red-500/10 flex items-center justify-center">
                <FileText className="text-red-400" size={24} />
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

          {/* Split Ranges */}
          <div className="space-y-4 mb-6">
            <div className="flex items-center justify-between">
              <h3 className="text-white font-medium">分割范围</h3>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleAddRange}
                icon={<Plus size={16} />}
              >
                添加范围
              </Button>
            </div>

            {ranges.map((range, index) => {
              const pageCount = range.end - range.start + 1;
              const pages = Array.from({ length: pageCount }, (_, i) => range.start + i);
              return (
                <div
                  key={index}
                  className="bg-[#1a1a21] rounded-xl border border-[#2e2e38] p-4"
                >
                  <div className="flex items-center gap-4 mb-3">
                    <span className="text-zinc-500 text-sm w-16">
                      文件 {index + 1}
                    </span>
                    <div className="flex items-center gap-2 flex-1">
                      <span className="text-zinc-400 text-sm">第</span>
                      <input
                        type="number"
                        value={range.start}
                        onChange={(e) =>
                          handleRangeChange(index, 'start', parseInt(e.target.value) || 1)
                        }
                        min={1}
                        max={pdfInfo?.page_count || 1}
                        className="w-20 px-3 py-2 bg-[#22222b] border border-[#2e2e38] rounded-lg text-white text-center focus:outline-none focus:border-red-500"
                      />
                      <span className="text-zinc-400 text-sm">页 至 第</span>
                      <input
                        type="number"
                        value={range.end}
                        onChange={(e) =>
                          handleRangeChange(index, 'end', parseInt(e.target.value) || 1)
                        }
                        min={1}
                        max={pdfInfo?.page_count || 1}
                        className="w-20 px-3 py-2 bg-[#22222b] border border-[#2e2e38] rounded-lg text-white text-center focus:outline-none focus:border-red-500"
                      />
                      <span className="text-zinc-400 text-sm">页</span>
                    </div>
                    {ranges.length > 1 && (
                      <button
                        onClick={() => handleRemoveRange(index)}
                        className="p-2 rounded-lg hover:bg-red-500/10 text-zinc-400 hover:text-red-400 transition-colors"
                      >
                        <Trash2 size={18} />
                      </button>
                    )}
                  </div>
                  <div className="mt-2 pt-3 border-t border-[#2e2e38]">
                    <p className="text-xs text-zinc-500 mb-1">
                      将包含页码：<span className="text-zinc-400 font-mono">{pages.join(', ')}</span>
                    </p>
                    <p className="text-xs text-zinc-500">
                      共 <span className="text-green-400 font-semibold">{pageCount}</span> 页
                    </p>
                  </div>
                </div>
              );
            })}
          </div>

          {/* Action */}
          <Button
            variant="primary"
            onClick={handleSplit}
            loading={loading}
            icon={<Scissors size={18} />}
          >
            分割 PDF
          </Button>
        </>
      )}
    </div>
  );
}
