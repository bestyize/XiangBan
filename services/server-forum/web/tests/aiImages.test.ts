import {test} from 'node:test'
import assert from 'node:assert/strict'
import {boundedImageResponse, generatedImageBlob, generatedImageItems, imagePayload, type ImageCapability} from '../src/lib/aiImages.ts'
const gif = Uint8Array.from(atob('R0lGODlhAQABAIAAAP8AAAAA/yH/C05FVFNDQVBFMi4wAwEAAAAh+QQAFAAAACwAAAAAAQABAAACAkQBACH5BAAUAAAALAAAAAABAAEAAAICTAEAOw=='), c => c.charCodeAt(0))
const capability: ImageCapability = {available:true,sizes:['1024x1024'],formats:['png','gif']}

test('GIF payload uses only the offered format and bounded animation parameters', () => {
  assert.deepEqual(imagePayload({prompt:' 猫挥手 ',n:1,output_format:'gif'},capability), {prompt:'猫挥手',n:1,size:'1024x1024',output_format:'gif',frame_count:9,frame_duration_ms:200})
  assert.deepEqual(imagePayload({prompt:'cat',n:4,output_format:'png'},capability), {prompt:'cat',n:4,size:'1024x1024'})
  assert.throws(() => imagePayload({prompt:'cat',n:1,output_format:'gif'},{...capability,formats:undefined}),/尚未开放/)
  for (const bad of [{n:2},{frame_count:1},{frame_count:17},{frame_duration_ms:19},{frame_duration_ms:2001},{size:'8x8'}]) {
    assert.throws(() => imagePayload({prompt:'cat',n:1,output_format:'gif',...bad},capability))
  }
})
test('animation blobs retain all original bytes and reject truncated or oversized animations', async () => {
  const blob = generatedImageBlob(gif)
  assert.equal(blob.type, 'image/gif')
  assert.deepEqual(new Uint8Array(await blob.arrayBuffer()), gif)
  assert.throws(() => generatedImageBlob(gif.subarray(0,45)))
  const giant = gif.slice(); giant[6] = 255; giant[7] = 255
  assert.throws(() => generatedImageBlob(giant))
  assert.throws(() => generatedImageBlob(new TextEncoder().encode('<svg onload="alert(1)">')))
})
test('bounded responses cancel excess data and failed items preserve completed images', async () => {
  let cancelled = false
  const response = new Response(new ReadableStream({start(c){c.enqueue(new Uint8Array(10))},cancel(){cancelled=true}}))
  await assert.rejects(boundedImageResponse(response,5)); assert.equal(cancelled,true)
  const images = await generatedImageItems({data:[{b64_json:btoa(String.fromCharCode(...gif))},{b64_json:'invalid'}]},2,new AbortController().signal)
  assert.equal(images.length,1); assert.equal(images[0]!.type,'image/gif')
})
test('image downloads carry no gateway credentials or cookies and disallow unsafe addresses', async () => {
  const original = globalThis.fetch
  const calls: RequestInit[] = []
  globalThis.fetch = async (_url, options) => {calls.push(options!);return new Response(gif)}
  try {
    await generatedImageItems({data:[{url:'https://cdn.example.com/test.gif'}]},1,new AbortController().signal)
    assert.equal(calls.length,1); assert.equal(calls[0]!.credentials,'omit'); assert.equal(calls[0]!.redirect,'error'); assert.equal(calls[0]!.headers,undefined)
    await assert.rejects(generatedImageItems({data:[{url:'https://user:secret@cdn.example.com/a.gif'}]},1,new AbortController().signal))
    assert.equal(calls.length,1)
  } finally {globalThis.fetch = original}
})
