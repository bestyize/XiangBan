import { packedResultGame } from '../common/packedResult.ts'

export const waterSort = packedResultGame({
  id: 'water-sort',
  name: '奶茶大师',
  stats: r => [[String(r.level), '完成关卡'], [String(r.steps), '倒水步数']],
  cheer: r => ['happy', `调完 ${r.level} 关，一滴都没洒`],
})
