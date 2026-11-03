import { test } from 'node:test'
import assert from 'node:assert/strict'
import { AiStreamDecoder, consumeAiStream } from '../src/lib/aiStream.ts'
import { aiHistoryStorage, boundedAiHistory, AI_HISTORY_LIMITS, readAiHistory, writeAiHistory } from '../src/lib/aiHistory.ts'

test('SSE handles split UTF-8, CRLF and multiline events', () => {
  const events: string[] = []
  const decoder = new AiStreamDecoder(data => events.push(data))
  const bytes = new TextEncoder().encode(': heartbeat\r\ndata: 你好\r\ndata: world\r\n\r\ndata: [DONE]\n\n')
  for (const byte of bytes) decoder.push(new Uint8Array([byte]))
  decoder.push(new Uint8Array(), true)
  assert.deepEqual(events, ['你好\nworld', '[DONE]'])
})
test('history stays isolated by account and tolerates bad storage', () => {
  const data = new Map<string, string>()
  const storage = { getItem: (key: string) => data.get(key) ?? null, setItem: (key: string, value: string) => data.set(key, value) } as unknown as Storage
  assert.equal(writeAiHistory(storage, 'a', [{ id: '1', title: 'hello', messages: [{ role: 'user', content: 'private' }] }]), true)
  assert.equal(readAiHistory(storage, 'a')[0]?.messages[0]?.content, 'private')
  assert.deepEqual(readAiHistory(storage, 'b'), [])
  data.set('xiangban.ai.history.v1.b', '{')
  assert.deepEqual(readAiHistory(storage, 'b'), [])
  assert.equal(writeAiHistory({ setItem: () => { throw new Error('full') } } as unknown as Storage, 'a', []), false)
})

test('DONE terminates an open stream and cancels the reader', { timeout:1000 }, async () => {
  let cancelled = false
  const stream = new ReadableStream<Uint8Array>({
    start(controller) { controller.enqueue(new TextEncoder().encode('data: {"choices":[{"delta":{"content":"完成"}}]}\n\ndata: [DONE]\n\n')) },
    cancel() { cancelled = true },
  })
  const received:string[] = []
  await consumeAiStream(stream, text => received.push(text))
  assert.deepEqual(received,['完成'])
  assert.equal(cancelled,true)
  assert.equal(stream.locked,false)
})

test('malformed/error events and callback failure cancel the stream', async () => {
  for (const event of ['not-json', '{"error":{"code":"unavailable"}}', 'null', '{"choices":[{"delta":{"content":"text"}}]}']) {
    let cancelled = false
    const stream = new ReadableStream<Uint8Array>({ start(c) { c.enqueue(new TextEncoder().encode(`data: ${event}\n\n`)) }, cancel() { cancelled = true } })
    await assert.rejects(consumeAiStream(stream, () => { throw new Error('too long') }))
    assert.equal(cancelled,true)
    assert.equal(stream.locked,false)
  }
})

test('truncated streams fail and final unterminated DONE is accepted', async () => {
  const stream = (text:string) => new ReadableStream<Uint8Array>({start(c) { c.enqueue(new TextEncoder().encode(text)); c.close() }})
  await assert.rejects(consumeAiStream(stream('data: {"choices":[]}\n\n'),()=>{}), /意外中断/)
  await consumeAiStream(stream('data: [DONE]'),()=>{})
})

test('abort unblocks a pending reader and cancels its source', { timeout:1000 }, async () => {
  let cancelled = false
  const stream = new ReadableStream<Uint8Array>({cancel() { cancelled = true }})
  const controller = new AbortController()
  const pending = consumeAiStream(stream,()=>{},controller.signal)
  controller.abort()
  await assert.rejects(pending, {name:'AbortError'})
  assert.equal(cancelled,true)
  assert.equal(stream.locked,false)
})

