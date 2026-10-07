import { validScores } from './scoring.ts'
import type { QuizDefinition } from './types.ts'

// Fragments never enter the HTTP request; only aggregate scores are included.
export function shareLink(quiz: QuizDefinition, scores: number[], base: string): string {
  if (!validScores(quiz, scores)) throw new Error('无法分享不完整的报告。')
  const url = new URL(`/tests/${quiz.id}/report`, base)
  url.hash = new URLSearchParams({ v: String(quiz.version), s: scores.join(',') }).toString()
  return url.toString()
}

export function readSharedScores(quiz: QuizDefinition, fragment: string): number[] | null {
  if (!fragment || fragment.length > 160) return null
  const params = new URLSearchParams(fragment.replace(/^#/, ''))
  if (params.size !== 2 || params.get('v') !== String(quiz.version)) return null
  const encoded = params.get('s')
  if (!encoded || !/^-?\d+(,-?\d+)*$/.test(encoded)) return null
  const scores = encoded.split(',').map(Number)
  return validScores(quiz, scores) ? scores : null
}
