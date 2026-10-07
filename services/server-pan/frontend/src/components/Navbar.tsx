import React, { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import {
  Cloud,
  Search,
  Share2,
  HardDrive,
  LogOut,
  KeyRound,
  ShieldCheck,
  ChevronDown,
} from 'lucide-react';
import { formatBytes } from '../api';
import { StorageStats } from '../types';

interface NavbarProps {
  stats: StorageStats | null;
  searchQuery: string;
  onSearchChange: (q: string) => void;
  onOpenMyShares: () => void;
  onOpenChangePassword: () => void;
}

export const Navbar: React.FC<NavbarProps> = ({
  stats,
  searchQuery,
  onSearchChange,
  onOpenMyShares,
  onOpenChangePassword,
}) => {
  const { user, logout } = useAuth();
  const [showUserMenu, setShowUserMenu] = useState(false);

  return (
    <header className="sticky top-0 z-30 bg-white/80 backdrop-blur-md border-b border-black/5 shadow-xs transition-all">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between gap-4">

        {/* 左侧：Mac 交通灯控制点 + Logo */}
        <div className="flex items-center gap-4 select-none">
          {/* macOS 交通灯三色圆点 */}
          <div className="flex items-center gap-2 pr-2 border-r border-slate-200">
            <span className="apple-dot apple-dot-red" title="PanDown 关闭" />
            <span className="apple-dot apple-dot-yellow" title="PanDown 最小化" />
            <span className="apple-dot apple-dot-green" title="PanDown 最大化" />
          </div>

          <div className="flex items-center gap-2.5 cursor-pointer" onClick={() => window.location.hash = ''}>
            <div className="w-9 h-9 rounded-xl bg-[#007AFF] flex items-center justify-center text-white shadow-sm shadow-[#007AFF]/30">
              <Cloud size={20} className="fill-white/30" />
            </div>
            <div>
              <span className="text-lg font-bold tracking-tight text-[#1D1D1F] flex items-center gap-1.5">
                PanDown
                <span className="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded-full bg-[#007AFF]/10 text-[#007AFF] border border-[#007AFF]/20">
                  Drive
                </span>
              </span>
            </div>
          </div>
        </div>

        {/* 中间：全局搜索框 */}
        <div className="flex-1 max-w-md mx-2">
          <div className="relative">
            <Search
              size={16}
              className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400"
            />
            <input
              type="text"
              placeholder="在当前网盘中搜索文件..."
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              className="w-full pl-10 pr-4 py-2 text-sm bg-[#F5F5F7] hover:bg-slate-100/90 focus:bg-white border border-transparent focus:border-[#007AFF]/30 rounded-xl outline-none transition-all placeholder:text-slate-400 text-slate-800 focus:ring-3 focus:ring-[#007AFF]/10"
            />
            {searchQuery && (
              <button
                onClick={() => onSearchChange('')}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-slate-400 hover:text-slate-600 bg-slate-200 rounded-full w-4 h-4 flex items-center justify-center"
              >
                ✕
              </button>
            )}
          </div>
        </div>

        {/* 右侧：存储状态、分享管理、用户态 */}
        <div className="flex items-center gap-3">
          {/* 容量指示卡 */}
          {stats && (
            <div className="hidden lg:flex items-center gap-2 px-3 py-1.5 bg-[#F5F5F7] rounded-xl border border-slate-200/60 text-xs text-slate-600">
              <HardDrive size={14} className="text-[#007AFF]" />
              <span>已用 <strong>{formatBytes(stats.used_bytes)}</strong></span>
              <span className="text-slate-300">|</span>
              <span className="text-slate-500">{stats.total_files} 文件</span>
            </div>
          )}

          {/* 我的分享按钮 */}
          <button
            onClick={onOpenMyShares}
            className="flex items-center gap-1.5 px-3 py-2 text-sm font-medium text-slate-700 hover:text-[#FF9500] hover:bg-amber-500/10 rounded-xl transition-all border border-slate-200/60 hover:border-amber-500/30"
            title="查看所有对外分享"
          >
            <Share2 size={16} className="text-[#FF9500]" />
            <span className="hidden sm:inline">我的分享</span>
            {stats && stats.total_shares > 0 && (
              <span className="ml-0.5 px-1.5 py-0.2 bg-[#FF9500] text-white text-[11px] font-bold rounded-full">
                {stats.total_shares}
              </span>
            )}
          </button>

          {/* 用户下拉菜单 */}
          <div className="relative">
            <button
              onClick={() => setShowUserMenu(!showUserMenu)}
              className="flex items-center gap-2 p-1.5 pl-2.5 rounded-xl hover:bg-[#F5F5F7] border border-transparent hover:border-slate-200/80 transition-all text-sm font-medium text-slate-800"
            >
              <div className="w-7 h-7 rounded-lg bg-gradient-to-tr from-[#007AFF] to-[#34C759] flex items-center justify-center text-white text-xs font-bold shadow-xs">
                {user?.username.charAt(0).toUpperCase() || 'A'}
              </div>
              <span className="hidden md:inline font-semibold">{user?.username}</span>
              <ChevronDown size={14} className="text-slate-400" />
            </button>

            {showUserMenu && (
              <>
                <div
                  className="fixed inset-0 z-40"
                  onClick={() => setShowUserMenu(false)}
                />
                <div className="absolute right-0 mt-2 w-56 bg-white rounded-2xl shadow-xl border border-slate-200/80 p-2 z-50 animate-in fade-in zoom-in-95 duration-100">
                  <div className="px-3 py-2 border-b border-slate-100 mb-1">
                    <p className="text-xs font-medium text-slate-400">当前账号</p>
                    <p className="text-sm font-bold text-slate-800 flex items-center gap-1.5">
                      {user?.username}
                      {user?.is_admin && (
                        <span className="inline-flex items-center gap-0.5 text-[10px] font-bold text-[#007AFF] bg-[#007AFF]/10 px-1.5 py-0.5 rounded-full">
                          <ShieldCheck size={11} />
                          管理员
                        </span>
                      )}
                    </p>
                  </div>

                  <button
                    onClick={() => {
                      setShowUserMenu(false);
                      onOpenChangePassword();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-slate-700 hover:text-slate-900 hover:bg-[#F5F5F7] rounded-xl transition-all"
                  >
                    <KeyRound size={16} className="text-[#007AFF]" />
                    修改密码
                  </button>

                  <button
                    onClick={() => {
                      setShowUserMenu(false);
                      logout();
                    }}
                    className="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-[#FF3B30] hover:bg-[#FF3B30]/10 rounded-xl transition-all"
                  >
                    <LogOut size={16} />
                    退出登录
                  </button>
                </div>
              </>
            )}
          </div>

        </div>

      </div>
    </header>
  );
};
