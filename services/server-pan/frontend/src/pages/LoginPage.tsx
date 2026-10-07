import React, { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { authApi } from '../api';
import { Cloud, Lock, User, ArrowRight, AlertCircle, Loader2, Sparkles } from 'lucide-react';

export const LoginPage: React.FC = () => {
  const { login } = useAuth();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!username.trim() || !password) return;

    setLoading(true);
    setError(null);

    try {
      const res = await authApi.login(username.trim(), password);
      login(res.token, res.user);
    } catch (err: any) {
      setError(err.message || '登录失败，请检查用户名或密码');
    } finally {
      setLoading(false);
    }
  };

  const fillDefaultCredentials = () => {
    setUsername('admin');
    setPassword('123456');
    setError(null);
  };

  return (
    <div className="min-h-screen bg-[#F5F5F7] flex flex-col items-center justify-center p-4 relative selection:bg-[#007AFF]/20 selection:text-[#007AFF]">
      {/* 背景苹果质感微光 */}
      <div className="absolute top-1/4 -left-20 w-96 h-96 bg-[#007AFF]/5 rounded-full blur-3xl pointer-events-none" />
      <div className="absolute bottom-1/4 -right-20 w-96 h-96 bg-[#34C759]/5 rounded-full blur-3xl pointer-events-none" />

      {/* 居中主卡片 */}
      <div className="w-full max-w-md bg-white rounded-3xl shadow-xl shadow-slate-200/60 border border-slate-200/80 overflow-hidden relative z-10 transition-all">

        {/* 顶部 macOS 交通灯圆点 */}
        <div className="px-6 py-4 border-b border-slate-100 flex items-center justify-between bg-slate-50/50">
          <div className="flex items-center gap-2">
            <span className="apple-dot apple-dot-red" />
            <span className="apple-dot apple-dot-yellow" />
            <span className="apple-dot apple-dot-green" />
          </div>
          <span className="text-[11px] font-semibold uppercase tracking-wider text-slate-400">
            PanDown Cloud
          </span>
        </div>

        <div className="p-8">
          {/* Logo 与主标题 */}
          <div className="text-center mb-7">
            <div className="w-14 h-14 rounded-2xl bg-[#007AFF] text-white flex items-center justify-center mx-auto mb-3 shadow-md shadow-[#007AFF]/30">
              <Cloud size={30} className="fill-white/20" />
            </div>
            <h1 className="text-2xl font-bold tracking-tight text-[#1D1D1F]">
              登录 PanDown
            </h1>
            <p className="text-xs text-slate-400 mt-1">
              简单好用 · 极速流畅的个人私有云盘
            </p>
          </div>

          {/* 默认管理员账号提示卡片 */}
          <div className="mb-6 p-3.5 bg-[#007AFF]/5 rounded-2xl border border-[#007AFF]/15 flex items-center justify-between text-xs">
            <div className="space-y-0.5">
              <span className="text-[11px] font-semibold text-[#007AFF] block">
                默认管理员账号
              </span>
              <span className="text-slate-600 font-mono text-[11px]">
                账号: <strong>admin</strong> | 密码: <strong>123456</strong>
              </span>
            </div>
            <button
              type="button"
              onClick={fillDefaultCredentials}
              className="flex items-center gap-1 px-2.5 py-1.5 bg-[#007AFF] hover:bg-[#0071E3] text-white text-[11px] font-bold rounded-xl transition-all shadow-xs"
            >
              <Sparkles size={12} />
              一键填入
            </button>
          </div>

          {/* 错误提示 */}
          {error && (
            <div className="mb-4 p-3 bg-[#FF3B30]/10 border border-[#FF3B30]/20 rounded-xl text-xs text-[#FF3B30] flex items-center gap-2">
              <AlertCircle size={16} className="shrink-0" />
              <span>{error}</span>
            </div>
          )}

          {/* 登录表单 */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-slate-600 mb-1.5">
                账号名称
              </label>
              <div className="relative">
                <User size={16} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400" />
                <input
                  type="text"
                  required
                  autoFocus
                  placeholder="请输入用户名 (如 admin)"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  className="w-full pl-10 pr-4 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all text-slate-800"
                />
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-slate-600 mb-1.5">
                账号密码
              </label>
              <div className="relative">
                <Lock size={16} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400" />
                <input
                  type="password"
                  required
                  placeholder="请输入登录密码"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  className="w-full pl-10 pr-4 py-2.5 bg-[#F5F5F7] border border-slate-200 focus:border-[#007AFF] focus:bg-white rounded-xl text-sm outline-none transition-all text-slate-800"
                />
              </div>
            </div>

            <button
              type="submit"
              disabled={loading || !username.trim() || !password}
              className="w-full mt-2 flex items-center justify-center gap-2 py-3 bg-[#007AFF] hover:bg-[#0071E3] disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-semibold rounded-xl shadow-md shadow-[#007AFF]/25 transition-all"
            >
              {loading ? (
                <>
                  <Loader2 size={16} className="animate-spin" />
                  验证中...
                </>
              ) : (
                <>
                  进入网盘
                  <ArrowRight size={16} />
                </>
              )}
            </button>
          </form>

        </div>

      </div>

      <div className="mt-6 text-xs text-slate-400 font-medium">
        Powered by Rust (Axum) + Apple Design Aesthetics
      </div>
    </div>
  );
};
