/** Keep only publicly visible works, preserving server order within each trust tier. */
export function publicWorks<T extends { status: string; trust_level: number }>(items: T[]): T[] {
  return items.filter(item => item.status === 'published').sort((a, b) => a.trust_level - b.trust_level)
}
