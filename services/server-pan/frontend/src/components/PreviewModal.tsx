import React, { useEffect, useState } from 'react';
import { fileApi, formatBytes } from '../api';
import { FileItem } from '../types';
import {
  X,
  Download,
  FileText,
  FileQuestion,
  Loader2,
  Copy,
  Check,
} from 'lucide-react';

interface PreviewModalProps {
  isOpen: boolean;
  onClose: () => void;
  item: FileItem | null;
  onDownload: (item: FileItem) => void;
}

export const PreviewModal: React.FC<PreviewModalProps> = ({
  isOpen,
  onClose,
  item,
  onDownload,
}) => {
  const [textContent, setTextContent] = useState<string | null>(null);
  const [loadingText, setLoadingText] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!isOpen || !item || item.is_dir) {
      setTextContent(null);
      return;
    }

    const ext = item.name.split('.').pop()?.toLowerCase() || '';
    const textExts = [
      'txt', 'md', 'json', 'js', 'ts', 'tsx', 'jsx', 'rs', 'py', 'go',
      'html', 'css', 'yaml', 'yml', 'toml', 'sh', 'sql', 'c', 'cpp', 'h',
      'ini', 'env', 'log', 'xml'
    ];

    if (textExts.includes(ext) || item.mime_type.startsWith('text/')) {
      setLoadingText(true);
      fetch(fileApi.getPreviewUrl(item.id), {
        headers: {
          Authorization: `Bearer ${localStorage.getItem('pandown_token') || ''}`,
        },
      })
        .then((res) => res.text())
        .then((text) => setTextContent(text))
        .catch(() => setTextContent('预览加载失败'))
        .finally(() => setLoadingText(false));
    } else {
      setTextContent(null);
    }
  }, [isOpen, item]);

  if (!isOpen || !item) return null;

  const ext = item.name.split('.').pop()?.toLowerCase() || '';
  const isImage = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'bmp', 'ico'].includes(ext) || item.mime_type.startsWith('image/');
  const isVideo = ['mp4', 'webm', 'mov', 'mkv', 'avi'].includes(ext) || item.mime_type.startsWith('video/');
  const isAudio = ['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a'].includes(ext) || item.mime_type.startsWith('audio/');
  const isPdf = ext === 'pdf' || item.mime_type === 'application/pdf';

  const previewUrl = fileApi.getPreviewUrl(item.id);

  const copyText = () => {
    if (textContent) {
      navigator.clipboard.writeText(textContent);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/40 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden">

        {/* 顶部标题栏 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2.5 min-w-0">
            <div className="flex items-center gap-1.5 mr-1 shrink-0">
              <span className="apple-dot apple-dot-red" onClick={onClose} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <h3 className="font-bold text-slate-800 text-sm truncate" title={item.name}>
              {item.name}
            </h3>
            <span className="text-xs text-slate-400 font-medium shrink-0">
              ({formatBytes(item.size)})
            </span>
          </div>

          <div className="flex items-center gap-2 shrink-0">
            {textContent && (
              <button
                onClick={copyText}
                className="flex items-center gap-1 px-3 py-1.5 bg-white hover:bg-slate-50 border border-slate-200 rounded-xl text-xs font-semibold text-slate-700 shadow-2xs transition-all"
                title="复制文本"
              >
                {copied ? <Check size={13} className="text-[#34C759]" /> : <Copy size={13} />}
                {copied ? '已复制' : '复制内容'}
              </button>
            )}

            <button
              onClick={() => onDownload(item)}
              className="flex items-center gap-1.5 px-3 py-1.5 bg-[#007AFF] hover:bg-[#0071E3] text-white rounded-xl text-xs font-semibold shadow-xs transition-all"
            >
              <Download size={14} />
              下载
            </button>

            <button onClick={onClose} className="text-slate-400 hover:text-slate-600 rounded-lg p-1">
              <X size={18} />
            </button>
          </div>
        </div>

        {/* 预览展示区域 */}
        <div className="flex-1 overflow-auto p-6 bg-[#F5F5F7] flex items-center justify-center min-h-[360px]">
          {isImage ? (
            <div className="max-w-full max-h-[70vh] flex items-center justify-center overflow-hidden rounded-2xl bg-white p-2 shadow-xs border border-slate-200/80">
              <img
                src={previewUrl}
                alt={item.name}
                className="max-w-full max-h-[65vh] object-contain rounded-xl select-none"
              />
            </div>
          ) : isVideo ? (
            <div className="w-full max-w-3xl rounded-2xl overflow-hidden shadow-lg bg-black">
              <video
                controls
                autoPlay
                className="w-full max-h-[65vh]"
                src={previewUrl}
              >
                您的浏览器不支持视频播放。
              </video>
            </div>
          ) : isAudio ? (
            <div className="bg-white p-8 rounded-3xl shadow-sm border border-slate-200/80 text-center space-y-4 max-w-md w-full">
              <div className="w-16 h-16 rounded-2xl bg-[#FF3B30]/10 text-[#FF3B30] flex items-center justify-center mx-auto">
                <FileText size={32} />
              </div>
              <h4 className="font-bold text-slate-800 text-sm truncate">{item.name}</h4>
              <audio controls className="w-full mt-2" src={previewUrl}>
                您的浏览器不支持音频播放。
              </audio>
            </div>
          ) : isPdf ? (
            <div className="w-full h-[70vh] rounded-2xl overflow-hidden shadow-xs bg-white border border-slate-200">
              <iframe
                src={previewUrl}
                title={item.name}
                className="w-full h-full border-0"
              />
            </div>
          ) : textContent !== null ? (
            loadingText ? (
              <div className="text-center text-slate-400">
                <Loader2 size={24} className="animate-spin text-[#007AFF] mx-auto mb-2" />
                <p className="text-xs">加载文本中...</p>
              </div>
            ) : (
              <div className="w-full h-full max-h-[70vh] bg-white rounded-2xl p-4 shadow-xs border border-slate-200 overflow-auto font-mono text-xs text-slate-800 leading-relaxed whitespace-pre-wrap select-text">
                {textContent}
              </div>
            )
          ) : (
            <div className="text-center bg-white p-8 rounded-3xl border border-slate-200 shadow-xs max-w-sm w-full">
              <div className="w-16 h-16 rounded-2xl bg-slate-100 text-slate-400 flex items-center justify-center mx-auto mb-3">
                <FileQuestion size={32} />
              </div>
              <h4 className="font-bold text-slate-800 text-sm mb-1">{item.name}</h4>
              <p className="text-xs text-slate-400 mb-4">
                当前文件格式暂不支持在线直接渲染预览
              </p>
              <button
                onClick={() => onDownload(item)}
                className="w-full flex items-center justify-center gap-2 py-2.5 bg-[#007AFF] hover:bg-[#0071E3] text-white text-xs font-semibold rounded-xl transition-all shadow-xs"
              >
                <Download size={14} />
                下载至本地查看
              </button>
            </div>
          )}
        </div>

      </div>
    </div>
  );
};
