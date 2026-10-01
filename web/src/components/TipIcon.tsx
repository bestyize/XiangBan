import type { TipKind } from '../api/types.ts'

const paths: Record<TipKind, string> = {
  rain: 'M4 11a8 8 0 0 1 16 0Z M12 11v7a2 2 0 0 1-4 0',
  cold: 'M12 3v18 M5 7l14 10 M19 7 5 17 M9 4l3 3 3-3 M9 20l3-3 3 3',
  traffic: 'M5 16V11l2-5h10l2 5v5Z M5 16v2 M19 16v2 M8 13h.01 M16 13h.01',
  holiday: 'M5 6h14v14H5Z M5 10h14 M9 4v4 M15 4v4 M9 15l2 2 4-4',
  makeup: 'M5 6h14v14H5Z M5 10h14 M9 4v4 M15 4v4 M12 13v4 M10 15h4',
  morning: 'M4 18h16 M7 18a5 5 0 0 1 10 0 M12 5v3 M5 9l2 2 M19 9l-2 2',
  duty: 'M12 5a8 8 0 1 0 0 16 8 8 0 0 0 0-16Z M12 9v4l3 2',
  week: 'M5 19V11 M10 19V7 M15 19v-6 M20 19V9',
  month: 'M12 3c3 4 5 6 5 10a5 5 0 0 1-10 0c0-2 1-4 2-5 0 2 1 3 2 3 0-3 0-5 1-8Z',
  payday: 'M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16Z M9 9l3 3 3-3 M12 12v5 M9.5 14h5',
}

export function TipIcon({ kind }: { kind: TipKind }) {
  return (
    <svg viewBox="0 0 24 24" className="size-5 shrink-0" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={paths[kind]} />
    </svg>
  )
}
