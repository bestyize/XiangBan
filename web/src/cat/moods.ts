import type { Mood } from '../api/types.ts'
import type { CatSpec } from './catSvg.ts'

export const moodFace: Record<Mood, CatSpec> = {
  idle: { eyes: 'open', mouth: 'cat', blush: true },
  smug: { eyes: 'smug', mouth: 'smirk', blush: true },
  happy: { eyes: 'happy', mouth: 'open', blush: true, props: ['sparkles'] },
  welcome: { eyes: 'closed', mouth: 'smile', blush: true },
  heartache: { eyes: 'teary', mouth: 'wavy', props: ['heart'] },
  sleepy: { eyes: 'sleepy', mouth: 'o', props: ['zzz'] },
  worried: { eyes: 'worried', mouth: 'frown', props: ['sweat'] },
  sparkle: { eyes: 'sparkle', mouth: 'open', blush: true, props: ['crown', 'sparkles'] },
  shocked: { eyes: 'shocked', mouth: 'o', props: ['exclaim'] },
  aggrieved: { eyes: 'teary', mouth: 'frown', props: ['question'] },
  excited: { eyes: 'heart', mouth: 'open', blush: true, props: ['sparkles'] },
  care: { eyes: 'closed', mouth: 'smile', blush: true, props: ['heart'] },
}

export const moodLabel: Record<Mood, string> = {
  idle: '在等你',
  smug: '得意',
  happy: '开心',
  welcome: '欢迎回来',
  heartache: '心疼',
  sleepy: '犯困',
  worried: '担心',
  sparkle: '惊喜',
  shocked: '震惊',
  aggrieved: '委屈',
  excited: '期待',
  care: '想抱抱你',
}
