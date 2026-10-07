import React, { useState } from 'react';
import { authApi } from '../api';
import { KeyRound, X, AlertCircle, CheckCircle2, Loader2 } from 'lucide-react';

interface ChangePasswordModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ChangePasswordModal: React.FC<ChangePasswordModalProps> = ({
  isOpen,
  onClose,
}) => {
  const [oldPassword, setOldPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPassword !== confirmPassword) {
      setError('两次输入的新密码不一致');
      return;
    }
    if (newPassword.length < 6) {
      setError('新密码长度不能少于 6 位');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await authApi.changePassword(oldPassword, newPassword);
      setSuccess(true);
      setTimeout(() => {
        setSuccess(false);
        setOldPassword('');
        setNewPassword('');
        setConfirmPassword('');
        onClose();
      }, 1500);
    } catch (err: any) {
      setError(err.message || '密码修改失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/30 backdrop-blur-xs animate-in fade-in duration-150">
      <div className="bg-white rounded-3xl shadow-2xl border border-slate-200/80 w-full max-w-md overflow-hidden">

        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2.5">
            <div className="flex items-center gap-1.5 mr-1">
              <span className="apple-dot apple-dot-red" onClick={onClose} />
              <span className="apple-dot apple-dot-yellow" />
              <span className="apple-dot apple-dot-green" />
            </div>
            <div className="w-7 h-7 rounded-lg bg-[#007AFF]/10 text-[#007AFF] flex items-center justify-center">
              <KeyRound size={16} />
            </div>
            <h3 className="font-bold text-slate-800 text-sm">修改登录密码</h3>
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 rounded-lg p-1">
            <X size={18} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {error && (
            <div className="p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
              <AlertCircle size={16} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {success && (
            <div className="p-3 bg-[#34C759]/10 border border-[#34C759]/20 rounded-xl text-xs text-[#34C759] flex items-center gap-2">
              <CheckCircle2 size={16} className="shrink-0" />
              <span>密码已成功修改！</span>
            </div>
          )}

          <div>
            <label className="block text-xs font-semibold text-slate-600 mb-1.5">
              当前原密码
            </label>
            <input
              type="password"
              required
              value={oldPassword}
              onChange={(e) => setOldPassword(e.target.value)}
              className="w-full px-3.5 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all text-slate-800"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-600 mb-1.5">
              新密码 (至少 6 位)
            </label>
            <input
              type="password"
              required
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              className="w-full px-3.5 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all text-slate-800"
            />
          </div>

          <div>
            <label className="block text-xs font-semibold text-slate-600 mb-1.5">
              确认新密码
            </label>
            <input
              type="password"
              required
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              className="w-full px-3.5 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all text-slate-800"
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
              disabled={loading || success}
              className="flex items-center gap-1.5 px-5 py-2 text-sm font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 rounded-xl shadow-xs transition-all"
            >
              {loading && <Loader2 size={16} className="animate-spin" />}
              确认修改
            </button>
          </div>
        </form>

      </div>
    </div>
  );
};
