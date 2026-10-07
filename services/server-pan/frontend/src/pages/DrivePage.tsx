import React, { useEffect, useState, useCallback } from 'react';
import { Navbar } from '../components/Navbar';
import { Breadcrumb } from '../components/Breadcrumb';
import { FileTable } from '../components/FileTable';
import { FileGrid } from '../components/FileGrid';
import { UploadModal } from '../components/UploadModal';
import { CreateFolderModal } from '../components/CreateFolderModal';
import { RenameModal } from '../components/RenameModal';
import { ShareModal } from '../components/ShareModal';
import { MySharesModal } from '../components/MySharesModal';
import { PreviewModal } from '../components/PreviewModal';
import { DeleteConfirmModal } from '../components/DeleteConfirmModal';
import { ChangePasswordModal } from '../components/ChangePasswordModal';
import { fileApi } from '../api';
import { BreadcrumbItem, FileItem, StorageStats } from '../types';
import {
  UploadCloud,
  FolderPlus,
  LayoutGrid,
  List,
  RefreshCw,
  Loader2,
  Search,
  X,
} from 'lucide-react';

export const DrivePage: React.FC = () => {
  // 当前路径与文件数据
  const [currentFolderId, setCurrentFolderId] = useState<string | null>(null);
  const [items, setItems] = useState<FileItem[]>([]);
  const [breadcrumbs, setBreadcrumbs] = useState<BreadcrumbItem[]>([]);
  const [stats, setStats] = useState<StorageStats | null>(null);
  const [loading, setLoading] = useState(true);

  // 搜索态
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<FileItem[] | null>(null);
  const [searching, setSearching] = useState(false);

  // 视图模式：'table' | 'grid'
  const [viewMode, setViewMode] = useState<'table' | 'grid'>('table');

  // 全局拖拽文件提示遮罩
  const [isWindowDragOver, setIsWindowDragOver] = useState(false);

  // 弹窗状态
  const [uploadOpen, setUploadOpen] = useState(false);
  const [createFolderOpen, setCreateFolderOpen] = useState(false);
  const [mySharesOpen, setMySharesOpen] = useState(false);
  const [changePasswordOpen, setChangePasswordOpen] = useState(false);

  const [renameTarget, setRenameTarget] = useState<FileItem | null>(null);
  const [shareTarget, setShareTarget] = useState<FileItem | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<FileItem | null>(null);
  const [previewTarget, setPreviewTarget] = useState<FileItem | null>(null);

  // 加载存储空间统计
  const loadStats = useCallback(async () => {
    try {
      const s = await fileApi.getStats();
      setStats(s);
    } catch {
      // ignore
    }
  }, []);

  // 加载文件列表
  const loadFiles = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fileApi.list(currentFolderId);
      setItems(res.items);
      setBreadcrumbs(res.breadcrumbs);
    } catch (err: any) {
      alert(err.message || '获取文件列表失败');
    } finally {
      setLoading(false);
    }
  }, [currentFolderId]);

  useEffect(() => {
    loadFiles();
    loadStats();
  }, [loadFiles, loadStats]);

  // 搜索处理（防抖处理或实时搜索）
  useEffect(() => {
    if (!searchQuery.trim()) {
      setSearchResults(null);
      setSearching(false);
      return;
    }

    setSearching(true);
    const timer = setTimeout(async () => {
      try {
        const results = await fileApi.search(searchQuery.trim());
        setSearchResults(results);
      } catch {
        setSearchResults([]);
      } finally {
        setSearching(false);
      }
    }, 250);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  // 全局拖拽事件处理
  useEffect(() => {
    const handleDragEnter = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer?.types.includes('Files')) {
        setIsWindowDragOver(true);
      }
    };
    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
    };
    const handleDragLeave = (e: DragEvent) => {
      if (e.clientX === 0 || e.clientY === 0) {
        setIsWindowDragOver(false);
      }
    };
    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      setIsWindowDragOver(false);
      if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
        setUploadOpen(true);
      }
    };

    window.addEventListener('dragenter', handleDragEnter);
    window.addEventListener('dragover', handleDragOver);
    window.addEventListener('dragleave', handleDragLeave);
    window.addEventListener('drop', handleDrop);

    return () => {
      window.removeEventListener('dragenter', handleDragEnter);
      window.removeEventListener('dragover', handleDragOver);
      window.removeEventListener('dragleave', handleDragLeave);
      window.removeEventListener('drop', handleDrop);
    };
  }, []);

  // 操作处理函数
  const handleOpenFolder = (id: string) => {
    setSearchQuery('');
    setSearchResults(null);
    setCurrentFolderId(id);
  };

  const handleNavigateBreadcrumb = (id: string | null) => {
    setSearchQuery('');
    setSearchResults(null);
    setCurrentFolderId(id);
  };

  const handleDownload = (item: FileItem) => {
    const url = fileApi.getDownloadUrl(item.id);
    const a = document.createElement('a');
    a.href = url;
    a.download = item.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  };

  const displayedItems = searchResults !== null ? searchResults : items;

  return (
    <div className="min-h-screen bg-[#F5F5F7] flex flex-col selection:bg-[#007AFF]/20 selection:text-[#007AFF]">
      {/* 顶部导航 */}
      <Navbar
        stats={stats}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        onOpenMyShares={() => setMySharesOpen(true)}
        onOpenChangePassword={() => setChangePasswordOpen(true)}
      />

      {/* 主界面工作区 */}
      <main className="flex-1 max-w-7xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-5 flex flex-col">

        {/* 顶部操作工具栏与面包屑 */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-2">
          {/* 面包屑导航（搜索时显示搜索提示） */}
          <div className="flex items-center min-w-0">
            {searchResults !== null ? (
              <div className="flex items-center gap-2 text-sm text-slate-700 bg-white px-3 py-1.5 rounded-xl border border-slate-200 shadow-2xs">
                {searching ? <Loader2 size={14} className="animate-spin text-[#007AFF]" /> : <Search size={14} className="text-[#007AFF]" />}
                <span>搜索结果：<strong>"{searchQuery}"</strong> ({searchResults.length} 个项目)</span>
                <button
                  onClick={() => setSearchQuery('')}
                  className="text-slate-400 hover:text-slate-600 ml-1 p-0.5 rounded"
                >
                  <X size={14} />
                </button>
              </div>
            ) : (
              <Breadcrumb
                items={breadcrumbs}
                onNavigate={handleNavigateBreadcrumb}
              />
            )}
          </div>

          {/* 快捷按钮组 */}
          <div className="flex items-center gap-2 shrink-0 self-end sm:self-auto">
            {/* 新建文件夹 */}
            <button
              onClick={() => setCreateFolderOpen(true)}
              className="flex items-center gap-1.5 px-3.5 py-2 text-xs font-semibold text-amber-700 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/20 rounded-xl transition-all shadow-2xs cursor-pointer"
            >
              <FolderPlus size={15} />
              新建文件夹
            </button>

            {/* 上传文件（苹果系统蓝高光） */}
            <button
              onClick={() => setUploadOpen(true)}
              className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold text-white bg-[#007AFF] hover:bg-[#0071E3] rounded-xl shadow-xs shadow-[#007AFF]/25 transition-all cursor-pointer"
            >
              <UploadCloud size={15} />
              上传文件
            </button>

            {/* 刷新 */}
            <button
              onClick={() => {
                loadFiles();
                loadStats();
              }}
              title="刷新列表"
              className="p-2 text-slate-500 hover:text-slate-800 bg-white hover:bg-slate-50 border border-slate-200/80 rounded-xl transition-all shadow-2xs cursor-pointer"
            >
              <RefreshCw size={15} className={loading ? 'animate-spin' : ''} />
            </button>

            {/* 视图模式切换 */}
            <div className="flex items-center bg-white border border-slate-200/80 rounded-xl p-0.5 shadow-2xs">
              <button
                onClick={() => setViewMode('table')}
                title="列表视图"
                className={`p-1.5 rounded-lg transition-all ${
                  viewMode === 'table'
                    ? 'bg-[#007AFF] text-white shadow-2xs'
                    : 'text-slate-400 hover:text-slate-700'
                }`}
              >
                <List size={15} />
              </button>
              <button
                onClick={() => setViewMode('grid')}
                title="网格图标视图"
                className={`p-1.5 rounded-lg transition-all ${
                  viewMode === 'grid'
                    ? 'bg-[#007AFF] text-white shadow-2xs'
                    : 'text-slate-400 hover:text-slate-700'
                }`}
              >
                <LayoutGrid size={15} />
              </button>
            </div>
          </div>
        </div>

        {/* 文件列表区域 */}
        <div className="flex-1">
          {loading && !items.length ? (
            <div className="py-24 text-center flex flex-col items-center justify-center text-slate-400">
              <Loader2 size={28} className="animate-spin text-[#007AFF] mb-3" />
              <p className="text-sm font-medium">正在读取文件目录...</p>
            </div>
          ) : viewMode === 'table' ? (
            <FileTable
              items={displayedItems}
              onOpenFolder={handleOpenFolder}
              onPreview={(item) => setPreviewTarget(item)}
              onDownload={handleDownload}
              onShare={(item) => setShareTarget(item)}
              onRename={(item) => setRenameTarget(item)}
              onDelete={(item) => setDeleteTarget(item)}
            />
          ) : (
            <FileGrid
              items={displayedItems}
              onOpenFolder={handleOpenFolder}
              onPreview={(item) => setPreviewTarget(item)}
              onDownload={handleDownload}
              onShare={(item) => setShareTarget(item)}
              onRename={(item) => setRenameTarget(item)}
              onDelete={(item) => setDeleteTarget(item)}
            />
          )}
        </div>

      </main>

      {/* 全屏拖拽提示遮罩 */}
      {isWindowDragOver && (
        <div className="fixed inset-0 z-50 bg-[#007AFF]/10 backdrop-blur-xs border-4 border-dashed border-[#007AFF] flex flex-col items-center justify-center pointer-events-none animate-in fade-in duration-100">
          <div className="bg-white p-8 rounded-3xl shadow-2xl flex flex-col items-center text-center max-w-sm">
            <div className="w-16 h-16 rounded-2xl bg-[#007AFF]/10 text-[#007AFF] flex items-center justify-center mb-3">
              <UploadCloud size={32} />
            </div>
            <h3 className="text-lg font-bold text-slate-800">松开鼠标即可上传</h3>
            <p className="text-xs text-slate-400 mt-1">文件将被自动上传至当前打开的目录</p>
          </div>
        </div>
      )}

      {/* 弹窗集合 */}
      <UploadModal
        isOpen={uploadOpen}
        onClose={() => setUploadOpen(false)}
        folderId={currentFolderId}
        onUploadSuccess={() => {
          loadFiles();
          loadStats();
        }}
      />

      <CreateFolderModal
        isOpen={createFolderOpen}
        onClose={() => setCreateFolderOpen(false)}
        folderId={currentFolderId}
        onSuccess={() => {
          loadFiles();
          loadStats();
        }}
      />

      <RenameModal
        isOpen={!!renameTarget}
        item={renameTarget}
        onClose={() => setRenameTarget(null)}
        onSuccess={() => {
          loadFiles();
        }}
      />

      <ShareModal
        isOpen={!!shareTarget}
        item={shareTarget}
        onClose={() => setShareTarget(null)}
        onShareCreated={() => {
          loadStats();
        }}
      />

      <MySharesModal
        isOpen={mySharesOpen}
        onClose={() => setMySharesOpen(false)}
        onSharesChanged={() => {
          loadStats();
        }}
      />

      <PreviewModal
        isOpen={!!previewTarget}
        item={previewTarget}
        onClose={() => setPreviewTarget(null)}
        onDownload={handleDownload}
      />

      <DeleteConfirmModal
        isOpen={!!deleteTarget}
        item={deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onSuccess={() => {
          loadFiles();
          loadStats();
        }}
      />

      <ChangePasswordModal
        isOpen={changePasswordOpen}
        onClose={() => setChangePasswordOpen(false)}
      />
    </div>
  );
};
