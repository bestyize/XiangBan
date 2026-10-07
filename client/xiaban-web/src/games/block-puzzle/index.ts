import { packedResultGame } from '../common/packedResult.ts'

export const blockPuzzle = packedResultGame({
  id: 'block-puzzle',
  name: '消失高手',
  stats: r => [[String(r.lines), '消除行列']],
  cheer: r => r.lines >= 20 ? ['laughing', `一口气消了 ${r.lines} 行，好解压`] : ['excited', '这一局，玩得漂亮'],
})
