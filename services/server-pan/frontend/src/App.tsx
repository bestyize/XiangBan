import React, { useEffect, useState } from 'react';
import { AuthProvider, useAuth } from './context/AuthContext';
import { LoginPage } from './pages/LoginPage';
import { DrivePage } from './pages/DrivePage';
import { PublicSharePage } from './pages/PublicSharePage';
import { Loader2 } from 'lucide-react';

const RootApp: React.FC = () => {
  const { user, loading } = useAuth();
  const [shareCode, setShareCode] = useState<string | null>(null);

  // 解析当前路由（支持 #share/:code 或 /share/:code）
  const parseRoute = () => {
    const hash = window.location.hash;
    const path = window.location.pathname;

    if (hash.startsWith('#share/')) {
      const code = hash.replace('#share/', '').split('?')[0];
      setShareCode(code || null);
    } else if (path.startsWith('/share/')) {
      const code = path.replace('/share/', '').split('?')[0];
      setShareCode(code || null);
    } else {
      setShareCode(null);
    }
  };

  useEffect(() => {
    parseRoute();
    const handleHashChange = () => parseRoute();
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  // 如果访问的是公开分享链接，则直接渲染公开提取页面（免登录）
  if (shareCode) {
    return <PublicSharePage code={shareCode} />;
  }

  // 加载用户会话中
  if (loading) {
    return (
      <div className="min-h-screen bg-[#F5F5F7] flex flex-col items-center justify-center">
        <Loader2 size={32} className="animate-spin text-[#007AFF] mb-3" />
        <p className="text-sm font-semibold text-slate-500">正在进入 PanDown...</p>
      </div>
    );
  }

  // 未登录跳转登录页，已登录进入网盘工作台
  return user ? <DrivePage /> : <LoginPage />;
};

export const App: React.FC = () => {
  return (
    <AuthProvider>
      <RootApp />
    </AuthProvider>
  );
};

export default App;
