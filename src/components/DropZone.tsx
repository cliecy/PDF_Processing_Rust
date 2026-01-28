import { useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, File, X } from 'lucide-react';

interface DropZoneProps {
  onFilesSelected: (files: File[]) => void;
  files: File[];
  onRemoveFile?: (index: number) => void;
  accept?: Record<string, string[]>;
  multiple?: boolean;
  maxFiles?: number;
  label?: string;
  description?: string;
}

export default function DropZone({
  onFilesSelected,
  files,
  onRemoveFile,
  accept = { 'application/pdf': ['.pdf'] },
  multiple = true,
  maxFiles,
  label = '拖放文件到这里',
  description = '或点击选择文件',
}: DropZoneProps) {
  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      if (maxFiles && files.length + acceptedFiles.length > maxFiles) {
        acceptedFiles = acceptedFiles.slice(0, maxFiles - files.length);
      }
      onFilesSelected([...files, ...acceptedFiles]);
    },
    [files, maxFiles, onFilesSelected]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept,
    multiple,
  });

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="space-y-4">
      <div
        {...getRootProps()}
        className={`drop-zone rounded-2xl p-12 text-center cursor-pointer transition-all duration-300 ${
          isDragActive ? 'active border-orange-500 bg-orange-500/5' : ''
        }`}
      >
        <input {...getInputProps()} />
        <div className="flex flex-col items-center gap-4">
          <div
            className={`w-16 h-16 rounded-2xl flex items-center justify-center transition-all duration-300 ${
              isDragActive
                ? 'bg-orange-500/20 text-orange-400 scale-110'
                : 'bg-[#2a2a35] text-zinc-400'
            }`}
          >
            <Upload size={28} />
          </div>
          <div>
            <p className="text-lg font-medium text-white">{label}</p>
            <p className="text-sm text-zinc-500 mt-1">{description}</p>
          </div>
          {maxFiles && (
            <p className="text-xs text-zinc-600">
              最多可选择 {maxFiles} 个文件
            </p>
          )}
        </div>
      </div>

      {/* File list */}
      {files.length > 0 && (
        <div className="space-y-2">
          <p className="text-sm text-zinc-400 font-medium">
            已选择 {files.length} 个文件
          </p>
          <div className="space-y-2 max-h-60 overflow-y-auto pr-2">
            {files.map((file, index) => (
              <div
                key={`${file.name}-${index}`}
                className="flex items-center gap-3 p-3 bg-[#22222b] rounded-xl group animate-fade-in"
              >
                <div className="w-10 h-10 rounded-lg bg-red-500/10 flex items-center justify-center">
                  <File className="text-red-400" size={20} />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-white truncate">
                    {file.name}
                  </p>
                  <p className="text-xs text-zinc-500">
                    {formatFileSize(file.size)}
                  </p>
                </div>
                {onRemoveFile && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemoveFile(index);
                    }}
                    className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-500 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover:opacity-100"
                  >
                    <X size={16} />
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
