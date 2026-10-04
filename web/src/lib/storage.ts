const memory = new Map<string, string | null>()

/** localStorage throws in some private modes and embedded browsers; fall back to memory there. */
export const storage = {
  get(key: string): string | null {
    // A failed write/delete can leave localStorage readable but stale.
    if (memory.has(key)) return memory.get(key) ?? null
    try {
      return localStorage.getItem(key)
    } catch {
      return null
    }
  },
  set(key: string, value: string) {
    try {
      localStorage.setItem(key, value)
      memory.delete(key)
    } catch {
      memory.set(key, value)
    }
  },
  remove(key: string) {
    try {
      localStorage.removeItem(key)
      memory.delete(key)
    } catch {
      memory.set(key, null)
    }
  },
}
