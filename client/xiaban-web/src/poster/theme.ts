import { useCallback, useState } from 'react'

export type PosterThemeId = 'night' | 'day'

/** Colors for one look of every share poster. */
export interface PosterTheme {
  id: PosterThemeId
  name: string
  /** Background, top → bottom */
  bg: [string, string]
  /** Warm lamp light behind 小伴 */
  glow: string
  ink: string
  sub: string
  faint: string
  /** Brand amber, tuned to stay readable on this background */
  accent: string
  /** The cool screen-light blue of 小伴's eyes, for small details */
  cool: string
  panel: string
  panelLine: string
  bubble: string
  bubbleInk: string
  chip: string
  qrInk: string
  /** Night skies get a few stars; daylight stays plain */
  stars: boolean
}

export const POSTER_THEMES: Record<PosterThemeId, PosterTheme> = {
  // For WeChat moments and group chats: the lit window at night.
  night: {
    id: 'night', name: '深夜',
    bg: ['#2d3f58', '#141c2f'],
    glow: 'rgba(255, 196, 110, 0.34)',
    ink: '#f5f0e6', sub: 'rgba(245, 240, 230, 0.68)', faint: 'rgba(245, 240, 230, 0.42)',
    accent: '#f0c07a', cool: '#8fd0ff',
    panel: 'rgba(255, 255, 255, 0.06)', panelLine: 'rgba(245, 240, 230, 0.12)',
    bubble: '#f7f0e3', bubbleInk: '#202a39',
    chip: 'rgba(255, 255, 255, 0.07)', qrInk: '#182338', stars: true,
  },
  // For Xiaohongshu feeds: bright and warm, the same cream as 小伴's shell.
  day: {
    id: 'day', name: '暖光',
    bg: ['#f9f3ea', '#efe3d1'],
    glow: 'rgba(255, 184, 88, 0.30)',
    ink: '#2b2622', sub: '#7b6e61', faint: '#a99a89',
    accent: '#b5712a', cool: '#2f86d6',
    panel: '#fffaf3', panelLine: 'rgba(122, 92, 58, 0.12)',
    bubble: '#ffffff', bubbleInk: '#2b2622',
    chip: '#f5ecdf', qrInk: '#2b2622', stars: false,
  },
}

const KEY = 'xb_poster_theme'

export function loadPosterTheme(): PosterThemeId {
  try {
    return localStorage.getItem(KEY) === 'day' ? 'day' : 'night'
  } catch {
    return 'night'
  }
}

export function savePosterTheme(id: PosterThemeId) {
  try {
    localStorage.setItem(KEY, id)
  } catch {
    // Private mode: the choice just isn't remembered.
  }
}

/** The look last picked on any share screen, so every poster follows it. */
export function usePosterTheme(): [PosterThemeId, (id: PosterThemeId) => void] {
  const [id, setId] = useState(loadPosterTheme)
  const set = useCallback((next: PosterThemeId) => {
    savePosterTheme(next)
    setId(next)
  }, [])
  return [id, set]
}
