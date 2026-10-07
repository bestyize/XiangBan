import type { RobotInput } from '../../robot/expressions.ts'

export interface GamePosterCopy {
  gameId: string
  name: string
  mode: string
  /** What 小伴 says about this round */
  cheer: string
  face: RobotInput
  score: number
  stats: [string, string][]
}

/** The host's record of the current round, which a share request must match. */
export interface RecordedRound { finished: boolean; score: number; lines: number; maxTile: number }

/**
 * Everything the website needs to know about one game's results. Each game directory exports one of these;
 * `src/games/index.ts` registers them. Results only ever carry final aggregate metrics, never account data.
 */
export interface ResultGame<R extends { score: number } = { score: number }> {
  id: string
  name: string
  /** `/games/<id>` opens without onboarding even when it is not a share link. */
  openDirectly?: boolean
  /** Options from the page URL forwarded to the pack entry (for example a challenge's round type). */
  launchParams?(search: URLSearchParams): Record<string, string>
  /** Validates a `game.share` payload from the pack. */
  parse(value: unknown): R | null
  /** The host only shares the finished current round with exactly the recorded metrics. */
  canShare(result: R, round: RecordedRound | null): boolean
  fragment(result: R): string
  /** Reads a share-page fragment; null for anything incomplete or ambiguous. */
  read(fragment: string): R | null
  /** Extra query parameters a challenge link keeps, so a friend plays the same setup. */
  challengeQuery?(result: R | null): Record<string, string>
  subtitle(result: R): string
  shareText(result: R): string
  poster(result: R): GamePosterCopy
}
