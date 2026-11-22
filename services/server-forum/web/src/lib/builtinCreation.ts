/** Browser-playable built-ins in mobile display order. Assets sync before dev and build. */
export const builtinAsset = (path: string) => `${import.meta.env?.DEV ? '' : 'https://res.xiangban.xyz'}/builtin-games/${path}`
export const BUILTIN_GAMES = [
  { id:'block-puzzle', name:'方块爆破', genre:'益智', description:'拖动方块填满整行或整列，消除得分。', orientation:'portrait', icon:'icon.webp' },
  { id:'dusk-glider', name:'暮色滑翔', genre:'冒险', description:'横屏 3D 滑翔，在日落之前飞过一座座岛。', orientation:'landscape', icon:'cover.svg' },
  { id:'ink-blade', name:'水墨剑客', genre:'冒险', description:'横屏水墨动作，出剑弹反，闯过竹林、雪江和烽火边关。', orientation:'landscape', icon:'cover.svg' },
  { id:'pro-cube', name:'3D全阶魔方', genre:'益智', description:'3 至 9 阶魔方，支持手势操作与智能还原。', orientation:'landscape', icon:'icon.webp' },
  { id:'snake-arena', name:'小伴贪吃蛇', genre:'解压', description:'无尽成长或三分钟冲分，吃糖果长成大蛇。', orientation:'landscape', icon:'cover.svg' },
  { id:'jade-gomoku', name:'雅韵五子棋', genre:'益智', description:'水墨玉石质感五子棋，支持人机对弈与双人对局。', orientation:'landscape', icon:'icon.webp' },
  { id:'merge-2048', name:'2048', genre:'益智', description:'滑动合并相同数字，慢慢合出 2048。', orientation:'portrait', icon:'icon.webp' },
  { id:'water-sort', name:'倒杯快乐', genre:'解压', description:'36 关调饮解谜，解封签、接订单，每日特调。', orientation:'portrait', icon:'icon.webp' },
  { id:'mbti', name:'MBTI 性格测评', genre:'测评', description:'48 道原创情境题，观察日常偏好。', orientation:'portrait', icon:'icon.webp' },
].map(game => ({ ...game, icon:builtinAsset(`${game.id}/${game.icon}`) }))

export const MOBILE_ONLY_APPS = {
  remoteControl: { id:'remote-control', name:'遥控器', icon:builtinAsset('remote-control/icon.webp') },
  quickLaunch: { id:'quick-launch', name:'轻启', icon:builtinAsset('quick-launch/icon.webp') },
} as const

export const WANCISOU_ICON = builtinAsset('wancisou/icon.webp')
export const WANCISOU_WEB_ENTRY = builtinAsset('wancisou/index.html')
export const ANDROID_DOWNLOAD_URL = 'https://github.com/bestyize/XiangBan/releases/latest'

/** The bundled search tool can launch only while this published work version allows it. */
export function canOpenBundledWancisou(detail: {
  kind: string
  slug: string
  status: string
  network_disabled?: boolean
  version: number
  versions: { version: number; manifest: Record<string, unknown> }[]
}, bundledVersion: unknown): boolean {
  const manifest = detail.versions.find(item => item.version === detail.version)?.manifest
  return detail.kind === 'app' && detail.slug === 'wancisou' && detail.status === 'published' &&
    detail.network_disabled === false && manifest?.id === 'wancisou' &&
    typeof bundledVersion === 'string' && manifest.version === bundledVersion
}

export type BuiltinGame = typeof BUILTIN_GAMES[number]
export function builtinGame(id:string): BuiltinGame | undefined { return BUILTIN_GAMES.find(game => game.id === id) }
/** Only the current opaque-origin game frame may acknowledge the host nonce. */
export function isBuiltinGameMessage(data: unknown, origin: string, source: unknown, frame: unknown, nonce: string): boolean {
  if (!data || typeof data !== 'object' || !frame || source !== frame || origin !== 'null' || !nonce) return false
  const message = data as Record<string, unknown>
  return message.channel === 'xiaban.game-pack' && message.protocol === 1 && message.nonce === nonce
}
