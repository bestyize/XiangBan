export interface DesktopProductDownload {
  version: string;
  notes: string;
  publishedAt: string;
  url: string;
  sha256?: string;
  bytes?: number;
}

export interface PublicNotice {
  id: string;
  title: string;
  body: string;
  revision: string;
}

export interface DesktopProduct {
  schemaVersion: number;
  introduction: string;
  downloads: Record<string, DesktopProductDownload>;
  announcements: PublicNotice[];
}

const WALLPAPER_ORIGIN = 'https://wallpaper.xiangban.xyz';

function resolveDownloadUrl(url: string): string {
  if (!url) return url;
  if (url.startsWith('/')) return `${WALLPAPER_ORIGIN}${url}`;
  return url;
}

export async function getDesktopProduct(): Promise<DesktopProduct> {
  const response = await fetch('/api/v1/desktop/product', { cache: 'no-store' });
  if (!response.ok) throw new Error('版本信息加载失败，请稍后重试。');
  const data: DesktopProduct = await response.json();
  if (data.downloads) {
    for (const key of Object.keys(data.downloads)) {
      const rec = data.downloads[key];
      if (rec?.url) {
        rec.url = resolveDownloadUrl(rec.url);
      }
    }
  }
  return data;
}
