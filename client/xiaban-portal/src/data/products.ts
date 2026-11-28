export type ProductCategory = 'all' | 'client' | 'web' | 'h5';

export interface ProductShot {
  label: string;
  src: string;
}

export interface ProductItem {
  id: string;
  name: string;
  domainLabel: string;
  badge: string;
  category: Exclude<ProductCategory, 'all'>;
  description: string;
  url: string;
  previewKind: 'dual-phone' | 'full' | 'full-center' | 'game-split';
  previewImages: {
    primary: string;
    primaryAlt: string;
    secondary?: string;
    secondaryAlt?: string;
    tertiary?: string;
    tertiaryAlt?: string;
  };
  lightboxTitle: string;
  shots: ProductShot[];
}

export interface DownloadOption {
  title: string;
  subtitle: string;
  url: string;
}

export const DEFAULT_ASSET_BASE = 'https://res.xiangban.xyz/img/xiaban-portal';

function resolveAssetBase(): string {
  const env = (import.meta as { env?: Record<string, string | undefined> }).env;
  const raw = env?.VITE_ASSET_BASE?.trim();
  return (raw && raw.length > 0 ? raw : DEFAULT_ASSET_BASE).replace(/\/+$/, '');
}

export const ASSET_BASE = resolveAssetBase();

export function assetUrl(filename: string): string {
  return `${ASSET_BASE}/${filename.replace(/^\/+/, '')}`;
}

export const BRAND_ICON_URL = assetUrl('app-icon-new.webp');

export const CONTACT_URL = 'https://chat.xiangban.xyz/contact';
export const COMMUNITY_URL = 'https://chat.xiangban.xyz/';
export const WALLPAPER_URL = 'https://wallpaper.xiangban.xyz/';
export const DOWNLOAD_URL = 'https://xiaban.xyz/download';
export const MOBILE_URL = 'https://xiaban.xyz/app';
export const DESKTOP_URL = 'https://xiaban.xyz/desktop';

export const FILTERS: Array<{ key: ProductCategory; label: string; count: number }> = [
  { key: 'all', label: '全部', count: 6 },
  { key: 'client', label: '客户端', count: 2 },
  { key: 'web', label: '社区/壁纸', count: 2 },
  { key: 'h5', label: '轻玩 H5', count: 2 },
];

