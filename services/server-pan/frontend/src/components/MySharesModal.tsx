import React, { useEffect, useState } from 'react';
import { shareApi, formatDate, formatBytes } from '../api';
import { ShareDto, UpdateShareRequest } from '../types';
import {
  Share2,
  X,
  Trash2,
  Copy,
  Check,
  Lock,
  ExternalLink,
  Eye,
  Download,
  Clock,
  Folder,
  File as FileIcon,
  Loader2,
  Edit3,
  FileText,
  AlertCircle,
  Calendar,
  Sparkles,
} from 'lucide-react';

interface MySharesModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSharesChanged?: () => void;
}

export const MySharesModal: React.FC<MySharesModalProps> = ({
  isOpen,
  onClose,
  onSharesChanged,
}) => {
  const [shares, setShares] = useState<ShareDto[]>([]);
  const [loading, setLoading] = useState(true);
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  // 编辑弹窗状态
  const [editingShare, setEditingShare] = useState<ShareDto | null>(null);
  const [editNote, setEditNote] = useState('');
  const [editPasswordMode, setEditPasswordMode] = useState<'keep' | 'change' | 'clear'>('keep');
  const [editPassword, setEditPassword] = useState('');
  const [editExpireMode, setEditExpireMode] = useState<'keep' | 'permanent' | '1d' | '7d' | '30d'>('keep');
  const [editLoading, setEditLoading] = useState(false);
  const [editError, setEditError] = useState<string | null>(null);

  const fetchShares = async () => {
    setLoading(true);
    try {
      const data = await shareApi.getMyShares();
      setShares(data);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isOpen) {
      fetchShares();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const handleDelete = async (code: string) => {
    if (!confirm('确定要取消此分享吗？取消后该链接将立即失效。')) return;
    try {
      await shareApi.delete(code);
      setShares((prev) => prev.filter((s) => s.code !== code));
      if (onSharesChanged) onSharesChanged();
    } catch (err: any) {
      alert(err.message || '操作失败');
    }
  };

  const copyShare = (share: ShareDto) => {
    const origin = window.location.origin;
    let text = `【PanDown 分享】${share.file_name}\n链接：${origin}/#share/${share.code}`;
    if (share.note) {
      text += `\n说明：${share.note}`;
    }
    if (share.has_password && share.password) {
      text += `\n提取码：${share.password}`;
    }
    navigator.clipboard.writeText(text);
    setCopiedCode(share.code);
    setTimeout(() => setCopiedCode(null), 2000);
  };

  const handleOpenEdit = (share: ShareDto) => {
    setEditingShare(share);
    setEditNote(share.note || '');
    setEditPasswordMode('keep');
    setEditPassword(share.password || '');
    setEditExpireMode('keep');
    setEditError(null);
  };

  const generateRandomPassword = () => {
    const chars = '23456789abcdefghjkmnpqrstuvwxyz';
    let res = '';
    for (let i = 0; i < 4; i++) {
      res += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    setEditPassword(res);
  };

  const handleSaveEdit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!editingShare) return;

    setEditLoading(true);
    setEditError(null);

    try {
      const payload: UpdateShareRequest = {};

      // 说明更新（传空串表示清空，传文本表示更新）
      payload.note = editNote.trim();

      // 提取码更新
      if (editPasswordMode === 'clear') {
        payload.password = '';
      } else if (editPasswordMode === 'change') {
        if (!editPassword.trim()) {
          setEditError('请输入新的提取码或选择保持不变');
          setEditLoading(false);
          return;
        }
        payload.password = editPassword.trim();
      }

      // 有效期更新
      if (editExpireMode === 'permanent') {
        payload.expire_days = 0;
      } else if (editExpireMode === '1d') {
        payload.expire_days = 1;
      } else if (editExpireMode === '7d') {
        payload.expire_days = 7;
      } else if (editExpireMode === '30d') {
        payload.expire_days = 30;
      }

      await shareApi.update(editingShare.code, payload);
      await fetchShares();
      if (onSharesChanged) onSharesChanged();
      setEditingShare(null);
    } catch (err: any) {
      setEditError(err.message || '更新分享信息失败');
    } finally {
      setEditLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-2xl overflow-hidden flex flex-col max-h-[85vh] relative">

        {/* 顶部标题栏 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-1.5 mr-1">
              <span className="apple-dot apple-dot-red" onClick={onClose} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <div className="w-7 h-7 rounded-lg bg-amber-500/10 text-[#FF9500] flex items-center justify-center">
              <Share2 size={16} />
            </div>
            <h3 className="font-bold text-slate-800 text-sm">我创建的分享</h3>
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 rounded-lg p-1">
            <X size={18} />
          </button>
        </div>

        {/* 列表主体 */}
        <div className="p-6 flex-1 overflow-y-auto space-y-3">
          {loading ? (
            <div className="py-20 text-center flex flex-col items-center justify-center text-slate-400">
              <Loader2 size={24} className="animate-spin text-[#007AFF] mb-2" />
              <p className="text-xs">正在加载分享列表...</p>
            </div>
          ) : shares.length === 0 ? (
            <div className="py-20 text-center flex flex-col items-center justify-center text-slate-400">
              <div className="w-12 h-12 rounded-2xl bg-amber-500/10 flex items-center justify-center text-amber-500 mb-2">
                <Share2 size={22} />
              </div>
              <p className="text-sm font-semibold text-slate-700">暂无任何外链分享</p>
              <p className="text-xs text-slate-400 mt-1">您可以在文件列表中选择文件或文件夹进行分享</p>
            </div>
          ) : (
            shares.map((s) => (
              <div
                key={s.code}
                className="p-4 bg-[#F5F5F7] rounded-2xl border border-slate-200/80 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 hover:border-slate-300 transition-all"
              >
                <div className="flex items-start gap-3 min-w-0 flex-1">
                  <div className="w-9 h-9 rounded-xl bg-white border border-slate-200 flex items-center justify-center shrink-0 text-slate-600 shadow-2xs">
                    {s.is_dir ? <Folder size={18} className="text-amber-500" /> : <FileIcon size={18} className="text-[#007AFF]" />}
                  </div>

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-bold text-sm text-slate-800 truncate" title={s.file_name}>
                        {s.file_name}
                      </span>
                      {s.is_expired && (
                        <span className="text-[10px] font-bold px-1.5 py-0.2 rounded-md bg-[#FF3B30]/10 text-[#FF3B30]">
                          已过期
                        </span>
                      )}
                    </div>

                    <div className="flex flex-wrap items-center gap-3 text-xs text-slate-400 mt-1">
                      <span>{s.is_dir ? '文件夹' : formatBytes(s.file_size)}</span>
                      <span>创建于 {formatDate(s.created_at)}</span>

                      {s.has_password && s.password ? (
                        <span className="flex items-center gap-1 font-mono text-[#007AFF] bg-white px-1.5 py-0.5 rounded border border-slate-200">
                          <Lock size={11} className="text-[#FF9500]" />
                          提取码: <strong>{s.password}</strong>
                        </span>
                      ) : (
                        <span className="text-[#34C759] font-medium">免密公开</span>
                      )}

                      {s.expires_at ? (
                        <span className="flex items-center gap-1 text-slate-500">
                          <Clock size={11} />
                          到期: {formatDate(s.expires_at)}
                        </span>
                      ) : (
                        <span className="text-slate-400">永久有效</span>
                      )}
                    </div>

                    {/* 分享留言 / 说明展示 */}
                    {s.note && (
                      <div className="mt-2 text-xs text-slate-700 bg-white px-3 py-1.5 rounded-xl border border-slate-200/60 flex items-start gap-1.5 shadow-2xs">
                        <FileText size={13} className="text-[#007AFF] mt-0.5 shrink-0" />
                        <span className="whitespace-pre-wrap break-all leading-relaxed">
                          {s.note}
                        </span>
                      </div>
                    )}

                    <div className="flex items-center gap-4 text-xs text-slate-500 mt-2 font-medium">
                      <span className="flex items-center gap-1">
                        <Eye size={12} className="text-slate-400" />
                        浏览 {s.view_count} 次
                      </span>
                      <span className="flex items-center gap-1">
                        <Download size={12} className="text-slate-400" />
                        下载 {s.download_count} 次
                      </span>
                    </div>
                  </div>
                </div>

                {/* 快捷动作 */}
                <div className="flex items-center gap-1.5 self-end sm:self-center shrink-0">
                  <button
                    onClick={() => copyShare(s)}
                    title="复制链接"
                    className="flex items-center gap-1 px-2.5 py-1.5 bg-white hover:bg-slate-50 border border-slate-200 rounded-xl text-xs font-semibold text-slate-700 shadow-2xs transition-all"
                  >
                    {copiedCode === s.code ? <Check size={13} className="text-[#34C759]" /> : <Copy size={13} />}
                    {copiedCode === s.code ? '已复制' : '复制'}
                  </button>

                  <button
                    onClick={() => handleOpenEdit(s)}
                    title="编辑分享信息"
                    className="p-1.5 bg-white hover:bg-slate-50 border border-slate-200 rounded-xl text-slate-500 hover:text-[#007AFF] transition-all"
                  >
                    <Edit3 size={14} />
                  </button>

                  <a
                    href={`/#share/${s.code}`}
                    target="_blank"
                    rel="noreferrer"
                    title="新窗口打开"
                    className="p-1.5 bg-white hover:bg-slate-50 border border-slate-200 rounded-xl text-slate-500 hover:text-[#007AFF] transition-all"
                  >
                    <ExternalLink size={14} />
                  </a>

                  <button
                    onClick={() => handleDelete(s.code)}
                    title="取消分享"
                    className="p-1.5 bg-white hover:bg-[#FF3B30]/10 border border-slate-200 hover:border-[#FF3B30]/30 rounded-xl text-slate-400 hover:text-[#FF3B30] transition-all"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>

        {/* 编辑分享信息模态浮层 */}
        {editingShare && (
          <div className="fixed inset-0 z-60 flex items-center justify-center p-4 bg-black/40 backdrop-blur-xs animate-in fade-in duration-150">
            <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-md overflow-hidden animate-in zoom-in-95 duration-150">
              {/* 编辑窗口顶栏 */}
              <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
                <div className="flex items-center gap-2.5">
                  <div className="flex items-center gap-1.5 mr-1">
                    <span className="apple-dot apple-dot-red" onClick={() => setEditingShare(null)} />
                    <span className="apple-dot apple-dot-yellow" />
                    <span className="apple-dot apple-dot-green" />
                  </div>
                  <div className="w-7 h-7 rounded-lg bg-blue-500/10 text-[#007AFF] flex items-center justify-center">
                    <Edit3 size={15} />
                  </div>
                  <h3 className="font-bold text-slate-800 text-sm">编辑分享信息</h3>
                </div>
                <button
                  onClick={() => setEditingShare(null)}
                  className="text-slate-400 hover:text-slate-600 rounded-lg p-1"
                >
                  <X size={18} />
                </button>
              </div>

              {/* 编辑表单 */}
              <form onSubmit={handleSaveEdit} className="p-6 space-y-4">
                {editError && (
                  <div className="p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
                    <AlertCircle size={16} className="shrink-0" />
                    <span>{editError}</span>
                  </div>
                )}

                {/* 目标文件提示 */}
                <div className="p-3 bg-[#F5F5F7] rounded-xl flex items-center gap-2 border border-slate-200/60 text-xs">
                  <span className="text-slate-400 font-medium">分享对象:</span>
                  <span className="font-bold text-slate-800 truncate flex-1">
                    {editingShare.file_name}
                  </span>
                  <span className="font-mono text-slate-400">{editingShare.code}</span>
                </div>

                {/* 分享留言 / 说明编辑 */}
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5">
                    <FileText size={14} className="text-[#007AFF]" />
                    <span>分享说明 / 留言（清空则移除说明）</span>
                  </label>
                  <textarea
                    rows={2}
                    maxLength={200}
                    placeholder="可填写对文件或文件夹的说明留言..."
                    value={editNote}
                    onChange={(e) => setEditNote(e.target.value)}
                    className="w-full px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-xs outline-none text-slate-800 resize-none transition-all placeholder:text-slate-400"
                  />
                </div>

                {/* 提取码修改 */}
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5">
                    <Lock size={14} className="text-[#FF9500]" />
                    <span>提取码安全设置</span>
                  </label>
                  <div className="grid grid-cols-3 gap-2 text-xs">
                    <button
                      type="button"
                      onClick={() => setEditPasswordMode('keep')}
                      className={`py-1.5 px-2 rounded-xl border text-center transition-all ${
                        editPasswordMode === 'keep'
                          ? 'border-[#007AFF] bg-[#007AFF]/10 text-[#007AFF] font-bold'
                          : 'border-slate-200 text-slate-600 hover:bg-slate-50'
                      }`}
                    >
                      保持当前
                    </button>
                    <button
                      type="button"
                      onClick={() => {
                        setEditPasswordMode('change');
                        if (!editPassword) generateRandomPassword();
                      }}
                      className={`py-1.5 px-2 rounded-xl border text-center transition-all ${
                        editPasswordMode === 'change'
                          ? 'border-[#007AFF] bg-[#007AFF]/10 text-[#007AFF] font-bold'
                          : 'border-slate-200 text-slate-600 hover:bg-slate-50'
                      }`}
                    >
                      修改提取码
                    </button>
                    <button
                      type="button"
                      onClick={() => setEditPasswordMode('clear')}
                      className={`py-1.5 px-2 rounded-xl border text-center transition-all ${
                        editPasswordMode === 'clear'
                          ? 'border-[#34C759] bg-[#34C759]/10 text-[#34C759] font-bold'
                          : 'border-slate-200 text-slate-600 hover:bg-slate-50'
                      }`}
                    >
                      取消密码(公开)
                    </button>
                  </div>

                  {editPasswordMode === 'change' && (
                    <div className="flex items-center gap-2 mt-2 pt-1 animate-in fade-in duration-150">
                      <input
                        type="text"
                        maxLength={10}
                        placeholder="输入新提取码"
                        value={editPassword}
                        onChange={(e) => setEditPassword(e.target.value)}
                        className="flex-1 px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-xs font-mono outline-none text-slate-800"
                      />
                      <button
                        type="button"
                        onClick={generateRandomPassword}
                        className="flex items-center gap-1 px-3 py-2 text-xs font-semibold text-[#007AFF] bg-[#007AFF]/10 hover:bg-[#007AFF]/20 rounded-xl transition-all shrink-0"
                      >
                        <Sparkles size={12} />
                        随机
                      </button>
                    </div>
                  )}
                </div>

                {/* 有效期设置 */}
                <div className="space-y-1.5">
                  <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5">
                    <Calendar size={14} className="text-[#34C759]" />
                    <span>有效期设置</span>
                  </label>
                  <select
                    value={editExpireMode}
                    onChange={(e) => setEditExpireMode(e.target.value as any)}
                    className="w-full px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-xs outline-none text-slate-800"
                  >
                    <option value="keep">保持当前有效期</option>
                    <option value="permanent">改为永久有效</option>
                    <option value="1d">延长至 1 天后到期</option>
                    <option value="7d">延长至 7 天后到期</option>
                    <option value="30d">延长至 30 天后到期</option>
                  </select>
                </div>

                {/* 底部按钮 */}
                <div className="flex items-center justify-end gap-2 pt-2">
                  <button
                    type="button"
                    onClick={() => setEditingShare(null)}
                    disabled={editLoading}
                    className="px-4 py-2 text-xs font-medium text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all"
                  >
                    取消
                  </button>
                  <button
                    type="submit"
                    disabled={editLoading}
                    className="flex items-center gap-2 px-5 py-2 text-xs font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 rounded-xl shadow-xs transition-all"
                  >
                    {editLoading && <Loader2 size={14} className="animate-spin" />}
                    保存修改
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

      </div>
    </div>
  );
};
