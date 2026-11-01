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
