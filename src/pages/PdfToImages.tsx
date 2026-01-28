import { useState } from 'react';
import { Image, FileText } from 'lucide-react';
import PageHeader from '../components/PageHeader';
import Button from '../components/Button';
import ResultCard from '../components/ResultCard';
import { useTauri, ProcessResult, PdfInfo } from '../hooks/useTauri';

export default function PdfToImages() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string>('');
  const [pdfInfo, setPdfInfo] = useState<PdfInfo | null>(null);
  const [format, setFormat] = useState<'png' | 'jpg'>('png');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ProcessResult | null>(null);
  const { selectPdfFiles, getPdfInfo, pdfToImages, selectOutputDir } = useTauri();

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

  const handleConvert = async () => {
    if (!filePath) return;

    try {
      const outputDir = await selectOutputDir();
      if (!outputDir) return;

      setLoading(true);
      const res = await pdfToImages(filePath, outputDir, format);
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
    setFormat('png');
    setResult(null);
  };

  if (result) {
    return (
      <div className="max-w-2xl mx-auto">
        <PageHeader
          title="PDF 转图片"
          description="将 PDF 页面转换为图片"
          icon={<Image size={28} />}
          iconColor="bg-pink-500/10 text-pink-400"
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
        title="PDF 转图片"
        description="将 PDF 页面转换为图片"
        icon={<Image size={28} />}
        iconColor="bg-pink-500/10 text-pink-400"
      />

      {!filePath ? (
        <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-12 text-center mb-6">
          <div className="w-16 h-16 rounded-2xl bg-[#22222b] flex items-center justify-center mx-auto mb-4">
            <FileText className="text-zinc-500" size={28} />
          </div>
          <p className="text-zinc-400 mb-4">选择要转换的 PDF 文件</p>
          <Button variant="primary" onClick={handleSelectFile}>
            选择 PDF 文件
          </Button>
        </div>
      ) : (
        <>
          {/* Selected File Info */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-4 mb-6">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-xl bg-pink-500/10 flex items-center justify-center">
                <FileText className="text-pink-400" size={24} />
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

          {/* Format Selection */}
          <div className="bg-[#1a1a21] rounded-2xl border border-[#2e2e38] p-6 mb-6">
            <h3 className="text-white font-medium mb-4">输出格式</h3>
            <div className="flex gap-3">
              {(['png', 'jpg'] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFormat(f)}
                  className={`flex-1 flex flex-col items-center gap-2 p-4 rounded-xl transition-all ${
                    format === f
                      ? 'bg-pink-500/20 border border-pink-500/40'
                      : 'bg-[#22222b] border border-transparent hover:border-[#3e3e48]'
                  }`}
                >
                  <Image
                    className={format === f ? 'text-pink-400' : 'text-zinc-400'}
                    size={24}
                  />
                  <span
                    className={`text-sm font-medium uppercase ${
                      format === f ? 'text-pink-400' : 'text-zinc-400'
                    }`}
                  >
                    {f}
                  </span>
                  <span className="text-xs text-zinc-600">
                    {f === 'png' ? '无损压缩' : '有损压缩'}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Info */}
          <div className="bg-green-500/5 border border-green-500/20 rounded-xl p-4 mb-6">
            <p className="text-green-400 text-sm">
              <strong>✓</strong> 此功能使用纯 Rust 实现，PDFium 库已自动包含在应用中，无需额外安装。
            </p>
          </div>

          {/* Action */}
          <Button
            variant="primary"
            onClick={handleConvert}
            loading={loading}
            icon={<Image size={18} />}
          >
            转换为 {format.toUpperCase()} 图片
          </Button>
        </>
      )}
    </div>
  );
}
