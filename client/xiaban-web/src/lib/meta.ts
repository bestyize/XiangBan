import { useEffect, useState } from 'react'
import { api } from '../api/client.ts'
import type { Meta } from '../api/types.ts'

let cached: Promise<Meta> | null = null

export function useMeta(): Meta | null {
  const [meta, setMeta] = useState<Meta | null>(null)
  useEffect(() => {
    let alive = true
    cached ??= api<Meta>('/api/meta')
    cached.then((m) => alive && setMeta(m)).catch(() => {
      cached = null
    })
    return () => {
      alive = false
    }
  }, [])
  return meta
}