test('restricted localStorage access never escapes and oversized history is bounded', () => {
  assert.equal(aiHistoryStorage(() => { throw new Error('denied') }),null)
  assert.deepEqual(readAiHistory(null,'a'),[])
  assert.equal(writeAiHistory(null,'a',[]),false)
  const conversations = Array.from({length:35},(_,i) => ({id:String(i),title:'title',messages:[{role:'user' as const,content:'x'.repeat(50_000)},{role:'assistant' as const,content:'answer'}]}))
  const bounded = boundedAiHistory(conversations)
  assert.ok(bounded.length <= AI_HISTORY_LIMITS.conversations)
  assert.equal(bounded[0]?.id,'0')
  assert.ok(JSON.stringify(bounded).length <= AI_HISTORY_LIMITS.serialized)
  assert.ok(bounded.length < conversations.length)
  const huge = boundedAiHistory([{id:'x',title:'x',messages:Array.from({length:104},(_,i) => ({role:i % 2 ? 'assistant' : 'user',content:'text'}))}])
  assert.equal(huge[0]?.messages.length,100)
  assert.equal(huge[0]?.messages[0]?.role,'user')
  assert.deepEqual(readAiHistory({getItem:()=> ' '.repeat(AI_HISTORY_LIMITS.serialized+1),setItem:()=>{}},'a'),[])
})

test('completion does not wait for a non-closing cancellation promise', { timeout:1000 }, async () => {
  let cancelled = false
  const stream = new ReadableStream<Uint8Array>({start(c) { c.enqueue(new TextEncoder().encode('data: [DONE]\n\n')) },cancel() { cancelled = true; return new Promise<void>(()=>{}) }})
  await consumeAiStream(stream,()=>{})
  assert.equal(cancelled,true)
  assert.equal(stream.locked,false)
})

function eventStream(...events: unknown[]): ReadableStream<Uint8Array> {
  const encoder = new TextEncoder()
  return new ReadableStream({
    start(controller) {
      for (const event of events) controller.enqueue(encoder.encode(`data: ${JSON.stringify(event)}\n\n`))
      controller.close()
    },
  })
}

test('Responses displays answer deltas once and ignores reasoning and final snapshots', async () => {
  const received: string[] = []
  await consumeAiStream(eventStream(
    { type: 'response.reasoning_summary_text.delta', delta: '内部摘要' },
    { type: 'response.output_text.delta', delta: '你好' },
    { type: 'response.output_text.delta', delta: '！' },
    { type: 'response.output_text.done', text: '你好！' },
    { type: 'response.completed', response: { status: 'completed' } },
  ), text => received.push(text), undefined, 'responses')
  assert.deepEqual(received, ['你好', '！'])
})

test('Responses requires successful completion', async () => {
  for (const type of ['response.failed', 'response.incomplete', 'error']) {
    await assert.rejects(consumeAiStream(eventStream({ type }), () => {}, undefined, 'responses'), /回复中断/)
  }
  await assert.rejects(consumeAiStream(eventStream({ type: 'response.created' }), () => {}, undefined, 'responses'), /意外中断/)
})

test('Chat Completions only displays the primary answer', async () => {
  const received: string[] = []
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(new TextEncoder().encode(
        'data: {"choices":[{"index":1,"delta":{"content":"其他答案"}},{"index":0,"delta":{"content":"正文","reasoning_content":"思考"}}]}\n\ndata: [DONE]\n\n',
      ))
      controller.close()
    },
  })
  await consumeAiStream(stream, text => received.push(text))
  assert.deepEqual(received, ['正文'])
})

test('invalid UTF-8 fails instead of displaying replacement characters', async () => {
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      controller.enqueue(Uint8Array.of(0xff))
      controller.close()
    },
  })
  await assert.rejects(consumeAiStream(stream, () => assert.fail('invalid bytes reached the UI')), /编码异常/)
})

test('Chat Completions stops before appending text from a second completion', async () => {
  const received: string[] = []
  const chunk = (id: string, content: string) => ({ id, choices: [{ index: 0, delta: { content } }] })
  await assert.rejects(consumeAiStream(eventStream(
    chunk('chatcmpl-greeting', 'Hi! How can'),
    chunk('chatcmpl-unrelated', '## 错误原因'),
  ), text => received.push(text)), /回复流异常/)
  assert.deepEqual(received, ['Hi! How can'])
})
