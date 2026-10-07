import React, { useEffect, useState, useCallback } from 'react';
import { publicApi, formatBytes, formatDate } from '../api';
import { ShareContentResponse, ShareInfoResponse, FileItem } from '../types';
import { FileIcon } from '../components/FileIcon';
import { PreviewModal } from '../components/PreviewModal';
import {
  Cloud,
  Lock,
  Download,
  Eye,
  Folder,
  AlertCircle,
  Loader2,
  Clock,
  User,
  ChevronRight,
  Home,
  ShieldCheck,
  FileText,
} from 'lucide-react';

interface PublicSharePageProps {
  code: string;
}

export const PublicSharePage: React.FC<PublicSharePageProps> = ({ code }) => {
  const [info, setInfo] = useState<ShareInfoResponse | null>(null);
  const [content, setContent] = useState<ShareContentResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 提取码输入
  const [password, setPassword] = useState('');
  const [verifying, setVerifying] = useState(false);
  const [verifyError, setVerifyError] = useState<string | null>(null);

  // 访问凭证
  const [shareToken, setShareToken] = useState<string | null>(() => {
    return sessionStorage.getItem(`pandown_share_token_${code}`);
  });

  // 文件夹内子目录浏览
  const [subFolderId, setSubFolderId] = useState<string | null>(null);

  // 预览状态
  const [previewTarget, setPreviewTarget] = useState<FileItem | null>(null);

  // 加载分享基础信息
  const loadInfo = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await publicApi.getInfo(code);
      setInfo(data);
    } catch (err: any) {
      setError(err.message || '分享链接不存在或已失效');
    } finally {
      setLoading(false);
    }
  }, [code]);

  // 加载分享内容（免密或已验证密码后）
  const loadContent = useCallback(async () => {
    try {
      const data = await publicApi.getContent(code, subFolderId, shareToken);
      setContent(data);
    } catch (err: any) {
      // 如果报错需要密码或凭证失效
      if (err.message?.includes('密码') || err.message?.includes('凭据')) {
        setShareToken(null);
        sessionStorage.removeItem(`pandown_share_token_${code}`);
      } else {
        setError(err.message || '加载分享内容失败');
      }
    }
  }, [code, subFolderId, shareToken]);

  useEffect(() => {
    loadInfo();
  }, [loadInfo]);

  useEffect(() => {
    if (info && (!info.has_password || shareToken)) {
      loadContent();
    }
  }, [info, shareToken, subFolderId, loadContent]);

  // 验证提取码
  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!password.trim()) return;

    setVerifying(true);
    setVerifyError(null);

    try {
      const res = await publicApi.verify(code, password.trim());
      setShareToken(res.share_token);
      sessionStorage.setItem(`pandown_share_token_${code}`, res.share_token);
    } catch (err: any) {
      setVerifyError(err.message || '提取码不正确');
    } finally {
      setVerifying(false);
    }
  };

  const handleDownloadFile = (fileItem: FileItem) => {
    const url = publicApi.getDownloadUrl(code, fileItem.id, shareToken);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileItem.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-[#F5F5F7] flex flex-col items-center justify-center">
        <Loader2 size={32} className="animate-spin text-[#007AFF] mb-3" />
        <p className="text-sm font-semibold text-slate-600">正在获取分享信息...</p>
      </div>
    );
  }

  if (error || !info) {
    return (
      <div className="min-h-screen bg-[#F5F5F7] flex flex-col items-center justify-center p-4">
        <div className="bg-white p-8 rounded-3xl shadow-xl border border-slate-200/80 max-w-md w-full text-center space-y-4">
          <div className="w-16 h-16 rounded-2xl bg-[#FF3B30]/10 text-[#FF3B30] flex items-center justify-center mx-auto">
            <AlertCircle size={32} />
          </div>
          <h2 className="text-lg font-bold text-slate-800">无法访问此分享</h2>
          <p className="text-xs text-slate-400">{error || '该分享可能已被创建者取消或已过期'}</p>
          <a
            href="/"
            className="inline-block px-5 py-2.5 bg-[#007AFF] text-white text-xs font-semibold rounded-xl hover:bg-[#0071E3] transition-all shadow-xs"
          >
            返回 PanDown 首页
          </a>
        </div>
      </div>
    );
  }

  // 1. 如果需要密码且尚未验证：展示 Apple 锁屏提取码界面
  if (info.has_password && !shareToken) {
    return (
      <div className="min-h-screen bg-[#F5F5F7] flex flex-col items-center justify-center p-4 relative selection:bg-[#007AFF]/20 selection:text-[#007AFF]">
        <div className="w-full max-w-md bg-white rounded-3xl shadow-xl border border-slate-200/80 overflow-hidden relative z-10">

          <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
            <div className="flex items-center gap-2">
              <span className="apple-dot apple-dot-red" />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <span className="text-[11px] font-semibold uppercase text-slate-400">
              加密分享验证
            </span>
          </div>

          <div className="p-8">
            <div className="text-center mb-6">
              <div className="w-14 h-14 rounded-2xl bg-amber-500/10 text-[#FF9500] flex items-center justify-center mx-auto mb-3 shadow-xs">
                <Lock size={28} />
              </div>
              <h2 className="text-xl font-bold text-slate-800">此分享受密码保护</h2>
              <p className="text-xs text-slate-400 mt-1 truncate px-2" title={info.target_name}>
                「{info.target_name}」
              </p>
              {info.note && (
                <div className="mt-3 text-xs text-slate-600 bg-[#F5F5F7] p-2.5 rounded-xl border border-slate-200/60 text-left flex items-start gap-1.5">
                  <FileText size={13} className="text-[#007AFF] mt-0.5 shrink-0" />
                  <span className="whitespace-pre-wrap break-all leading-relaxed">{info.note}</span>
                </div>
              )}
            </div>

            {verifyError && (
              <div className="mb-4 p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
                <AlertCircle size={16} className="shrink-0" />
                <span>{verifyError}</span>
              </div>
            )}

            <form onSubmit={handleVerify} className="space-y-4">
              <div>
                <label className="block text-xs font-semibold text-slate-600 mb-1.5">
                  请输入提取码
                </label>
                <input
                  type="text"
                  required
                  autoFocus
                  placeholder="请输入提取码"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full px-4 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm font-mono outline-none text-slate-800 text-center tracking-widest text-base font-bold"
                />
              </div>

              <button
                type="submit"
                disabled={verifying || !password.trim()}
                className="w-full py-3 bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 text-white text-sm font-semibold rounded-xl shadow-md shadow-[#007AFF]/25 transition-all flex items-center justify-center gap-2 cursor-pointer"
              >
                {verifying ? <Loader2 size={16} className="animate-spin" /> : <ShieldCheck size={16} />}
                验证并提取文件
              </button>
            </form>

            <div className="mt-6 pt-4 border-t border-slate-100 flex items-center justify-between text-xs text-slate-400">
              <span className="flex items-center gap-1">
                <User size={13} />
                分享者: {info.creator_name}
              </span>
              <span>{formatDate(info.created_at)}</span>
            </div>
          </div>

        </div>
      </div>
    );
  }

  // 2. 正常浏览与提取界面
  return (
    <div className="min-h-screen bg-[#F5F5F7] flex flex-col selection:bg-[#007AFF]/20 selection:text-[#007AFF]">
      {/* 顶部导航栏 */}
      <header className="sticky top-0 z-30 bg-white/80 backdrop-blur-md border-b border-black/5 shadow-xs">
        <div className="max-w-6xl mx-auto px-4 sm:px-6 h-16 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5 pr-2 border-r border-slate-200">
              <span className="apple-dot apple-dot-red" />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <a href="/" className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-xl bg-[#007AFF] text-white flex items-center justify-center shadow-xs">
                <Cloud size={18} className="fill-white/20" />
              </div>
              <span className="font-bold text-slate-800 text-base">PanDown 分享</span>
            </a>
          </div>

          <div className="flex items-center gap-3 text-xs text-slate-500">
            <span className="flex items-center gap-1">
              <User size={13} className="text-[#007AFF]" />
              {info.creator_name} 共享
            </span>
            {info.expires_at && (
              <span className="hidden sm:flex items-center gap-1 text-slate-400">
                <Clock size={13} />
                {formatDate(info.expires_at)} 到期
              </span>
            )}
          </div>
        </div>
      </header>

      {/* 主体内容 */}
      <main className="flex-1 max-w-6xl w-full mx-auto px-4 sm:px-6 py-8">

        {/* 资源头图展示卡 */}
        <div className="bg-white rounded-3xl p-6 sm:p-8 shadow-xs border border-slate-200/80 mb-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div className="flex items-center gap-4">
            <FileIcon isDir={info.is_dir} name={info.target_name} size={32} />
            <div>
              <h1 className="text-xl font-bold text-slate-900 break-all">
                {info.target_name}
              </h1>
              <div className="flex items-center gap-3 text-xs text-slate-400 mt-1 font-medium">
                <span>{info.is_dir ? '共享文件夹' : formatBytes(info.file_size)}</span>
                <span>•</span>
                <span>分享时间: {formatDate(info.created_at)}</span>
              </div>
            </div>
          </div>

          {/* 如果是单文件，右侧直接展示主下载与预览按钮 */}
          {!info.is_dir && content && content.items.length > 0 && (
            <div className="flex items-center gap-2.5 w-full sm:w-auto">
              <button
                onClick={() => setPreviewTarget(content.items[0])}
                className="flex-1 sm:flex-none flex items-center justify-center gap-1.5 px-4 py-2.5 bg-slate-100 hover:bg-slate-200 text-slate-700 text-xs font-semibold rounded-xl transition-all"
              >
                <Eye size={15} />
                在线预览
              </button>
              <button
                onClick={() => handleDownloadFile(content.items[0])}
                className="flex-1 sm:flex-none flex items-center justify-center gap-2 px-6 py-2.5 bg-[#007AFF] hover:bg-[#0071E3] text-white text-xs font-semibold rounded-xl shadow-xs shadow-[#007AFF]/25 transition-all cursor-pointer"
              >
                <Download size={15} />
                立即下载
              </button>
            </div>
          )}
        </div>

        {/* 分享留言 / 说明展示 */}
        {info.note && (
          <div className="bg-white rounded-2xl p-4 shadow-xs border border-slate-200/80 mb-6 flex items-start gap-3">
            <div className="w-8 h-8 rounded-xl bg-[#007AFF]/10 text-[#007AFF] flex items-center justify-center shrink-0 mt-0.5">
              <FileText size={16} />
            </div>
            <div className="flex-1 min-w-0">
              <span className="text-xs font-bold text-slate-700 block mb-0.5">
                分享留言 / 补充说明
              </span>
              <p className="text-xs text-slate-600 leading-relaxed whitespace-pre-wrap break-words">
                {info.note}
              </p>
            </div>
          </div>
        )}

        {/* 如果是文件夹分享：展示文件夹内部列表和穿透面包屑 */}
        {info.is_dir && content && (
          <div className="space-y-4">
            {/* 面包屑导航 */}
            <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-600 bg-white/70 px-4 py-2 rounded-xl border border-slate-200/60 overflow-x-auto">
              {content.breadcrumbs.map((b, idx) => {
                const isLast = idx === content.breadcrumbs.length - 1;
                return (
                  <React.Fragment key={b.id || idx}>
                    {idx > 0 && <ChevronRight size={13} className="text-slate-400 shrink-0" />}
                    <button
                      onClick={() => setSubFolderId(b.id)}
                      disabled={isLast}
                      className={`flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors ${
                        isLast ? 'text-slate-900 font-bold' : 'text-[#007AFF] hover:underline cursor-pointer'
                      }`}
                    >
                      {idx === 0 ? <Home size={13} /> : <Folder size={13} className="text-amber-500" />}
                      <span>{b.name}</span>
                    </button>
                  </React.Fragment>
                );
              })}
            </div>

            {/* 文件清单列表 */}
            <div className="bg-white rounded-3xl border border-slate-200/80 shadow-xs overflow-hidden">
              <table className="w-full text-left border-collapse text-sm">
                <thead>
                  <tr className="border-b border-slate-100 text-slate-400 font-semibold text-xs bg-slate-50/50">
                    <th className="py-3 px-4 w-12 text-center">类型</th>
                    <th className="py-3 px-4">名称</th>
                    <th className="py-3 px-4 w-32 hidden sm:table-cell">大小</th>
                    <th className="py-3 px-4 w-44 hidden md:table-cell">修改时间</th>
                    <th className="py-3 px-4 w-40 text-right pr-6">操作</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {content.items.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="py-12 text-center text-slate-400 text-xs">
                        该文件夹内暂无文件
                      </td>
                    </tr>
                  ) : (
                    content.items.map((item) => (
                      <tr
                        key={item.id}
                        className="hover:bg-[#F5F5F7] transition-colors cursor-pointer"
                        onClick={() => {
                          if (item.is_dir) {
                            setSubFolderId(item.id);
                          } else {
                            setPreviewTarget(item);
                          }
                        }}
                      >
                        <td className="py-2.5 px-4 text-center">
                          <div className="inline-flex items-center justify-center">
                            <FileIcon isDir={item.is_dir} name={item.name} size={18} />
                          </div>
                        </td>
                        <td className="py-2.5 px-4">
                          <span className={`font-medium ${item.is_dir ? 'text-slate-900 font-bold hover:text-[#007AFF]' : 'text-slate-800'}`}>
                            {item.name}
                          </span>
                        </td>
                        <td className="py-2.5 px-4 text-slate-400 text-xs hidden sm:table-cell">
                          {item.is_dir ? '-' : formatBytes(item.size)}
                        </td>
                        <td className="py-2.5 px-4 text-slate-400 text-xs hidden md:table-cell">
                          {formatDate(item.updated_at)}
                        </td>
                        <td className="py-2.5 px-4 text-right pr-6" onClick={(e) => e.stopPropagation()}>
                          <div className="flex items-center justify-end gap-1.5">
                            {!item.is_dir && (
                              <>
                                <button
                                  onClick={() => setPreviewTarget(item)}
                                  title="在线预览"
                                  className="p-1.5 text-slate-500 hover:text-[#007AFF] hover:bg-[#007AFF]/10 rounded-lg transition-all"
                                >
                                  <Eye size={16} />
                                </button>
                                <button
                                  onClick={() => handleDownloadFile(item)}
                                  title="下载文件"
                                  className="flex items-center gap-1 px-2.5 py-1 text-xs font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] rounded-lg transition-all shadow-2xs"
                                >
                                  <Download size={13} />
                                  下载
                                </button>
                              </>
                            )}
                            {item.is_dir && (
                              <button
                                onClick={() => setSubFolderId(item.id)}
                                className="text-xs font-semibold text-[#007AFF] hover:underline px-2 py-1"
                              >
                                打开目录 →
                              </button>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        )}

      </main>

      {/* 预览弹窗 */}
      <PreviewModal
        isOpen={!!previewTarget}
        item={previewTarget}
        onClose={() => setPreviewTarget(null)}
        onDownload={handleDownloadFile}
      />
    </div>
  );
};
