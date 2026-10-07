import type { TitleCode } from '../api/types.ts'

/** Accent per title, shared by the result page and the share card. */
export interface TitleTheme {
  accent: string
  /** Card background, top to bottom */
  sky: [string, string]
}

export const titleTheme: Record<TitleCode, TitleTheme> = {
  early_leaver: { accent: '#7ee0a1', sky: ['#1f5c4a', '#123629'] },
  on_time: { accent: '#ffb938', sky: ['#3a3f8f', '#1f2758'] },
  normal_human: { accent: '#8cc8ff', sky: ['#2d4f8f', '#1a2d57'] },
  overtime_hero: { accent: '#ff8f7a', sky: ['#5a2f6e', '#2a1840'] },
  night_owl: { accent: '#b7a6ff', sky: ['#2b2466', '#140f33'] },
  immortal: { accent: '#7fe3ea', sky: ['#13314a', '#071624'] },
  exact_on_time: { accent: '#ffd76a', sky: ['#6b4a12', '#2e1f05'] },
  weekend_work: { accent: '#ff9bd2', sky: ['#5c2148', '#2b0e22'] },
  borrowed_weekend: { accent: '#ffc38a', sky: ['#5a3a24', '#2a1a0e'] },
  holiday_countdown: { accent: '#ffe066', sky: ['#7a2a1f', '#3a120b'] },
}
