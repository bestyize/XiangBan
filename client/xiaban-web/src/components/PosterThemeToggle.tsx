import { POSTER_THEMES, type PosterThemeId } from '../poster/theme.ts'

/** Night or daylight version of a share poster. */
export function PosterThemeToggle({ value, onChange, className = '' }: {
  value: PosterThemeId
  onChange: (id: PosterThemeId) => void
  className?: string
}) {
  return (
    <div role="radiogroup" aria-label="海报配色" className={`share-style-picker ${className}`}>
      {(Object.keys(POSTER_THEMES) as PosterThemeId[]).map((id) => {
        const on = id === value
        return (
          <button key={id} type="button" role="radio" aria-checked={on} onClick={() => onChange(id)}
            className="share-style-option">
            <span aria-hidden="true" className="share-style-swatch" style={{ background: `linear-gradient(${POSTER_THEMES[id].bg[0]}, ${POSTER_THEMES[id].bg[1]})` }} />
            {POSTER_THEMES[id].name}
          </button>
        )
      })}
    </div>
  )
}
