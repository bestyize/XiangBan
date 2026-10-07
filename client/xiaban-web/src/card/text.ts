/** Characters that must not start a line in Chinese typesetting. */
const NO_LINE_START = '，。！？、；：）」』》…—,.!?;:)'

/** Latin words and numbers ("2048", "AI", "3:00", "42%") stay whole; everything else breaks per character. */
function tokens(text: string): string[] {
  return text.match(/[A-Za-z0-9][A-Za-z0-9.:%+'-]*|[\s\S]/gu) ?? []
}

/**
 * Greedy line breaking for CJK text: measure token by token, never starting a line with
 * closing punctuation, and end the last allowed line with an ellipsis when the text does not fit.
 */
export function wrapText(text: string, maxWidth: number, maxLines: number, measure: (s: string) => number): string[] {
  const parts = tokens(text)
  const lines: string[] = []
  let line = ''
  for (let i = 0; i < parts.length; i++) {
    const next = line + parts[i]
    if (measure(next) <= maxWidth || line === '' || NO_LINE_START.includes(parts[i])) {
      line = next
      continue
    }
    lines.push(line.trimEnd())
    line = parts[i].trimStart()
    if (lines.length === maxLines) {
      lines[maxLines - 1] = [...lines[maxLines - 1]].slice(0, -1).join('') + '…'
      return lines
    }
  }
  if (line) lines.push(line)
  return lines
}

/**
 * Same line count as wrapText, but with the lines evened out, so a bubble never ends on one lonely character.
 * Finds the narrowest width that still needs no extra line.
 */
export function balancedWrap(text: string, maxWidth: number, maxLines: number, measure: (s: string) => number): string[] {
  const greedy = wrapText(text, maxWidth, maxLines, measure)
  if (greedy.length < 2 || greedy[greedy.length - 1].endsWith('…')) return greedy
  let lo = maxWidth / greedy.length
  let hi = maxWidth
  for (let i = 0; i < 14; i++) {
    const mid = (lo + hi) / 2
    const trial = wrapText(text, mid, maxLines + 1, measure)
    if (trial.length <= greedy.length && !trial[trial.length - 1].endsWith('…')) hi = mid
    else lo = mid
  }
  return wrapText(text, hi, maxLines, measure)
}
