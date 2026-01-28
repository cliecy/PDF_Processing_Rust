import { CheckCircle, XCircle, FolderOpen } from 'lucide-react';
import Button from './Button';

interface ResultCardProps {
  success: boolean;
  message: string;
  outputPath?: string;
  onOpenFolder?: () => void;
  onReset?: () => void;
}

export default function ResultCard({
  success,
  message,
  outputPath,
  onOpenFolder,
  onReset,
}: ResultCardProps) {
  return (
    <div
      className={`rounded-2xl p-6 border animate-fade-in ${
        success
          ? 'bg-green-500/5 border-green-500/20'
          : 'bg-red-500/5 border-red-500/20'
      }`}
    >
      <div className="flex items-start gap-4">
        <div
          className={`w-12 h-12 rounded-xl flex items-center justify-center ${
            success ? 'bg-green-500/10' : 'bg-red-500/10'
          }`}
        >
          {success ? (
            <CheckCircle className="text-green-400" size={24} />
          ) : (
            <XCircle className="text-red-400" size={24} />
          )}
        </div>
        <div className="flex-1">
          <h3
            className={`font-semibold ${
              success ? 'text-green-400' : 'text-red-400'
            }`}
          >
            {success ? '处理成功' : '处理失败'}
          </h3>
          <p className="text-zinc-400 text-sm mt-1">{message}</p>
          {outputPath && (
            <p className="text-zinc-500 text-xs mt-2 font-mono truncate">
              {outputPath}
            </p>
          )}
        </div>
      </div>
      <div className="flex gap-3 mt-4 pt-4 border-t border-[#2e2e38]">
        {success && onOpenFolder && (
          <Button
            variant="secondary"
            size="sm"
            icon={<FolderOpen size={16} />}
            onClick={onOpenFolder}
          >
            打开文件夹
          </Button>
        )}
        {onReset && (
          <Button variant="ghost" size="sm" onClick={onReset}>
            重新开始
          </Button>
        )}
      </div>
    </div>
  );
}
