import React, { useState } from 'react';
import { fileApi } from '../api';
import { FolderPlus, X, AlertCircle, Loader2 } from 'lucide-react';

interface CreateFolderModalProps {
  isOpen: boolean;
  onClose: () => void;
  folderId: string | null;
  onSuccess: () => void;
}

export const CreateFolderModal: React.FC<CreateFolderModalProps> = ({
  isOpen,
  onClose,
  folderId,
  onSuccess,
}) => {
  const [folderName, setFolderName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!folderName.trim()) return;

    setLoading(true);
    setError(null);

    try {
      await fileApi.mkdir(folderName.trim(), folderId);
      setFolderName('');
      onSuccess();
      onClose();
    } catch (err: any) {
      setError(err.message || '创建文件夹失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-md overflow-hidden">

        {/* 顶部标题栏 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-1.5 mr-1">
              <span className="apple-dot apple-dot-red" onClick={onClose} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <div className="w-7 h-7 rounded-lg bg-amber-500/10 text-amber-500 flex items-center justify-center">
              <FolderPlus size={16} />
            </div>
            <h3 className="font-bold text-slate-800 text-sm">新建文件夹</h3>
          </div>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-slate-600 rounded-lg p-1"
          >
            <X size={18} />
          </button>
        </div>

        {/* 表单主体 */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
              <AlertCircle size={16} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}

          <div>
            <label className="block text-xs font-semibold text-slate-600 mb-1.5">
              文件夹名称
            </label>
            <input
              type="text"
              autoFocus
              placeholder="例如：学习资料、设计稿、工作文档"
              value={folderName}
              onChange={(e) => setFolderName(e.target.value)}
              className="w-full px-3.5 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all focus:ring-3 focus:ring-[#007AFF]/10 text-slate-800"
            />
          </div>

          <div className="flex items-center justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              disabled={loading}
              className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all"
            >
              取消
            </button>
            <button
              type="submit"
              disabled={loading || !folderName.trim()}
              className="flex items-center gap-1.5 px-5 py-2 text-sm font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 disabled:cursor-not-allowed rounded-xl shadow-xs transition-all"
            >
              {loading && <Loader2 size={16} className="animate-spin" />}
              创建
            </button>
          </div>
        </form>

      </div>
    </div>
  );
};
