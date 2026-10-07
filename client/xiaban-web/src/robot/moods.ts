import type { Mood } from '../api/types.ts'
import type { RobotSpec } from './expressions.ts'

/** Server moods (copy system) → what the robot shows. */
export const moodFace: Record<Mood, RobotSpec> = {
  idle: { expression: 'idle' },
  smug: { expression: 'proud' },
  happy: { expression: 'happy', symbol: 'sparkle' },
  welcome: { expression: 'greeting' },
  heartache: { expression: 'worried', symbol: 'heart' },
  sleepy: { expression: 'sleepy' },
  worried: { expression: 'worried', symbol: 'sweat' },
  sparkle: { expression: 'excited' },
  shocked: { expression: 'surprised' },
  aggrieved: { expression: 'sad' },
  excited: { expression: 'love' },
  care: { expression: 'shy', symbol: 'heart' },
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

export interface DexArt {
  spec: RobotSpec
  /** Tile background in the collection. */
  tint: string
}

/** Art for the 30 web-unlockable expressions; ids match the server's UnlockCatalog. */
export const dexArt: Record<string, DexArt> = {
  hello: { spec: { expression: 'greeting', symbol: 'note' }, tint: '#ffe9ad' },
  days_3: { spec: { expression: 'happy', badge: '3' }, tint: '#ffd9c2' },
  days_7: { spec: { expression: 'proud', badge: '7' }, tint: '#ffcfd8' },
  days_14: { spec: { expression: 'shy', badge: '14' }, tint: '#f6d3ff' },
  days_30: { spec: { expression: 'excited', badge: '30' }, tint: '#d6dcff' },
  days_60: { spec: { expression: 'love', badge: '60' }, tint: '#c9ecff' },
  days_100: { spec: { expression: 'laughing', badge: '100' }, tint: '#ffe066' },
  title_early_leaver: { spec: moodFace.smug, tint: '#c8f2d6' },
  title_on_time: { spec: moodFace.happy, tint: '#ffe9ad' },
  title_normal_human: { spec: moodFace.welcome, tint: '#d4ebff' },
  title_overtime_hero: { spec: moodFace.heartache, tint: '#ffd6cc' },
  title_night_owl: { spec: moodFace.sleepy, tint: '#e1dcff' },
  title_immortal: { spec: { expression: 'charging' }, tint: '#cdf3f5' },
  title_exact_on_time: { spec: { expression: 'excited', symbol: 'clock' }, tint: '#ffe58a' },
  title_weekend_work: { spec: moodFace.shocked, tint: '#ffd3ea' },
  title_borrowed_weekend: { spec: { expression: 'sad', symbol: 'calendar' }, tint: '#ffe2c4' },
  title_holiday_countdown: { spec: { expression: 'love', symbol: 'clock' }, tint: '#fff0a6' },
  streak_3: { spec: { expression: 'proud', symbol: 'flame' }, tint: '#ffd9c2' },
  streak_5: { spec: { expression: 'excited', symbol: 'flame' }, tint: '#ffc9b5' },
  streak_10: { spec: { expression: 'proud', symbol: 'crown' }, tint: '#ffb9a3' },
  monday: { spec: { expression: 'sleepy', symbol: 'coffee' }, tint: '#dfe3ea' },
  friday: { spec: { expression: 'laughing', symbol: 'note' }, tint: '#d2f5c8' },
  rainy: { spec: { expression: 'idle', symbol: 'rain' }, tint: '#cfe3ff' },
  spring_festival: { spec: { expression: 'happy', symbol: 'envelope' }, tint: '#ffc2bb' },
  mid_autumn: { spec: { expression: 'shy', symbol: 'mooncake' }, tint: '#d9d6ff' },
  national_day: { spec: { expression: 'greeting', symbol: 'suitcase' }, tint: '#ffe0a8' },
  comeback: { spec: { expression: 'happy', symbol: 'heart', face: { tears: 0.55 } }, tint: '#ffd6e0' },
  redemption: { spec: { expression: 'laughing', symbol: 'sparkle' }, tint: '#c8f2d6' },
  named: { spec: { expression: 'happy', symbol: 'flower' }, tint: '#ffe9ad' },
  guardian: { spec: { expression: 'shy', symbol: 'shield' }, tint: '#c9f0d3' },
}

export const fallbackArt: DexArt = { spec: { expression: 'idle' }, tint: '#ffe9ad' }