export const PRODUCTS: ProductItem[] = [
  {
    id: 'mobile',
    name: '小伴（手机 App）',
    domainLabel: 'xiaban.xyz/app',
    badge: 'Android · iOS 开发中',
    category: 'client',
    description: '随身小伴 AI、一句话创造工具与游戏、内置游乐场、轻启跳过开屏广告与原生社区。',
    url: MOBILE_URL,
    previewKind: 'dual-phone',
    previewImages: {
      primary: assetUrl('mobile-ai-home.webp'),
      primaryAlt: '小伴手机 App 主页实拍',
      secondary: assetUrl('mobile-chat-build.webp'),
      secondaryAlt: '小伴手机 App 创造工具与游戏',
    },
    lightboxTitle: '小伴（手机 App）实机界面',
    shots: [
      { label: '小伴 AI 首页', src: assetUrl('mobile-ai-home.webp') },
      { label: '创造工具/游戏', src: assetUrl('mobile-chat-build.webp') },
      { label: '原生社区', src: assetUrl('mobile-community-new.webp') },
    ],
  },
  {
    id: 'desktop',
    name: '小伴桌面',
    domainLabel: 'xiaban.xyz/desktop',
    badge: 'Win · macOS',
    category: 'client',
    description: '桌宠、动态壁纸、14 种特效、超级小伴悬浮助手与本地通用 Agent 创作。',
    url: DESKTOP_URL,
    previewKind: 'full',
    previewImages: {
      primary: assetUrl('desktop-dragon.webp'),
      primaryAlt: '小伴桌面 v3.1.8 实机界面',
    },
    lightboxTitle: '小伴桌面 v3.1.8 实机界面',
    shots: [
      { label: '3D 龙鱼 & 特效', src: assetUrl('desktop-dragon.webp') },
      { label: '小伴 AI 新建任务', src: assetUrl('desktop-home.webp') },
      { label: 'AI 插件与专家', src: assetUrl('desktop-plugins-v318.webp') },
    ],
  },
  {
    id: 'community',
    name: '小伴社区',
    domainLabel: 'chat.xiangban.xyz',
    badge: 'Web · 移动网页版',
    category: 'web',
    description: '创意作品与微观展厅、小伴前沿、AI 日报、开源精选、模型榜与网页版小伴 AI。',
    url: COMMUNITY_URL,
    previewKind: 'full',
    previewImages: {
      primary: assetUrl('site-community.webp'),
      primaryAlt: '小伴社区网页版实拍（chat.xiangban.xyz）',
    },
    lightboxTitle: '小伴社区实拍（chat.xiangban.xyz）',
    shots: [
      { label: 'PC 网页版首页', src: assetUrl('site-community.webp') },
      { label: '移动端视图', src: assetUrl('mobile-community-new.webp') },
    ],
  },
  {
    id: 'game',
    name: '小伴游戏厅',
    domainLabel: 'game.xiaban.xyz',
    badge: 'PWA · 8 款零广告',
    category: 'h5',
    description: '免登录即玩的 8 款官方精品小游戏（水墨剑客、倒杯快乐、暮色滑翔等），支持离线。',
    url: 'https://game.xiaban.xyz/',
    previewKind: 'game-split',
    previewImages: {
      primary: assetUrl('site-game.webp'),
      primaryAlt: '小伴游戏厅首页实拍（game.xiaban.xyz）',
      secondary: assetUrl('work-ink-blade.webp'),
      secondaryAlt: '《水墨剑客》实机画面',
      tertiary: assetUrl('work-dusk-glider.webp'),
      tertiaryAlt: '《暮色滑翔》实机画面',
    },
    lightboxTitle: '小伴游戏厅实拍（game.xiaban.xyz）',
    shots: [
      { label: '游戏厅首页', src: assetUrl('site-game.webp') },
      { label: '水墨剑客', src: assetUrl('work-ink-blade.webp') },
      { label: '暮色滑翔', src: assetUrl('work-dusk-glider.webp') },
    ],
  },
  {
    id: 'people',
    name: '人格贩卖机',
    domainLabel: 'people.xiaban.xyz',
    badge: 'H5 · 12 套免费',
    category: 'h5',
    description: '12 套潮玩盲盒风趣味与人格测评，高清海报无水印导出，零门槛 URL 双人合盘。',
    url: 'https://people.xiaban.xyz/',
    previewKind: 'full-center',
    previewImages: {
      primary: assetUrl('site-people.webp'),
      primaryAlt: '人格贩卖机线上实拍（people.xiaban.xyz）',
    },
    lightboxTitle: '人格贩卖机实拍（people.xiaban.xyz）',
    shots: [
      { label: 'VEND·ME 首页', src: assetUrl('site-people.webp') },
    ],
  },
  {
    id: 'wallpaper',
    name: '小伴壁纸站',
    domainLabel: 'wallpaper.xiangban.xyz',
    badge: 'Web · 4K 图库与视频',
    category: 'web',
    description: '深色画廊级 4K 静态壁纸与循环动态视频网站，免装即逛，登录账号跨端同步收藏。',
    url: WALLPAPER_URL,
    previewKind: 'full',
    previewImages: {
      primary: assetUrl('site-wallpaper.webp'),
      primaryAlt: '小伴壁纸站网页实拍（wallpaper.xiangban.xyz）',
    },
    lightboxTitle: '小伴壁纸站实拍（wallpaper.xiangban.xyz）',
    shots: [
      { label: '4K 壁纸网站', src: assetUrl('site-wallpaper.webp') },
    ],
  },
];

export const DOWNLOAD_OPTIONS: DownloadOption[] = [
  {
    title: '小伴桌面 · Windows',
    subtitle: 'Windows 10 / 11 (64 位)',
    url: `${DESKTOP_URL}?os=win`,
  },
  {
    title: '小伴桌面 · macOS',
    subtitle: 'macOS 12+ (Apple 芯片)',
    url: `${DESKTOP_URL}?os=mac`,
  },
  {
    title: '小伴手机 App · Android',
    subtitle: 'AI 伙伴、创造工具/游戏、游乐场与社区',
    url: MOBILE_URL,
  },
];

export const FOOTER_LINKS: Array<{ label: string; url: string }> = [
  { label: '小伴 App', url: MOBILE_URL },
  { label: '小伴桌面', url: DESKTOP_URL },
  { label: '小伴社区', url: COMMUNITY_URL },
  { label: '小伴游戏厅', url: 'https://game.xiaban.xyz/' },
  { label: '人格贩卖机', url: 'https://people.xiaban.xyz/' },
  { label: '小伴壁纸站', url: WALLPAPER_URL },
];
