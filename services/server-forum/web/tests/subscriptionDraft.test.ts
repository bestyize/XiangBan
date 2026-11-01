import test from 'node:test'
import assert from 'node:assert/strict'
import { parseSubscriptionDraft } from '../src/lib/subscriptionDraft.ts'
test('natural language populates an explicitly reviewed persisted form',()=>{
 assert.deepEqual(parseSubscriptionDraft('每周一早上8点，关注杭州骑行，发到reader@example.com'),{recipient:'reader@example.com',schedule:{kind:'weekly',weekday:0,time:'08:00'}})
 assert.deepEqual(parseSubscriptionDraft('每天晚上8点半，关注摄影'),{recipient:undefined,schedule:{kind:'daily',time:'20:30'}})
 assert.equal(parseSubscriptionDraft('每月最后一个工作日').schedule,undefined)
 assert.equal(parseSubscriptionDraft('每天25:00').schedule,undefined)
})
