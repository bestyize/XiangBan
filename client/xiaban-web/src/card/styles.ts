import type { Checkin } from '../api/types.ts'
import { titleTheme } from '../lib/titleTheme.ts'
import type { PosterThemeId } from '../poster/theme.ts'

export type Decoration = 'stars' | 'confetti' | 'lanterns' | 'moon'

export interface CardStyle {
  id: string
  name: string
  /** Base look shared with every other poster */
  theme: PosterThemeId
  /** Night-only variants paint their own sky and title color */
  sky?: [string, string]
  accent?: string
  decoration: Decoration | null
}

const HOLIDAY: Record<string, Pick<CardStyle, 'sky' | 'accent' | 'decoration'>> = {
  spring_festival: { sky: ['#9b1c1c', '#3d0707'], accent: '#ffd76a', decoration: 'lanterns' },
  mid_autumn: { sky: ['#1b2f6b', '#070d26'], accent: '#ffe6a3', decoration: 'moon' },
}

/** PRD 5.7: the two shared looks, one per title, and a limited style for rare titles and holidays. */
export function cardStyles(checkin: Checkin): CardStyle[] {
  const theme = titleTheme[checkin.title.code]
  const styles: CardStyle[] = [
    { id: 'night', name: '深夜', theme: 'night', decoration: 'stars' },
    { id: 'day', name: '暖光', theme: 'day', decoration: null },
    { id: 'title', name: `${checkin.title.name}配色`, theme: 'night', sky: theme.sky, accent: theme.accent, decoration: 'stars' },
  ]
  if (checkin.holiday) {
    const look = HOLIDAY[checkin.holiday.code] ?? { sky: ['#8a2318', '#2e0a05'], accent: '#ffd76a', decoration: 'confetti' }
    styles.push({ id: 'holiday', name: `${checkin.holiday.name}限定`, theme: 'night', ...look })
  } else if (checkin.title.rare) {
    styles.push({ id: 'rare', name: '稀有限定', theme: 'night', sky: ['#5b3d0c', '#1c1203'], accent: '#ffd76a', decoration: 'confetti' })
  }
  return styles
}

/** One number for the card: the percentile when it flatters, otherwise the streak or 相伴天数. */
export function cardStatLine(checkin: Checkin, companionName: string): string {
  const { percentile, streak, totalDays } = checkin.stats
  if (percentile && percentile.percent >= 20) {
    const where = percentile.scope === 'city' && percentile.cityName ? percentile.cityName : ''
    return `比${where} ${percentile.percent}% 的下班啦用户下班早`
  }
  if (streak >= 2) return `已经连续打卡 ${streak} 天`
  return `和${companionName}相伴的第 ${totalDays} 天`
}
