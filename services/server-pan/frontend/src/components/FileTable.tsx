import React from 'react';
import { FileItem } from '../types';
import { FileIcon } from './FileIcon';
import { formatBytes, formatDate } from '../api';
import {
  Download,
  Share2,
  Trash2,
  Edit3,
  Eye,
  CornerDownRight,
} from 'lucide-react';

interface FileTableProps {
  items: FileItem[];
  onOpenFolder: (id: string) => void;
  onPreview: (item: FileItem) => void;
  onDownload: (item: FileItem) => void;
  onShare: (item: FileItem) => void;
  onRename: (item: FileItem) => void;
  onDelete: (item: FileItem) => void;
}

export const FileTable: React.FC<FileTableProps> = ({
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
    <div className="bg-white rounded-2xl border border-slate-200/80 shadow-xs overflow-hidden my-3">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse text-sm">
          <thead>
            <tr className="border-b border-slate-100 text-slate-400 font-semibold text-xs bg-slate-50/50 select-none">
              <th className="py-3 px-4 w-12 text-center">类型</th>
              <th className="py-3 px-4">名称</th>
              <th className="py-3 px-4 w-32 hidden sm:table-cell">大小</th>
              <th className="py-3 px-4 w-44 hidden md:table-cell">修改时间</th>
              <th className="py-3 px-4 w-52 text-right pr-6">操作</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {items.map((item) => {
              const canPreview = !item.is_dir;

              return (
                <tr
                  key={item.id}
                  className="group hover:bg-[#F5F5F7]/80 transition-colors cursor-pointer"
                  onClick={() => {
                    if (item.is_dir) {
                      onOpenFolder(item.id);
                    } else {
                      onPreview(item);
                    }
                  }}
                >
                  {/* 图标列 */}
                  <td className="py-2.5 px-4 text-center shrink-0">
                    <div className="inline-flex items-center justify-center">
                      <FileIcon isDir={item.is_dir} name={item.name} size={18} />
                    </div>
                  </td>

                  {/* 名称列 */}
                  <td className="py-2.5 px-4">
                    <div className="flex items-center gap-2">
                      <span
                        className={`font-medium truncate max-w-xs md:max-w-md ${
                          item.is_dir
                            ? 'text-slate-900 group-hover:text-[#007AFF] font-semibold'
                            : 'text-slate-800'
                        }`}
                        title={item.name}
                      >
                        {item.name}
                      </span>
                    </div>
                  </td>

                  {/* 大小列 */}
                  <td className="py-2.5 px-4 text-slate-500 text-xs hidden sm:table-cell">
                    {item.is_dir ? '-' : formatBytes(item.size)}
                  </td>

                  {/* 修改时间列 */}
                  <td className="py-2.5 px-4 text-slate-400 text-xs hidden md:table-cell">
                    {formatDate(item.updated_at)}
                  </td>

                  {/* 操作按钮组 */}
                  <td
                    className="py-2.5 px-4 text-right pr-6"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex items-center justify-end gap-1 opacity-90 group-hover:opacity-100 transition-opacity">
                      {/* 预览 */}
                      {canPreview && (
                        <button
                          onClick={() => onPreview(item)}
                          title="在线预览"
                          className="p-1.5 rounded-lg text-slate-500 hover:text-[#007AFF] hover:bg-[#007AFF]/10 transition-colors"
                        >
                          <Eye size={16} />
                        </button>
                      )}

                      {/* 下载 */}
                      {!item.is_dir && (
                        <button
                          onClick={() => onDownload(item)}
                          title="下载文件"
                          className="p-1.5 rounded-lg text-slate-500 hover:text-[#34C759] hover:bg-[#34C759]/10 transition-colors"
                        >
                          <Download size={16} />
                        </button>
                      )}

                      {/* 分享 */}
                      <button
                        onClick={() => onShare(item)}
                        title="创建分享链接"
                        className="p-1.5 rounded-lg text-slate-500 hover:text-[#FF9500] hover:bg-amber-500/10 transition-colors"
                      >
                        <Share2 size={16} />
                      </button>

                      {/* 重命名 */}
                      <button
                        onClick={() => onRename(item)}
                        title="重命名"
                        className="p-1.5 rounded-lg text-slate-500 hover:text-slate-800 hover:bg-slate-200/80 transition-colors"
                      >
                        <Edit3 size={16} />
                      </button>

                      {/* 删除 */}
                      <button
                        onClick={() => onDelete(item)}
                        title="删除"
                        className="p-1.5 rounded-lg text-slate-500 hover:text-[#FF3B30] hover:bg-[#FF3B30]/10 transition-colors"
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
