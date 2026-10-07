import React from 'react';
import { FileItem } from '../types';
import { FileIcon } from './FileIcon';
import { formatBytes } from '../api';
import {
  Download,
  Share2,
  Trash2,
  Edit3,
  Eye,
  CornerDownRight,
} from 'lucide-react';

interface FileGridProps {
  items: FileItem[];
  onOpenFolder: (id: string) => void;
  onPreview: (item: FileItem) => void;
  onDownload: (item: FileItem) => void;
  onShare: (item: FileItem) => void;
  onRename: (item: FileItem) => void;
  onDelete: (item: FileItem) => void;
}

export const FileGrid: React.FC<FileGridProps> = ({
  items,
  onOpenFolder,
  onPreview,
  onDownload,
  onShare,
  onRename,
  onDelete,
}) => {
  if (items.length === 0) {
    return (
      <div className="py-20 text-center flex flex-col items-center justify-center bg-white rounded-3xl border border-slate-200/80 shadow-xs my-4">
        <div className="w-16 h-16 rounded-2xl bg-[#007AFF]/10 flex items-center justify-center text-[#007AFF] mb-3">
          <CornerDownRight size={28} />
        </div>
        <h3 className="text-base font-bold text-slate-800">当前文件夹为空</h3>
        <p className="text-xs text-slate-400 mt-1 max-w-sm">
          点击上方“上传文件”或直接将文件拖拽至此页面开始存储
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3.5 my-3">
      {items.map((item) => (
        <div
          key={item.id}
          className="group relative bg-white hover:bg-white rounded-2xl border border-slate-200/80 hover:border-[#007AFF]/40 shadow-xs hover:shadow-md p-3.5 flex flex-col items-center text-center transition-all cursor-pointer select-none"
          onDoubleClick={() => {
            if (item.is_dir) {
              onOpenFolder(item.id);
            } else {
              onPreview(item);
            }
          }}
          onClick={() => {
            if (item.is_dir) {
              onOpenFolder(item.id);
            }
          }}
        >
          {/* 快捷操作浮层（右上角） */}
          <div
            className="absolute top-2 right-2 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity bg-white/95 shadow-sm rounded-lg p-0.5 border border-slate-200 z-10"
            onClick={(e) => e.stopPropagation()}
          >
            {!item.is_dir && (
              <button
                onClick={() => onPreview(item)}
                title="预览"
                className="p-1 rounded text-slate-600 hover:text-[#007AFF] hover:bg-[#007AFF]/10"
              >
                <Eye size={13} />
              </button>
            )}
            {!item.is_dir && (
              <button
                onClick={() => onDownload(item)}
                title="下载"
                className="p-1 rounded text-slate-600 hover:text-[#34C759] hover:bg-[#34C759]/10"
              >
                <Download size={13} />
              </button>
            )}
            <button
              onClick={() => onShare(item)}
              title="分享"
              className="p-1 rounded text-slate-600 hover:text-[#FF9500] hover:bg-amber-500/10"
            >
              <Share2 size={13} />
            </button>
            <button
              onClick={() => onRename(item)}
              title="重命名"
              className="p-1 rounded text-slate-600 hover:text-slate-900 hover:bg-slate-100"
            >
              <Edit3 size={13} />
            </button>
            <button
              onClick={() => onDelete(item)}
              title="删除"
              className="p-1 rounded text-slate-600 hover:text-[#FF3B30] hover:bg-[#FF3B30]/10"
            >
              <Trash2 size={13} />
            </button>
          </div>

          {/* 文件大图标 */}
          <div className="my-3 transition-transform group-hover:scale-105">
            <FileIcon isDir={item.is_dir} name={item.name} size={36} />
          </div>

          {/* 名称 */}
          <p
            className="text-xs font-semibold text-slate-800 group-hover:text-[#007AFF] w-full truncate px-1 transition-colors"
            title={item.name}
          >
            {item.name}
          </p>

          {/* 辅助信息胶囊 */}
          <span className="text-[11px] text-slate-400 mt-1 font-medium">
            {item.is_dir ? '文件夹' : formatBytes(item.size)}
          </span>
        </div>
      ))}
    </div>
  );
};
