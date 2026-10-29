import type { ExpressionName } from './expressions.ts'
import type { Beat } from './mind.ts'

export type PresenceKind = 'welcome_back' | 'first_today' | 'late_night'

export interface PresenceStep {
  expression: ExpressionName
  seconds: number
}

const SEQUENCES: Record<PresenceKind, PresenceStep[]> = {
  welcome_back: [{ expression: 'greeting', seconds: 2 }, { expression: 'happy', seconds: 1 }],
  first_today: [{ expression: 'greeting', seconds: 2.2 }, { expression: 'happy', seconds: 1.2 }],
  late_night: [{ expression: 'sleepy', seconds: 3.5 }],
}

export const isPresenceKind = (value: unknown): value is PresenceKind =>
  typeof value === 'string' && Object.hasOwn(SEQUENCES, value)

export const presencePlay = (kind: PresenceKind): PresenceStep[] => SEQUENCES[kind].map((step) => ({ ...step }))

export const presenceBeats = (kind: PresenceKind): Beat[] =>
  presencePlay(kind).map(({ expression, seconds }) => ({ expr: expression, hold: seconds }))

/** Subscribes to the native `companion-presence` event; resolves to the unsubscribe function. */
export function listenPresence(
  listen: (event: string, callback: (payload: unknown) => void) => Promise<() => void>,
  play: (beats: Beat[]) => void,
  isEnabled: () => boolean,
): Promise<() => void> {
  return listen('companion-presence', (payload) => {
    if (!isEnabled()) return
    const kind = (payload as { kind?: unknown } | null | undefined)?.kind
    if (isPresenceKind(kind)) play(presenceBeats(kind))
  })
}
