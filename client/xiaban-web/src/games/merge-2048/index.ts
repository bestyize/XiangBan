import { packedResultGame } from '../common/packedResult.ts'

export const merge2048 = packedResultGame({
  id: 'merge-2048',
  name: '2048',
  stats: r => [[String(r.maxTile), '最大方块']],
  cheer: r => r.maxTile >= 2048 ? ['laughing', '合出 2048 了！我要给你鼓掌'] : ['excited', `合到了 ${r.maxTile}，下次冲 2048`],
})
