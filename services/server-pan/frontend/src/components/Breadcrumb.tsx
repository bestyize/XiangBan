import React from 'react';
import { ChevronRight, Home, Folder } from 'lucide-react';
import { BreadcrumbItem } from '../types';

interface BreadcrumbProps {
  items: BreadcrumbItem[];
  onNavigate: (folderId: string | null) => void;
}

export const Breadcrumb: React.FC<BreadcrumbProps> = ({ items, onNavigate }) => {
  return (
    <nav className="flex items-center gap-1 text-sm font-medium overflow-x-auto py-1 scrollbar-none">
      {items.map((item, index) => {
        const isLast = index === items.length - 1;
        const isRoot = index === 0;

        return (
          <React.Fragment key={item.id ?? 'root'}>
            {index > 0 && (
              <ChevronRight size={14} className="text-slate-400 shrink-0 mx-0.5" />
            )}
            <button
              onClick={() => onNavigate(item.id)}
              disabled={isLast}
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg transition-all shrink-0 ${
                isLast
                  ? 'text-slate-900 font-bold bg-white/70 shadow-2xs border border-slate-200/50 cursor-default'
                  : 'text-slate-600 hover:text-[#007AFF] hover:bg-white/50 cursor-pointer'
              }`}
            >
              {isRoot ? (
                <Home size={15} className={isLast ? 'text-[#007AFF]' : 'text-slate-500'} />
              ) : (
                <Folder size={15} className={isLast ? 'text-amber-500' : 'text-slate-400'} />
              )}
              <span>{item.name}</span>
            </button>
          </React.Fragment>
        );
      })}
    </nav>
  );
};
