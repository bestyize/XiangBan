export type GameCategory = 'puzzle' | 'action' | 'casual' | 'board';
export type GameOrientation = 'portrait' | 'landscape' | 'any';
export type FilterKey = 'all' | 'portrait' | GameCategory;

export interface GameItem {
  id: string;
  name: string;
  genre: string;
  category: GameCategory;
  orient: GameOrientation;
  orientLabel: string;
  size: string;
  version: string;
  tagline: string;
  desc: string;
  icon: string;
  cover: string;
  featured: boolean;
  heroBadge?: string;
  controls: string;
  scoreUnit?: string;
}

export interface AccentOption {
  id: string;
  name: string;
  color: string;
  soft: string;
}

export const ACCENTS: AccentOption[] = [
  { id: 'red', name: '小伴红', color: '#e3203c', soft: '#fff3f4' },
  { id: 'green', name: '薄荷绿', color: '#2e8b5e', soft: '#eef8f2' },
  { id: 'blue', name: '海湾蓝', color: '#2563eb', soft: '#eff6ff' },
  { id: 'purple', name: '暮紫', color: '#7c3aed', soft: '#f5f3ff' },
  { id: 'amber', name: '琥珀', color: '#d97706', soft: '#fffbeb' },
  { id: 'graphite', name: '石墨', color: '#3f3f46', soft: '#f4f4f5' }
];

export const FILTERS: { key: FilterKey; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'portrait', label: '单手竖屏' },
  { key: 'puzzle', label: '益智' },
  { key: 'action', label: '动作' },
  { key: 'casual', label: '解压' },
  { key: 'board', label: '棋牌' }
];

