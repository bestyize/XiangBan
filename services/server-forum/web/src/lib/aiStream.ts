/** Incremental SSE decoder: TextDecoder preserves split UTF-8 sequences and events span chunks. */
export class AiStreamDecoder {
  private decoder = new TextDecoder()
  private buffer = ''
  private data: string[] = []
  private onData: (data: string) => void
  constructor(onData: (data: string) => void) { this.onData = onData }
  push(chunk: Uint8Array, final = false) {
    this.buffer += this.decoder.decode(chunk, { stream: !final })
    if (this.buffer.length + this.data.join('').length > 1_048_576) throw new Error('AI 回复数据过大，请重试。')
    // A CR may be the first half of CRLF; keep it until the next chunk.
    let match: RegExpExecArray | null
    while ((match = /\r\n|\n|\r(?!$)/.exec(this.buffer))) {
      const line = this.buffer.slice(0, match.index)
      this.buffer = this.buffer.slice(match.index + match[0].length)
      this.line(line)
    }
    if (final) {
      if (this.buffer.endsWith('\r')) this.buffer = this.buffer.slice(0, -1)
      if (this.buffer) this.line(this.buffer)
      this.buffer = ''
      this.line('')
    }
  }
  private line(line: string) {
    if (!line) {
      if (this.data.length) this.onData(this.data.join('\n'))
      this.data = []
    } else if (line === 'data' || line.startsWith('data:')) {
      this.data.push(line === 'data' ? '' : line.slice(5).replace(/^ /, ''))
    }
  }
}

/** Finish on the protocol terminator rather than waiting for the HTTP connection to close. */
export async function consumeAiStream(body: ReadableStream<Uint8Array>, onText: (text: string) => void, signal?: AbortSignal): Promise<void> {
  let complete = false
  const parser = new AiStreamDecoder(data => {
    if (complete) return
    if (data.trim() === '[DONE]') { complete = true; return }
    let event: unknown
    try { event = JSON.parse(data) } catch { throw new Error('AI 回复格式异常，请重试。') }
    if (!event || typeof event !== 'object') throw new Error('AI 回复格式异常，请重试。')
    if ('error' in event && event.error) throw new Error('AI 回复中断，请重试。')
    const choices = 'choices' in event ? event.choices : undefined
    if (Array.isArray(choices)) {
      const text = choices[0]?.delta?.content
      if (typeof text === 'string') onText(text)
    }
  })
  const reader = body.getReader()
  const abort = () => { void reader.cancel().catch(() => undefined) }
  signal?.addEventListener('abort', abort, { once: true })
  try {
    signal?.throwIfAborted()
    while (!complete) {
      const { value, done } = await reader.read()
      signal?.throwIfAborted()
      if (done) { parser.push(new Uint8Array(), true); break }
      parser.push(value)
    }
    if (!complete) throw new Error('AI 回复意外中断，请重试。')
  } finally {
    signal?.removeEventListener('abort', abort)
    // This also cancels malformed streams and releases pending network work on callback errors.
    void reader.cancel().catch(() => undefined)
    reader.releaseLock()
  }
}
