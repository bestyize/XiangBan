import { packedResultGame } from '../common/packedResult.ts'

/** Ink blade packs kills into `lines`, best combo into `maxTile`, parries into `steps` and a clear flag into `level`. */
export const inkBlade = packedResultGame({
  id: 'ink-blade',
  name: '水墨剑客',
  stats: r => [[String(r.lines), '击败'], [String(r.maxTile), '最高连击'], [String(r.steps), '弹反']],
  cheer: (r) => {
    if (r.level > 0) return r.steps >= 10 ? ['proud', `弹反 ${r.steps} 次，一剑破无常`] : ['laughing', '月下一剑，无常也拦不住你']
    return r.lines >= 10 ? ['excited', `斩了 ${r.lines} 个墨影，差一点就到头了`] : ['happy', '提剑再入，画卷还在等你']
  },
  mode: r => r.level > 0 ? '一卷终了' : '画卷未完',
  shareLead: r => r.level > 0 ? '一剑破无常，' : '',
})
