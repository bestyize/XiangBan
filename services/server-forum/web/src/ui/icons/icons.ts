/**
 * 社区前台通用 UI 图标路径表（24×24 viewBox，线宽 1.8，圆头圆角，与手机端 :common:ui XIcons 对齐）。
 * 品牌 Logo 与业务插画留在业务层，不放进此表。
 */
export const UI_STROKE_ICONS = {
  // 导航与方向
  'chevron-left': ['M15 5l-7 7 7 7'],
  'chevron-right': ['M9 5l7 7-7 7'],
  'chevron-down': ['M6 9l6 6 6-6'],
  'chevron-up': ['M6 15l6-6 6 6'],
  'arrow-right': ['M5 12h13M13 6l6 6-6 6'],
  'arrow-up': ['M12 19V5M6 11l6-6 6 6'],
  compass: ['M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0Z', 'm15.5 8.5-2.2 4.8-4.8 2.2 2.2-4.8 4.8-2.2Z'],
  grid: ['M4 3h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z', 'M15 3h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z', 'M4 14h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z', 'M15 14h5a1 1 0 0 1 1 1v5a1 1 0 0 1-1 1h-5a1 1 0 0 1-1-1v-5a1 1 0 0 1 1-1Z'],
  search: ['M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0Z', 'm15 15 6 6'],
  sort: ['M4 7h16', 'M7 12h10', 'M10 17h4'],
  more: ['M5 12h.01M12 12h.01M19 12h.01'],
  'more-vertical': ['M12 5h.01M12 12h.01M12 19h.01'],
  external: ['M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6', 'M15 3h6v6M10 14 21 3'],

  // 操作与编辑
  close: ['m6 6 12 12M18 6 6 18'],
  plus: ['M12 5v14M5 12h14'],
  check: ['M20 6 9 17l-5-5'],
  edit: ['M12 20h9', 'M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z'],
  create: ['m15 4 5 5-10 10-6 1 1-6Z', 'm12 7 5 5', 'M5 3v4M3 5h4'],
  trash: ['M3 6h18', 'M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6', 'M10 11v6M14 11v6', 'M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2'],
  copy: ['M11 9h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2Z', 'M5 15V6a2 2 0 0 1 2-2h9'],
  share: ['M12 15V3m-4 4 4-4 4 4', 'M5 11v9h14v-9'],
  'share-up': ['M4 12v7a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-7', 'm16 6-4-4-4 4', 'M12 2v13'],
  link: ['M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71', 'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71'],
  download: ['M12 3v12m-5-5 5 5 5-5', 'M4 16v5h16v-5'],
  upload: ['M12 16V4m-5 5 5-5 5 5', 'M4 16v5h16v-5'],
  refresh: ['M20 7v5h-5', 'M20 12a8 8 0 1 0-2.4 5.7'],
  expand: ['M4 9V4h5m6 0h5v5m0 6v5h-5m-6 0H4v-5'],
  collapse: ['M4 9h5V4m6 0v5h5m0 6h-5v5m-6 0v-5H4'],
  sliders: ['M4 7h16M4 17h16', 'M8 4v6M16 14v6'],
  settings: ['M9 12a3 3 0 1 0 6 0 3 3 0 1 0-6 0', 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l-.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z'],

  // 状态与反馈
  info: ['M3.5 12a8.5 8.5 0 1 0 17 0 8.5 8.5 0 1 0-17 0', 'M12 11v5M12 7.8v.4'],
  warning: ['M12 4l9 16H3Z', 'M12 10v4M12 17h.01'],
  help: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'M9.5 9.2a2.6 2.6 0 0 1 5 1c0 1.6-2.5 2-2.5 3.3', 'M12 17h.01'],
  block: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'M5.6 5.6 18.4 18.4'],
  shield: ['m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6Z', 'm8 12 3 3 5-6'],
  flag: ['M5 21V4M5 4h11l-2 4 2 4H5'],
  eye: ['M2 12s3.8-6.5 10-6.5S22 12 22 12s-3.8 6.5-10 6.5S2 12 2 12Z', 'M9 12a3 3 0 1 0 6 0 3 3 0 1 0-6 0'],
  bell: ['M6 17v-6a6 6 0 0 1 12 0v6l2 2H4Z', 'M10 22h4'],
  sun: ['M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z', 'M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5 19 19M5 19l1.5-1.5M17.5 6.5 19 5'],
  moon: ['M20.5 14.5A8.5 8.5 0 0 1 9.5 3.5a9 9 0 1 0 11 11Z'],

  // 社区与互动
  spark: ['m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5Z'],
  chat: ['M4 4h16v13h-8l-5 4v-4H4Z'],
  community: ['M5 4h14a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-7l-6 4v-4H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z', 'M7 9h10M7 13h6'],
  message: ['M20.8 12a8.8 8.8 0 0 1-8.8 8.8H3.4l1.7-3.4A8.8 8.8 0 1 1 20.8 12Z'],
  heart: ['M12 20L5.9 14.3C4.2 12.7 3 11.1 3 8.9A4.9 4.9 0 0 1 7.9 4c1.6 0 3 .8 4.1 2.1C13.1 4.8 14.5 4 16.1 4A4.9 4.9 0 0 1 21 8.9c0 2.2-1.2 3.8-2.9 5.4Z'],
  star: ['M12 3.3l2.7 5.6 6.3.9-4.6 4.4 1.1 6.3-5.5-3-5.5 3 1.1-6.3L3 9.8l6.3-.9Z'],
  bookmark: ['M6 3h12v18l-6-4-6 4Z'],
  user: ['M16 8a4 4 0 1 1-8 0 4 4 0 0 1 8 0Z', 'M4 21v-2a8 8 0 0 1 16 0v2'],
  users: ['M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2', 'M5.5 7a3.5 3.5 0 1 0 7 0 3.5 3.5 0 1 0-7 0', 'M22 21v-2a4 4 0 0 0-3-3.87', 'M16 3.13a3.5 3.5 0 0 1 0 6.74'],
  smile: ['M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z', 'M8.5 14.5c1 1.3 2.2 2 3.5 2s2.5-.7 3.5-2', 'M9 9.5h.01', 'M15 9.5h.01'],
  at: ['M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8Z', 'M16 8v5a2.5 2.5 0 0 0 5 0v-1a9 9 0 1 0-4 7.5'],

  // 媒体、文件与设备
  play: ['M8 5v14l11-7Z'],
  image: ['M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z', 'M9 10a1.5 1.5 0 1 1-3 0 1.5 1.5 0 0 1 3 0Z', 'm21 16-5-5-9 9'],
  landscape: ['M5 5h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2Z', 'm3 16 5-5 4 4 3-3 6 6'],
  'image-copy': ['M11 9h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2Z', 'M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1', 'M11 17l2.5-2.5 2 2 1.5-1.5 2 2'],
  'share-card': ['M6 4h12a3 3 0 0 1 3 3v10a3 3 0 0 1-3 3H6a3 3 0 0 1-3-3V7a3 3 0 0 1 3-3Z', 'M7 9h6M7 13h10', 'M18 9.5a1.5 1.5 0 1 0-3 0 1.5 1.5 0 0 0 3 0Z'],
  folder: ['M3 6h6l2 3h10v11H3Z'],
  document: ['M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z', 'M14 2v6h6M8 13h8M8 17h5'],
  news: ['M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z', 'M7 8h10M7 12h6M7 16h8'],
  quote: ['M4 6h16M4 12h12M4 18h8'],
  qr: ['M3 3h7v7H3ZM14 3h7v7h-7ZM3 14h7v7H3Z', 'M14 14h3v3h-3ZM19 14h2v2h-2ZM14 19h2v2h-2ZM18 18h3v3h-3Z'],
  keyboard: ['M4 6h16a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2Z', 'M6 10h.01m4 0h.01m4 0h.01m4 0h.01M8 14h8'],
  monitor: ['M3 4h18v13H3Z', 'M12 17v4M8 21h8'],
  cpu: ['M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2Z', 'M10 10h4v4h-4Z', 'M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4'],
  gamepad: ['M8 7.5h8a5 5 0 0 1 5 5 5 5 0 0 1-5 5H8a5 5 0 0 1-5-5 5 5 0 0 1 5-5Z', 'M8 10.5v4M6 12.5h4', 'M15.5 11.5h.01M17.8 13.5h.01'],
} as const

