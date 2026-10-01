import { useMemo } from 'react'
import { catSvg, type CatSpec } from '../cat/catSvg.ts'

interface CatProps {
  spec: CatSpec
  label: string
  size?: number
  className?: string
}

export function Cat({ spec, label, size = 200, className = '' }: CatProps) {
  // The markup is generated from our own constants only, never from user input.
  const html = useMemo(() => catSvg(spec), [spec])
  return (
    <span
      role="img"
      aria-label={label}
      className={`inline-block shrink-0 ${className}`}
      style={{ width: size, height: size }}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
