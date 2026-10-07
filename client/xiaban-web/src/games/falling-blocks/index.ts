import { packedResultGame } from '../common/packedResult.ts'

export const fallingBlocks = packedResultGame({
  id: 'falling-blocks',
  name: '俄罗斯方块',
  stats: r => [[String(r.lines), '消除行数']],
  cheer: r => r.lines >= 20 ? ['laughing', `一口气消了 ${r.lines} 行，好解压`] : ['excited', '这一局，玩得漂亮'],
})
