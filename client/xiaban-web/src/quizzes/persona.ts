import type { Axis, QuizQuestion } from './types.ts'

export type PersonaItem = readonly [Axis, string, readonly [string, string, string, string]]

/** Each scenario contrasts two preferences; options rotate to avoid a fixed winning position. */
export function personaQuestions(id: 'animal' | 'afterhours', items: readonly PersonaItem[]): QuizQuestion[] {
  const values = [-2, -1, 1, 2]
  return items.map(([axis, prompt, labels], index) => {
    const options = labels.map((label, i) => ({ label, value: values[i] }))
    const offset = index % options.length
    return {
      id: `${id}-${index + 1}`, prompt, axis, direction: 1, source: 'original',
      options: [...options.slice(offset), ...options.slice(0, offset)],
    }
  })
}
