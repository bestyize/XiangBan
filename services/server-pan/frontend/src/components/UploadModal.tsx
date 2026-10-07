import React, { useRef, useState } from 'react';
import { fileApi, formatBytes } from '../api';
import {
  UploadCloud,
  X,
  File as FileIcon,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from 'lucide-react';

interface UploadModalProps {
  isOpen: boolean;
  onClose: () => void;
  folderId: string | null;
  onUploadSuccess: () => void;
}

export const UploadModal: React.FC<UploadModalProps> = ({
  isOpen,
  onClose,
  folderId,
  onUploadSuccess,
}) => {
  const [selectedFiles, setSelectedFiles] = useState<File[]>([]);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  if (!isOpen) return null;

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      const filesArray = Array.from(e.target.files);
      setSelectedFiles((prev) => [...prev, ...filesArray]);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    if (e.dataTransfer.files) {
      const filesArray = Array.from(e.dataTransfer.files);
      setSelectedFiles((prev) => [...prev, ...filesArray]);
    }
  };

  const handleRemoveFile = (index: number) => {
    setSelectedFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const handleStartUpload = async () => {
    if (selectedFiles.length === 0) return;
    setUploading(true);
    setError(null);

    try {
      await fileApi.upload(selectedFiles, folderId);
      setSelectedFiles([]);
      onUploadSuccess();
      onClose();
    } catch (err: any) {
      setError(err.message || '上传失败，请重试');
    } finally {
      setUploading(false);
    }
  };

  const totalSize = selectedFiles.reduce((acc, f) => acc + f.size, 0);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-lg overflow-hidden flex flex-col max-h-[85vh]">

        {/* 顶部标题栏（带 macOS 交通灯控制点） */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5">
              <span className="apple-dot apple-dot-red" onClick={onClose} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <h3 className="font-bold text-slate-800 text-sm ml-1">上传文件到 PanDown</h3>
          </div>
          <button
            onClick={onClose}
            disabled={uploading}
            className="text-slate-400 hover:text-slate-600 rounded-lg p-1"
          >
            <X size={18} />
          </button>
        </div>

        {/* 主体内容 */}
        <div className="p-6 flex-1 overflow-y-auto space-y-4">
          {/* 拖拽上传框 */}
          <div
            onDragOver={(e) => {
              e.preventDefault();
              setIsDragOver(true);
            }}
            onDragLeave={() => setIsDragOver(false)}
            onDrop={handleDrop}
            onClick={() => fileInputRef.current?.click()}
            className={`border-2 border-dashed rounded-2xl p-6 text-center cursor-pointer transition-all flex flex-col items-center justify-center ${
              isDragOver
                ? 'border-[#007AFF] bg-[#007AFF]/5'
                : 'border-slate-200 hover:border-[#007AFF]/50 hover:bg-slate-50'
            }`}
          >
            <input
              ref={fileInputRef}
              type="file"
              multiple
              className="hidden"
              onChange={handleFileChange}
            />
            <div className="w-12 h-12 rounded-2xl bg-[#007AFF]/10 flex items-center justify-center text-[#007AFF] mb-2.5">
              <UploadCloud size={24} />
            </div>
            <p className="text-sm font-semibold text-slate-800">
              拖拽文件到这里，或 <span className="text-[#007AFF] underline">点击选择</span>
            </p>
            <p className="text-xs text-slate-400 mt-1">支持批量多选文件同时上传</p>
          </div>

          {/* 错误提示 */}
          {error && (
            <div className="p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
              <AlertCircle size={16} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* 待上传文件列表 */}
          {selectedFiles.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs font-semibold text-slate-500 px-1">
                <span>待上传文件 ({selectedFiles.length})</span>
                <span>总大小: {formatBytes(totalSize)}</span>
              </div>
              <div className="max-h-48 overflow-y-auto space-y-1.5 pr-1">
                {selectedFiles.map((f, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-2.5 bg-[#F5F5F7] rounded-xl text-xs border border-slate-200/60"
                  >
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                      <FileIcon size={16} className="text-slate-400 shrink-0" />
                      <span className="font-medium text-slate-800 truncate" title={f.name}>
                        {f.name}
                      </span>
                      <span className="text-slate-400 text-[11px] shrink-0">
                        ({formatBytes(f.size)})
                      </span>
                    </div>
                    {!uploading && (
                      <button
                        onClick={() => handleRemoveFile(idx)}
                        className="text-slate-400 hover:text-[#FF3B30] p-1 rounded transition-colors ml-2"
                      >
                        <X size={14} />
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* 底部按钮栏 */}
        <div className="px-6 py-4 bg-slate-50/70 border-t border-slate-100 flex items-center justify-between">
          <button
            onClick={() => {
              setSelectedFiles([]);
              onClose();
            }}
            disabled={uploading}
            className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-800 hover:bg-slate-200/50 rounded-xl transition-all"
          >
            取消
          </button>
          <button
            onClick={handleStartUpload}
            disabled={uploading || selectedFiles.length === 0}
            className="flex items-center gap-2 px-5 py-2 text-sm font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 disabled:cursor-not-allowed rounded-xl shadow-xs transition-all"
          >
            {uploading ? (
              <>
                <Loader2 size={16} className="animate-spin" />
                正在上传中...
              </>
            ) : (
              <>
                <CheckCircle2 size={16} />
                确认上传 ({selectedFiles.length})
              </>
            )}
          </button>
        </div>

      </div>
    </div>
  );
};
