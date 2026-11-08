import { test } from 'node:test'
import { strict as assert } from 'node:assert'
import { checkedMbtiResultLink, decodeMbtiPng } from '../src/lib/mbtiExport.ts'

const link='https://chat.xiangban.xyz/builtin-games/mbti/share.html?t=INFJ&p=25%2C25%2C25%2C75&n=12%2C12%2C12%2C12&d=2026-11-08T13%3A46%3A00.000Z'
test('only the intended public result snapshot may be passed to the system share panel',()=>{
  assert.equal(checkedMbtiResultLink(link),link)
  for(const bad of [link.replace('chat.xiangban.xyz','chat.xiangban.xyz.evil.example'),link.replace('https:','http:'),link+'&answers=private',link+'&t=ESTJ',link+'#private',link.replace('25%2C25%2C25%2C75','75%2C25%2C25%2C75'),link.replace('12%2C12%2C12%2C12','12%2C0%2C12%2C12')])assert.equal(checkedMbtiResultLink(bad),null,bad)
})
test('image messages must be bounded PNG data',()=>{
  const header=Buffer.from([137,80,78,71,13,10,26,10,0,0]).toString('base64')
  assert.deepEqual(decodeMbtiPng('data:image/png;base64,'+header),new Uint8Array([137,80,78,71,13,10,26,10,0,0]))
  for(const bad of ['data:image/jpeg;base64,'+header,'data:image/png;base64,////','data:image/png;base64,'+'A'.repeat(2_800_000),null])assert.equal(decodeMbtiPng(bad),null)
})