export const UI_FILLED_ICONS = {
  'client-monitor': ['M4 3.5h16a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-6.5V20H17a1 1 0 0 1 0 2H7a1 1 0 0 1 0-2h3.5v-2.5H4a2 2 0 0 1-2-2v-10a2 2 0 0 1 2-2Z'],
  'client-phone': ['M8 1.5h8a2.5 2.5 0 0 1 2.5 2.5v16a2.5 2.5 0 0 1-2.5 2.5H8a2.5 2.5 0 0 1-2.5-2.5V4A2.5 2.5 0 0 1 8 1.5ZM10.75 18.2a.85.85 0 0 0 0 1.7h2.5a.85.85 0 0 0 0-1.7Z'],
  'play-solid': ['M8 5.6v12.8a1 1 0 0 0 1.52.85l10.2-6.4a1 1 0 0 0 0-1.7L9.52 4.75A1 1 0 0 0 8 5.6Z'],
} as const

export type UiStrokeIconName = keyof typeof UI_STROKE_ICONS
export type UiFilledIconName = keyof typeof UI_FILLED_ICONS
export type UiIconName = UiStrokeIconName | UiFilledIconName

export const UI_ICON_NAMES = [
  ...Object.keys(UI_STROKE_ICONS),
  ...Object.keys(UI_FILLED_ICONS),
] as UiIconName[]

/** 兼容旧 AppIcon.vue 别名的归一化映射。 */
export const LEGACY_ICON_ALIASES: Record<string, UiIconName> = {
  arrow: 'chevron-right',
  back: 'chevron-left',
  ai: 'spark',
  me: 'user',
  gear: 'settings',
  settings: 'sliders',
  'smile-round': 'smile',
  'at-round': 'at',
  restore: 'collapse',
}

export function resolveUiIconName(name: string): UiIconName {
  if (name in LEGACY_ICON_ALIASES) return LEGACY_ICON_ALIASES[name]
  if (name in UI_STROKE_ICONS || name in UI_FILLED_ICONS) return name as UiIconName
  return 'community'
}
