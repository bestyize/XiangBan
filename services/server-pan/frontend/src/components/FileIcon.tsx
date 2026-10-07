import React from 'react';
import {
  Folder,
  FileText,
  FileImage,
  FileVideo,
  FileAudio,
  FileArchive,
  FileCode,
  FileSpreadsheet,
  File as GenericFile,
} from 'lucide-react';

interface FileIconProps {
  isDir: boolean;
  name: string;
  size?: number;
}

export const FileIcon: React.FC<FileIconProps> = ({ isDir, name, size = 24 }) => {
  if (isDir) {
    return (
      <div className="w-10 h-10 rounded-xl bg-amber-500/10 flex items-center justify-center text-amber-500 shadow-xs border border-amber-500/20">
        <Folder size={size} className="fill-amber-500/20" />
      </div>
    );
  }

  const ext = name.split('.').pop()?.toLowerCase() || '';

  // 图片类 - 苹果鲜亮绿
  if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-[#34C759]/10 flex items-center justify-center text-[#34C759] shadow-xs border border-[#34C759]/20">
        <FileImage size={size} />
      </div>
    );
  }

  // 视频类 - 苹果活力紫
  if (['mp4', 'mkv', 'mov', 'webm', 'avi', 'flv'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-[#AF52DE]/10 flex items-center justify-center text-[#AF52DE] shadow-xs border border-[#AF52DE]/20">
        <FileVideo size={size} />
      </div>
    );
  }

  // 音频类 - 苹果珊瑚红
  if (['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-[#FF3B30]/10 flex items-center justify-center text-[#FF3B30] shadow-xs border border-[#FF3B30]/20">
        <FileAudio size={size} />
      </div>
    );
  }

  // 代码文本类 - 苹果经典蓝
  if (['js', 'ts', 'tsx', 'jsx', 'rs', 'py', 'go', 'html', 'css', 'json', 'yaml', 'toml', 'sh', 'sql', 'c', 'cpp'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-[#007AFF]/10 flex items-center justify-center text-[#007AFF] shadow-xs border border-[#007AFF]/20">
        <FileCode size={size} />
      </div>
    );
  }

  // 表格类 - 苹果深翠绿
  if (['xlsx', 'xls', 'csv'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-emerald-500/10 flex items-center justify-center text-emerald-600 shadow-xs border border-emerald-500/20">
        <FileSpreadsheet size={size} />
      </div>
    );
  }

  // 压缩文件 - 苹果暖橙黄
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-[#FF9500]/10 flex items-center justify-center text-[#FF9500] shadow-xs border border-[#FF9500]/20">
        <FileArchive size={size} />
      </div>
    );
  }

  // PDF / 普通文档 - 苹果天青蓝
  if (['pdf', 'doc', 'docx', 'md', 'txt'].includes(ext)) {
    return (
      <div className="w-10 h-10 rounded-xl bg-sky-500/10 flex items-center justify-center text-sky-600 shadow-xs border border-sky-500/20">
        <FileText size={size} />
      </div>
    );
  }

  // 默认通用文件 - 灰黑纯净
  return (
    <div className="w-10 h-10 rounded-xl bg-slate-500/10 flex items-center justify-center text-slate-500 shadow-xs border border-slate-500/20">
      <GenericFile size={size} />
    </div>
  );
};