export const GAMES: GameItem[] = [
  {
    id: 'ink-blade',
    name: '水墨剑客',
    genre: '动作',
    category: 'action',
    orient: 'landscape',
    orientLabel: '横屏',
    size: '1.0 MB',
    version: '1.9.0',
    tagline: '看准朱光弹反 · 七卷四十二幅',
    desc: '横屏水墨动作：竹林、雪江、烽火、怒涛、灯市、悬山、归墨七卷各六幅。看准朱光出剑弹反，在终章归墨卷接住回锋、与无名剑客对剑破心，逐幅解锁、集三印、求逸品。',
    icon: './game-icons/ink-blade.png',
    cover: './game-icons/ink-blade-cover.svg',
    featured: true,
    heroBadge: '动作主打 · 横屏沉浸',
    controls: '双手触控 · 弹反出剑',
    scoreUnit: '分'
  },
  {
    id: 'water-sort',
    name: '倒杯快乐',
    genre: '解压',
    category: 'casual',
    orient: 'any',
    orientLabel: '竖横皆可',
    size: '228 KB',
    version: '2.1.0',
    tagline: '45 关调饮解谜 · 每日特调',
    desc: '3 个入门练习、45 关调饮解谜和每日特调。空杯中转、解封签、接订单，单手即可随时续玩。',
    icon: './game-icons/water-sort.webp',
    cover: './game-icons/water-sort-cover.svg',
    featured: true,
    heroBadge: '单手解压 · 随时续玩',
    controls: '单手点按 · 随时存档',
    scoreUnit: '分'
  },
  {
    id: 'dusk-glider',
    name: '暮色滑翔',
    genre: '解压',
    category: 'casual',
    orient: 'landscape',
    orientLabel: '横屏',
    size: '808 KB',
    version: '1.0.0',
    tagline: '3D 日落群岛 · 按住俯冲起飞',
    desc: '横屏 3D 滑翔：按住俯冲蓄力，松开乘风起飞，在日落之前飞过一座座微光群岛。',
    icon: './game-icons/dusk-glider.png',
    cover: './game-icons/dusk-glider-cover.svg',
    featured: true,
    heroBadge: '3D 治愈 · 日落滑翔',
    controls: '单指长按 · 俯冲爬升',
    scoreUnit: 'm'
  },
  {
    id: 'block-puzzle',
    name: '方块爆破',
    genre: '益智',
    category: 'puzzle',
    orient: 'portrait',
    orientLabel: '竖屏',
    size: '656 KB',
    version: '2.1.2',
    tagline: '经典 8×8 消除 · 连消爆分',
    desc: '经典 8×8 消除：拖放三块填满横行或竖列，连消拿高分。支持残局自动存档与音效反馈。',
    icon: './game-icons/block-puzzle.webp',
    cover: './game-icons/block-puzzle-cover.svg',
    featured: false,
    controls: '单手拖放 · 残局续玩',
    scoreUnit: '分'
  },
  {
    id: 'merge-2048',
    name: '2048',
    genre: '益智',
    category: 'puzzle',
    orient: 'portrait',
    orientLabel: '竖屏',
    size: '140 KB',
    version: '1.1.0',
    tagline: '黑曜石琥珀光 · 顺滑合成',
    desc: '黑曜石与琥珀光视觉，四向顺滑滑动合并数字，在方寸之间一步步合出 2048。',
    icon: './game-icons/merge-2048.png',
    cover: './game-icons/merge-2048-cover.svg',
    featured: false,
    controls: '单手滑动 · 即开即合',
    scoreUnit: '分'
  },
  {
    id: 'snake-arena',
    name: '小伴贪吃蛇',
    genre: '动作',
    category: 'action',
    orient: 'landscape',
    orientLabel: '横屏',
    size: '132 KB',
    version: '1.0.5',
    tagline: '无尽成长 / 三分钟冲分',
    desc: '无尽成长或三分钟限时冲分，吃糖果长成大蛇，灵巧走位反杀 AI、赚取糖果币。',
    icon: './game-icons/snake-arena.png',
    cover: './game-icons/snake-arena-cover.svg',
    featured: false,
    controls: '左手摇杆 · 右手加速',
    scoreUnit: '分'
  },
  {
    id: 'pro-cube',
    name: '3D全阶魔方',
    genre: '益智',
    category: 'puzzle',
    orient: 'landscape',
    orientLabel: '横屏',
    size: '696 KB',
    version: '1.0.0',
    tagline: '3~9 阶磁力手感 · 智能还原',
    desc: '3 至 9 阶全系真实质感磁力魔方：支持手势划动切层、自由视角、计时与智能分步还原。',
    icon: './game-icons/pro-cube.png',
    cover: './game-icons/pro-cube-cover.svg',
    featured: false,
    controls: '触控切层 · 3D 视角',
    scoreUnit: '分'
  },
  {
    id: 'jade-gomoku',
    name: '雅韵五子棋',
    genre: '棋牌',
    category: 'board',
    orient: 'landscape',
    orientLabel: '横屏',
    size: '184 KB',
    version: '1.0.0',
    tagline: '水墨玉石质感 · 人机/双人',
    desc: '水墨玉石质感五子棋，支持智能人机对弈与双人同屏对局，沉浸式金石落子音效与连珠特效。',
    icon: './game-icons/jade-gomoku.png',
    cover: './game-icons/jade-gomoku-cover.svg',
    featured: false,
    controls: '点击落子 · 同屏双人',
    scoreUnit: '胜'
  }
];

export function filterGames(games: GameItem[], filterKey: FilterKey): GameItem[] {
  if (filterKey === 'all') return games;
  if (filterKey === 'portrait') {
    return games.filter((g) => g.orient === 'portrait' || g.orient === 'any');
  }
  return games.filter((g) => g.category === filterKey);
}

export function getGameById(id: string | null | undefined): GameItem | undefined {
  if (!id) return undefined;
  return GAMES.find((g) => g.id === id);
}

export function getFeaturedGames(): GameItem[] {
  return GAMES.filter((g) => g.featured);
}

export function formatScore(game: GameItem, score: number): string {
  if (!score || score <= 0) return '';
  const num = score.toLocaleString('zh-CN');
  return game.scoreUnit ? `${num} ${game.scoreUnit}` : num;
}
