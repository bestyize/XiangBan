/** Characters that must not start a line in Chinese typesetting. */
const NO_LINE_START = '，。！？、；：）」』》…—,.!?;:)'

/**
 * Greedy line breaking for CJK text: measure character by character, never starting a line with
 * closing punctuation, and end the last allowed line with an ellipsis when the text does not fit.
 */
export function wrapText(text: string, maxWidth: number, maxLines: number, measure: (s: string) => number): string[] {
  const chars = [...text]
  const lines: string[] = []
  let line = ''
  for (let i = 0; i < chars.length; i++) {
    const next = line + chars[i]
    if (measure(next) <= maxWidth || line === '' || NO_LINE_START.includes(chars[i])) {
      line = next
      continue
    }
    lines.push(line)
    line = chars[i]
    if (lines.length === maxLines) {
      lines[maxLines - 1] = lines[maxLines - 1].slice(0, -1) + '…'
      return lines
    }
  }
  if (line) lines.push(line)
  return lines
}
