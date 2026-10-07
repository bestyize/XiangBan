import { POSTER_THEMES, type PosterThemeId } from '../poster/theme.ts'

const HINT: Record<PosterThemeId, string> = { night: '适合朋友圈', day: '适合小红书' }

/** Night or daylight version of a share poster. */
export function PosterThemeToggle({ value, onChange, tone = 'dark', className = '' }: {
  value: PosterThemeId
  onChange: (id: PosterThemeId) => void
  /** Colors of the page the toggle sits on */
  tone?: 'dark' | 'light'
  className?: string
}) {
  const idle = tone === 'dark' ? 'border-window/20 text-window/75' : 'border-[#d9cbb7] text-[#6f6255]'
  return (
    <div role="radiogroup" aria-label="海报配色" className={`flex justify-center gap-2 ${className}`}>
      {(Object.keys(POSTER_THEMES) as PosterThemeId[]).map((id) => {
        const on = id === value
        return (
          <button key={id} type="button" role="radio" aria-checked={on} onClick={() => onChange(id)}
            className={`flex min-h-11 items-center gap-1.5 rounded-full border-2 px-4 text-sm transition active:scale-95 ${on ? 'border-lamp bg-lamp font-bold text-ink' : idle}`}>
            <span aria-hidden="true" className="size-3 rounded-full border border-black/10" style={{ background: `linear-gradient(${POSTER_THEMES[id].bg[0]}, ${POSTER_THEMES[id].bg[1]})` }} />
            {POSTER_THEMES[id].name}
            <span className={`text-xs ${on ? 'text-ink/70' : 'opacity-70'}`}>{HINT[id]}</span>
          </button>
        )
      })}
    </div>
  )
}
