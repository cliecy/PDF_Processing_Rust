import { useState } from 'react';
import { Minimize2, FileText } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult, PdfInfo } from '../hooks/useTauri';

export default function CompressPdf() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [quality, setQuality] = useState(75);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const { selectPdfFiles, getPdfInfo, compressPdf, selectOutputFile, openFolder, copyToClipboard, formatFileSize } = useTauri();

  const handleSelectFile = async () => {
    try {
      const paths = await selectPdfFiles(false);
      if (paths && paths.length > 0) {
        const path = paths[0];
        setFilePath(path);
        setFileName(path.split(/[/\\]/).pop() || path);
        const info = await getPdfInfo(path);
        setPdfInfo(info);
      }
    } catch (error) {
      console.error('Error selecting file:', error);
    }
  };

  const handleCompress = async () => {
    if (!filePath) return;

    try {
      const outputPath = await selectOutputFile('compressed.pdf');
      if (!outputPath) return;

      setLoading(true);
      const res = await compressPdf(filePath, outputPath, quality);
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
    setQuality(75);
    setResult(null);
  };

  const qualityLabels: Record<number, string> = {
    25: '轻度优化',
    50: '标准优化',
    75: '深度优化',
    90: '强力优化',
    100: '最大优化',
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="PDF 压缩"
          description="减小 PDF 文件体积"
          icon={<Minimize2 size={28} />}
          iconColor="bg-green-500/10 text-green-400"
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
        title="PDF 压缩"
        description="减小 PDF 文件体积"
        icon={<Minimize2 size={28} />}
        iconColor="bg-green-500/10 text-green-400"
      />

      {!filePath ? (
        <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-12 text-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
            <FileText className="text-zinc-500" size={28} />
          </div>
          <p className="text-zinc-400 mb-4">选择要压缩的 PDF 文件</p>
          <Button variant="primary" onClick={handleSelectFile}>
            选择 PDF 文件
          </Button>
        </div>
      ) : (
        <>
          {/* Selected File Info */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-4 mb-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-xl bg-green-500/10 flex items-center justify-center">
                <FileText className="text-green-400" size={24} />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-white font-medium truncate">{fileName}</p>
                <p className="text-sm text-zinc-500">
                  {pdfInfo?.page_count} 页 · {formatFileSize(pdfInfo?.file_size || 0)} · PDF {pdfInfo?.pdf_version}
                  {pdfInfo?.is_encrypted && <span className="ml-2 text-amber-400">已加密</span>}
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={handleSelectFile}>
                更换文件
              </Button>
            </div>
          </div>

          {/* Optimization Level */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-6 mb-6">
            <h3 className="text-white font-medium mb-4">优化级别</h3>
            <div className="space-y-4">
              <input
                type="range"
                min="25"
                max="100"
                step="25"
                value={quality}
                onChange={(e) => setQuality(parseInt(e.target.value))}
                className="w-full h-2 bg-[#22222b] rounded-lg appearance-none cursor-pointer accent-green-500"
              />
              <div className="flex justify-between text-xs text-zinc-500">
                <span>轻度</span>
                <span>标准</span>
                <span>深度</span>
                <span>强力</span>
                <span>最大</span>
              </div>
              <div className="text-center">
                <span className="inline-block px-4 py-2 bg-green-500/10 rounded-lg text-green-400 font-medium">
                  {qualityLabels[quality]} ({quality}%)
                </span>
              </div>
            </div>
          </div>

          {/* Size Info */}
          <div className="bg-blue-500/5 border border-blue-500/20 rounded-xl p-4 mb-6">
            <p className="text-blue-400 text-sm">
              当前文件大小：<strong>{formatFileSize(pdfInfo?.file_size || 0)}</strong>
            </p>
            <p className="text-xs text-zinc-500 mt-1">
              当前实现是无损结构优化，不会主动降低图片分辨率。更高级别会做更积极的对象清理与重排。
            </p>
          </div>

          {/* Action */}
          <Button
            variant="primary"
            onClick={handleCompress}
            loading={loading}
            icon={<Minimize2 size={18} />}
          >
            压缩 PDF
          </Button>
        </>
      )}
    </div>
  );
}
