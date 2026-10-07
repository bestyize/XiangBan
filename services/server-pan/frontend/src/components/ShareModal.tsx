import React, { useState } from 'react';
import { shareApi } from '../api';
import { FileItem } from '../types';
import {
  Share2,
  X,
  Lock,
  Calendar,
  Copy,
  Check,
  AlertCircle,
  Loader2,
  Sparkles,
  Link as LinkIcon,
  FileText,
} from 'lucide-react';

interface ShareModalProps {
  isOpen: boolean;
  onClose: () => void;
  item: FileItem | null;
  onShareCreated?: () => void;
}

export const ShareModal: React.FC<ShareModalProps> = ({
  isOpen,
  onClose,
  item,
  onShareCreated,
}) => {
  const [needPassword, setNeedPassword] = useState(false);
  const [password, setPassword] = useState('');
  const [expireDays, setExpireDays] = useState<number | ''>('');
  const [note, setNote] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 成功生成分享后的状态
  const [shareResult, setShareResult] = useState<{
    code: string;
    fullUrl: string;
    hasPassword: boolean;
    password?: string;
    note?: string;
  } | null>(null);

  const [copied, setCopied] = useState(false);

  if (!isOpen || !item) return null;

  // 随机生成 4 位提取码
  const generateRandomPassword = () => {
    const chars = '23456789abcdefghjkmnpqrstuvwxyz';
    let res = '';
    for (let i = 0; i < 4; i++) {
      res += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    setPassword(res);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const pwd = needPassword ? password.trim() : undefined;
      const days = expireDays === '' ? undefined : Number(expireDays);
      const noteVal = note.trim() || undefined;

      const res = await shareApi.create(item.id, pwd, days, noteVal);
      const origin = window.location.origin;
      const fullUrl = `${origin}/#share/${res.code}`;

      setShareResult({
        code: res.code,
        fullUrl,
        hasPassword: res.has_password,
        password: res.password,
        note: res.note,
      });

      if (onShareCreated) onShareCreated();
    } catch (err: any) {
      setError(err.message || '创建分享链接失败');
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = () => {
    if (!shareResult) return;
    let textToCopy = `【PanDown 分享】${item.name}\n链接：${shareResult.fullUrl}`;
    if (shareResult.note) {
      textToCopy += `\n说明：${shareResult.note}`;
    }
    if (shareResult.hasPassword && shareResult.password) {
      textToCopy += `\n提取码：${shareResult.password}`;
    }

    navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleReset = () => {
    setShareResult(null);
    setNeedPassword(false);
    setPassword('');
    setExpireDays('');
    setNote('');
    setError(null);
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-lg overflow-hidden">

        {/* 顶部标题栏 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-1.5 mr-1">
              <span className="apple-dot apple-dot-red" onClick={handleReset} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <div className="w-7 h-7 rounded-lg bg-amber-500/10 text-[#FF9500] flex items-center justify-center">
              <Share2 size={16} />
            </div>
            <h3 className="font-bold text-slate-800 text-sm">
              分享 {item.is_dir ? '文件夹' : '文件'}
            </h3>
          </div>
          <button onClick={handleReset} className="text-slate-400 hover:text-slate-600 rounded-lg p-1">
            <X size={18} />
          </button>
        </div>

        {shareResult ? (
          /* 生成成功界面 */
          <div className="p-6 space-y-5 animate-in zoom-in-95 duration-200">
            <div className="w-12 h-12 rounded-2xl bg-[#34C759]/10 text-[#34C759] flex items-center justify-center mx-auto mb-2">
              <Check size={26} />
            </div>
            <div className="text-center">
              <h4 className="text-base font-bold text-slate-800">分享链接已生成！</h4>
              <p className="text-xs text-slate-400 mt-0.5">任何人均可通过此链接访问您共享的资源</p>
            </div>

            <div className="p-4 bg-[#F5F5F7] rounded-2xl border border-slate-200/80 space-y-3">
              <div>
                <span className="text-[11px] font-semibold text-slate-400 block mb-1">分享链接</span>
                <div className="flex items-center gap-2 bg-white px-3 py-2 rounded-xl border border-slate-200">
                  <LinkIcon size={14} className="text-slate-400 shrink-0" />
                  <span className="text-xs text-slate-700 font-mono truncate select-all flex-1">
                    {shareResult.fullUrl}
                  </span>
                </div>
              </div>

              {shareResult.hasPassword && shareResult.password && (
                <div className="flex items-center justify-between bg-white px-3.5 py-2.5 rounded-xl border border-slate-200">
                  <span className="text-xs text-slate-500 font-medium flex items-center gap-1.5">
                    <Lock size={14} className="text-[#FF9500]" />
                    提取码 (访问密码)
                  </span>
                  <span className="text-sm font-bold font-mono text-[#007AFF] bg-[#007AFF]/10 px-2.5 py-0.5 rounded-lg">
                    {shareResult.password}
                  </span>
                </div>
              )}

              {shareResult.note && (
                <div className="bg-white px-3.5 py-2.5 rounded-xl border border-slate-200">
                  <span className="text-[11px] font-semibold text-slate-400 block mb-1 flex items-center gap-1">
                    <FileText size={12} className="text-[#007AFF]" />
                    分享说明
                  </span>
                  <p className="text-xs text-slate-700 whitespace-pre-wrap leading-relaxed">
                    {shareResult.note}
                  </p>
                </div>
              )}
            </div>

            <div className="flex items-center justify-end gap-2.5 pt-2">
              <button
                onClick={handleReset}
                className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-800 hover:bg-slate-100 rounded-xl transition-all"
              >
                完成
              </button>
              <button
                onClick={copyToClipboard}
                className="flex items-center gap-2 px-5 py-2 text-sm font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] rounded-xl shadow-xs transition-all"
              >
                {copied ? <Check size={16} /> : <Copy size={16} />}
                {copied ? '已复制到剪贴板！' : '复制分享信息'}
              </button>
            </div>
          </div>
        ) : (
          /* 创建分享表单 */
          <form onSubmit={handleCreate} className="p-6 space-y-5">
            {error && (
              <div className="p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
                <AlertCircle size={16} className="shrink-0" />
                <span>{error}</span>
              </div>
            )}

            {/* 分享对象预览 */}
            <div className="p-3 bg-[#F5F5F7] rounded-xl flex items-center gap-2.5 border border-slate-200/60">
              <div className="text-xs font-semibold text-slate-500">分享目标:</div>
              <div className="text-xs font-bold text-slate-800 truncate flex-1" title={item.name}>
                {item.name}
              </div>
              <span className="text-[11px] text-slate-400">
                {item.is_dir ? '文件夹' : '文件'}
              </span>
            </div>

            {/* 提取码开关 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5 cursor-pointer">
                  <Lock size={14} className="text-[#FF9500]" />
                  <span>启用提取码保护</span>
                </label>
                <input
                  type="checkbox"
                  checked={needPassword}
                  onChange={(e) => {
                    setNeedPassword(e.target.checked);
                    if (e.target.checked && !password) {
                      generateRandomPassword();
                    }
                  }}
                  className="w-4 h-4 text-[#007AFF] rounded focus:ring-[#007AFF]"
                />
              </div>

              {needPassword && (
                <div className="flex items-center gap-2 animate-in fade-in duration-150">
                  <input
                    type="text"
                    maxLength={10}
                    placeholder="请输入提取码 (如 1234 或 abcd)"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="flex-1 px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm font-mono outline-none text-slate-800"
                  />
                  <button
                    type="button"
                    onClick={generateRandomPassword}
                    className="flex items-center gap-1 px-3 py-2 text-xs font-semibold text-[#007AFF] bg-[#007AFF]/10 hover:bg-[#007AFF]/20 rounded-xl transition-all"
                  >
                    <Sparkles size={13} />
                    随机生成
                  </button>
                </div>
              )}
            </div>

            {/* 有效期限 */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5">
                <Calendar size={14} className="text-[#34C759]" />
                <span>有效期设置</span>
              </label>
              <select
                value={expireDays}
                onChange={(e) => setExpireDays(e.target.value === '' ? '' : Number(e.target.value))}
                className="w-full px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none text-slate-800"
              >
                <option value="">永久有效</option>
                <option value={1}>1 天有效</option>
                <option value={7}>7 天有效</option>
                <option value={30}>30 天有效</option>
              </select>
            </div>

            {/* 分享信息 / 说明（可选） */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-slate-700 flex items-center gap-1.5">
                <FileText size={14} className="text-[#007AFF]" />
                <span>分享说明 / 留言（可选）</span>
              </label>
              <textarea
                rows={2}
                maxLength={200}
                placeholder="例如：这是项目终稿设计，提取后请先阅读 README..."
                value={note}
                onChange={(e) => setNote(e.target.value)}
                className="w-full px-3.5 py-2 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-xs outline-none text-slate-800 resize-none transition-all placeholder:text-slate-400"
              />
            </div>

            {/* 底部操作按钮 */}
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
                disabled={loading || (needPassword && !password.trim())}
                className="flex items-center gap-2 px-5 py-2 text-sm font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 disabled:cursor-not-allowed rounded-xl shadow-xs transition-all"
              >
                {loading && <Loader2 size={16} className="animate-spin" />}
                创建分享
              </button>
            </div>
          </form>
        )}

      </div>
    </div>
  );
};
