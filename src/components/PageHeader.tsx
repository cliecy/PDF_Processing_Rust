import { ReactNode } from 'react';
import { ArrowLeft } from 'lucide-react';

interface PageHeaderProps {
  title: string;
  description: string;
  icon: ReactNode;
  iconColor: string;
  onBack?: () => void;
}

export default function PageHeader({
  title,
  description,
  icon,
  iconColor,
  onBack,
}: PageHeaderProps) {
  return (
    <div className="mb-8">
      {onBack && (
        <button
          onClick={onBack}
          className="flex items-center gap-2 text-zinc-400 hover:text-white mb-4 transition-colors"
        >
          <ArrowLeft size={18} />
          <span className="text-sm">返回</span>
        </button>
      )}
      <div className="flex items-center gap-4">
        <div
          className={`w-14 h-14 rounded-2xl flex items-center justify-center ${iconColor}`}
        >
          {icon}
        </div>
        <div>
          <h1 className="text-2xl font-semibold text-white">{title}</h1>
          <p className="text-zinc-400 mt-1">{description}</p>
        </div>
      </div>
    </div>
  );
}
