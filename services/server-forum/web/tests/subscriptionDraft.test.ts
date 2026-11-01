import test from 'node:test'
import assert from 'node:assert/strict'
import { parseSubscriptionDraft } from '../src/lib/subscriptionDraft.ts'
test('natural language populates an explicitly reviewed persisted form',()=>{
 assert.deepEqual(parseSubscriptionDraft('每周一早上8点，关注杭州骑行，发到reader@example.com'),{recipient:'reader@example.com',schedule:{kind:'weekly',weekday:0,time:'08:00'}})
 assert.deepEqual(parseSubscriptionDraft('每天晚上8点半，关注摄影'),{recipient:undefined,schedule:{kind:'daily',time:'20:30'}})
 assert.equal(parseSubscriptionDraft('每月最后一个工作日').schedule,undefined)
 assert.equal(parseSubscriptionDraft('每天25:00').schedule,undefined)
})

import { subscriptionInput, newSubscriptionDraft } from '../src/lib/subscriptionDraft.ts'
const listResponse = {
 items: [{id:'4d9962bb-7572-412b-b15a-a2643f1eae68',owner:'a13eb3d7-7e8d-45d5-8289-d566f9c64e04',revision:2,
  title:'城市骑行',interests:'杭州骑行与周末活动',recipient:'reader@example.test',
  schedule:{kind:'weekly' as const,time:'08:00',weekday:0},timezone:'Asia/Shanghai',
  next_run:'2026-11-02T00:00:00Z',status:'active',verified_at:'2026-11-01T18:00:00Z',
  created_at:'2026-11-01T17:00:00Z',updated_at:'2026-11-01T18:00:00Z'}],
 delivery_configured:false,worker_enabled:false,
}
const inputKeys=['id','revision','title','interests','recipient','schedule','timezone','active','confirmed'].sort()
function assertWriteContract(value:unknown){assert.deepEqual(Object.keys(JSON.parse(JSON.stringify(value))).sort(),inputKeys)}
test('edit and save project the full list response without mutating it',()=>{
 const row=listResponse.items[0], draft=subscriptionInput(row,row.status==='active',false)
 draft.title='修改主题';draft.schedule.kind==='weekly'&&(draft.schedule.time='09:30')
 const payload=subscriptionInput(draft,draft.active,true)
 assertWriteContract(payload);assert.equal(payload.revision,2);assert.equal(payload.title,'修改主题');assert.equal(payload.confirmed,true)
 assert.equal(row.title,'城市骑行');assert.equal(row.schedule.time,'08:00')
})
test('pause and resume omit every server-only field',()=>{
 const row=listResponse.items[0]
 const paused=subscriptionInput(row,row.status!=='active',true)
 assertWriteContract(paused);assert.equal(paused.active,false)
 const resumed=subscriptionInput({...row,revision:3},true,true)
 assertWriteContract(resumed);assert.equal(resumed.active,true);assert.equal(resumed.revision,3)
})
test('new after edit replaces the entire draft and uses a fresh POST identity',()=>{
 let draft=subscriptionInput(listResponse.items[0],true,false)
 Object.assign(draft,{owner:listResponse.items[0].owner,status:'active'})
 draft=newSubscriptionDraft()
 assertWriteContract(draft);assert.equal(draft.revision,0);assert.notEqual(draft.id,listResponse.items[0].id)
 assert.equal(draft.title,'');assert.equal(draft.recipient,'');assert.equal(draft.confirmed,false)
 assertWriteContract(subscriptionInput(draft,true,true))
})
