import { packedResultGame } from '../common/packedResult.ts'

/** Dusk glider packs perfect landings into `lines`, longest chain into `maxTile`, islands reached into `level` and flights into `steps`. */
export const duskGlider = packedResultGame({
  id: 'dusk-glider',
  name: '暮色滑翔',
  stats: r => [[String(r.lines), '完美落地'], [String(r.maxTile), '最长连击'], [String(r.level), '抵达岛屿']],
  cheer: (r) => {
    if (r.level >= 5) return ['proud', `一路飞过 ${r.level} 座岛，天黑前你都没停过`]
    if (r.maxTile >= 5) return ['excited', `连续 ${r.maxTile} 次完美落地，顺风全被你借走了`]
    if (r.level >= 2) return ['happy', `飞过 ${r.level} 座岛了，再滑一次看看更远的风景`]
    return ['happy', '天黑得有点快，再飞一次，前面还有岛']
  },
  mode: r => r.level >= 2 ? `飞过 ${r.level} 座岛` : '第一座岛',
  shareLead: r => r.level >= 2 ? `飞过 ${r.level} 座岛，` : '',
})
