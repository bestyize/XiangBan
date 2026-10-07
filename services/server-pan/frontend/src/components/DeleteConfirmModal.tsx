import React, { useState } from 'react';
import { FileItem } from '../types';
import { fileApi } from '../api';
import { Trash2, X, AlertTriangle, Loader2 } from 'lucide-react';

interface DeleteConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  item: FileItem | null;
  onSuccess: () => void;
}

export const DeleteConfirmModal: React.FC<DeleteConfirmModalProps> = ({
  isOpen,
  onClose,
  item,
  onSuccess,
}) => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen || !item) return null;

  const handleDelete = async () => {
    setLoading(true);
    setError(null);

    try {
      await fileApi.delete(item.id);
      onSuccess();
      onClose();
    } catch (err: any) {
      setError(err.message || '删除失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-md overflow-hidden">

        {/* 顶部交通灯 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-1.5">
            <span className="apple-dot apple-dot-red" onClick={onClose} />
            <span className="apple-dot apple-dot-yellow" />
            <span className="apple-dot apple-dot-green" />
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 rounded-lg p-1">
            <X size={18} />
          </button>
        </div>

        <div className="p-6 text-center space-y-3">
          <div className="w-14 h-14 rounded-2xl bg-[#FF3B30]/10 text-[#FF3B30] flex items-center justify-center mx-auto">
            <AlertTriangle size={28} />
          </div>

          <h3 className="text-base font-bold text-slate-800">
            确认删除此{item.is_dir ? '文件夹' : '文件'}？
          </h3>

          <p className="text-xs text-slate-500 break-all px-4">
            「<strong>{item.name}</strong>」
            {item.is_dir && (
              <span className="block text-[#FF3B30] mt-1 font-medium">
                注意：删除该文件夹将同时递归清空其内部的所有子文件和子目录！
              </span>
            )}
          </p>

          {error && (
            <p className="text-xs text-[#FF3B30] bg-[#FF3B30]/10 p-2 rounded-xl">
              {error}
            </p>
          )}

          <div className="flex items-center justify-center gap-3 pt-3">
            <button
              onClick={onClose}
              disabled={loading}
              className="px-5 py-2.5 text-xs font-semibold text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all"
            >
              取消
            </button>
            <button
              onClick={handleDelete}
              disabled={loading}
              className="flex items-center gap-1.5 px-6 py-2.5 text-xs font-semibold text-white bg-[#FF3B30] hover:bg-[#E02D23] disabled:opacity-50 rounded-xl shadow-xs transition-all"
            >
              {loading ? <Loader2 size={14} className="animate-spin" /> : <Trash2 size={14} />}
              确认删除
            </button>
          </div>
        </div>

      </div>
    </div>
  );
};
