import { useCallback, useState } from 'react'

export type PosterThemeId = 'night' | 'day'

/** Colors for one look of every share poster. */
export interface PosterTheme {
  id: PosterThemeId
  name: string
  /** Background, top → bottom */
  bg: [string, string]
  /** Soft blush light behind 小伴 */
  glow: string
  ink: string
  sub: string
  faint: string
  /** Brand red, tuned to stay readable on this background */
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
  // Exported poster palettes stay independent of the app's appearance setting.
  night: {
    id: 'night', name: '深色',
    bg: ['#242124', '#141414'],
    glow: 'rgba(255, 107, 129, 0.14)',
    ink: '#f2f2f2', sub: '#b8b8b8', faint: '#949494',
    accent: '#ff6b81', cool: '#8fd0ff',
    panel: '#292629', panelLine: 'rgba(255, 255, 255, 0.10)',
    bubble: '#33282d', bubbleInk: '#f2f2f2',
    chip: '#3a3035', qrInk: '#262626', stars: false,
  },
  day: {
    id: 'day', name: '浅色',
    bg: ['#fff9fa', '#f7f7f7'],
    glow: 'rgba(229, 27, 62, 0.08)',
    ink: '#262626', sub: '#666666', faint: '#737373',
    accent: '#e51b3e', cool: '#2f86d6',
    panel: '#ffffff', panelLine: 'rgba(38, 38, 38, 0.08)',
    bubble: '#ffffff', bubbleInk: '#262626',
    chip: '#fff0f3', qrInk: '#262626', stars: false,
  },
}

const KEY = 'xb_poster_theme'

export function loadPosterTheme(): PosterThemeId {
  try {
    return localStorage.getItem(KEY) === 'night' ? 'night' : 'day'
  } catch {
    return 'day'
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
