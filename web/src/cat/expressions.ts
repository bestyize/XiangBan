import type { CatSpec } from './catSvg.ts'
import { moodFace } from './moods.ts'

export interface Expression {
  spec: CatSpec
  /** Tile background in the collection */
  tint: string
}

/** Art for the 30 web-unlockable expressions; ids match the server's UnlockCatalog. */
export const expressions: Record<string, Expression> = {
  hello: { spec: { eyes: 'open', mouth: 'smile', blush: true, props: ['note'] }, tint: '#ffe9ad' },
  days_3: { spec: { eyes: 'happy', mouth: 'cat', blush: true, badge: '3' }, tint: '#ffd9c2' },
  days_7: { spec: { eyes: 'wink', mouth: 'tongue', blush: true, badge: '7' }, tint: '#ffcfd8' },
  days_14: { spec: { eyes: 'closed', mouth: 'smile', blush: true, badge: '14', props: ['flower'] }, tint: '#f6d3ff' },
  days_30: { spec: { eyes: 'sparkle', mouth: 'open', blush: true, badge: '30', props: ['moon'] }, tint: '#d6dcff' },
  days_60: { spec: { eyes: 'heart', mouth: 'smile', blush: true, badge: '60' }, tint: '#c9ecff' },
  days_100: { spec: { eyes: 'happy', mouth: 'open', blush: true, badge: '100', props: ['partyHat', 'sparkles'] }, tint: '#ffe066' },
  title_early_leaver: { spec: moodFace.smug, tint: '#c8f2d6' },
  title_on_time: { spec: moodFace.happy, tint: '#ffe9ad' },
  title_normal_human: { spec: moodFace.welcome, tint: '#d4ebff' },
  title_overtime_hero: { spec: moodFace.heartache, tint: '#ffd6cc' },
  title_night_owl: { spec: moodFace.sleepy, tint: '#e1dcff' },
  title_immortal: { spec: moodFace.worried, tint: '#cdf3f5' },
  title_exact_on_time: { spec: moodFace.sparkle, tint: '#ffe58a' },
  title_weekend_work: { spec: moodFace.shocked, tint: '#ffd3ea' },
  title_borrowed_weekend: { spec: { eyes: 'teary', mouth: 'frown', props: ['calendar'] }, tint: '#ffe2c4' },
  title_holiday_countdown: { spec: { eyes: 'heart', mouth: 'open', blush: true, props: ['clock'] }, tint: '#fff0a6' },
  streak_3: { spec: { eyes: 'determined', mouth: 'smile', props: ['flame'] }, tint: '#ffd9c2' },
  streak_5: { spec: { eyes: 'determined', mouth: 'open', blush: true, props: ['flame', 'sparkles'] }, tint: '#ffc9b5' },
  streak_10: { spec: { eyes: 'sparkle', mouth: 'smirk', blush: true, props: ['flame', 'crown'] }, tint: '#ffb9a3' },
  monday: { spec: { eyes: 'dizzy', mouth: 'wavy', props: ['coffee'] }, tint: '#dfe3ea' },
  friday: { spec: { eyes: 'happy', mouth: 'open', blush: true, props: ['note', 'sparkles'] }, tint: '#d2f5c8' },
  rainy: { spec: { eyes: 'open', mouth: 'cat', props: ['umbrella', 'rain'] }, tint: '#cfe3ff' },
  spring_festival: { spec: { eyes: 'happy', mouth: 'smile', blush: true, props: ['envelope'] }, tint: '#ffc2bb' },
  mid_autumn: { spec: { eyes: 'closed', mouth: 'smile', blush: true, props: ['mooncake', 'moon'] }, tint: '#d9d6ff' },
  national_day: { spec: { eyes: 'wink', mouth: 'open', blush: true, props: ['suitcase'] }, tint: '#ffe0a8' },
  comeback: { spec: { eyes: 'teary', mouth: 'smile', blush: true, props: ['heart'] }, tint: '#ffd6e0' },
  redemption: { spec: { eyes: 'squint', mouth: 'open', blush: true, props: ['sparkles'] }, tint: '#c8f2d6' },
  named: { spec: { eyes: 'open', mouth: 'tongue', blush: true, props: ['flower'] }, tint: '#ffe9ad' },
  guardian: { spec: { eyes: 'closed', mouth: 'smile', blush: true, props: ['shield'] }, tint: '#c9f0d3' },
}

export const fallbackExpression: Expression = { spec: moodFace.idle, tint: '#ffe9ad' }
