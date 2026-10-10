import { useMemo, useState } from 'react'
import type { City } from '../api/types.ts'

interface CityPickerProps {
  cities: City[]
  value: string | null
  onChange: (code: string) => void
}

export function CityPicker({ cities, value, onChange }: CityPickerProps) {
  const [query, setQuery] = useState('')
  const shown = useMemo(() => {
    const q = query.trim()
    if (!q) return cities
    return cities.filter((c) => c.name.includes(q) || c.province.includes(q))
  }, [cities, query])

  return (
    <div>
      <label className="sr-only" htmlFor="city-search">搜索城市</label>
      <input
        id="city-search"
        type="search"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="搜城市，比如：杭州"
        className="w-full rounded-2xl border-2 border-dusk-soft bg-dusk-deep/60 px-4 py-3 text-window placeholder:text-window/40 focus:border-lamp focus:outline-none"
      />
      <div className="mt-3 flex max-h-[34vh] flex-wrap gap-2 overflow-y-auto pb-1" role="radiogroup" aria-label="城市">
        {shown.map((c) => {
          const selected = c.code === value
          return (
            <button
              key={c.code}
              type="button"
              role="radio"
              aria-checked={selected}
              onClick={() => onChange(c.code)}
              className={`rounded-full border-2 px-4 py-1.5 text-[15px] transition ${
                selected ? 'border-lamp bg-action font-bold text-white' : 'border-dusk-soft text-window/85 hover:border-window/60'
              }`}
            >
              {c.name}
            </button>
          )
        })}
        {shown.length === 0 && <p className="text-sm text-window/60">没找到，可以选「其他城市」。</p>}
      </div>
    </div>
  )
}
