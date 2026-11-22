import assert from 'node:assert/strict'
import { test } from 'node:test'
import { readTopicDraft, topicDraftKey, writeTopicDraft, type TopicDraft } from '../src/lib/topicDraft.ts'

function localDisk() {
  const data = new Map<string, string>()
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, raw: string) => { data.set(key, raw) },
    removeItem: (key: string) => { data.delete(key) },
  }
}

const draft: TopicDraft = {
  form: { sectionKey: 'discuss', boardKey: 'casual-fun', title: '草稿', body: '重新打开后继续写。', tags: ['标签'], readLevel: 1 },
  tagInput: '正在输入',
  attachments: [{ id: 'attachment', name: '附件', size: 2, price: 3, purchased: false }],
  work: {
    source: 'uploaded', key: 'files/work.zip', kind: 'game', title: '作品', summary: '介绍',
    originalName: 'work.zip', format: 'zip', size: 12, rightsAccepted: true,
  },
  poll: { options: ['第一项', '第二项'], days: 7 },
  requestId: 'f62a574a-7c95-4c2d-b177-b9f3b2ba956a',
}

test('a refreshed composer recovers the same draft and upload receipt only in its account', () => {
  const disk = localDisk()
  const key = topicDraftKey('alice')
  writeTopicDraft(key, draft, disk)
  assert.deepEqual(readTopicDraft(topicDraftKey('alice'), disk), draft)
  assert.equal(readTopicDraft(topicDraftKey('bob'), disk), null)
  assert.equal(readTopicDraft(topicDraftKey('alice', 21), disk), null)

  const linked: TopicDraft = { ...draft, work: { source: 'mine', id: 7, title: '作品', kind: 'game', version: 1 } }
  writeTopicDraft(key, linked, disk)
  assert.deepEqual(readTopicDraft(key, disk), linked)
  writeTopicDraft(topicDraftKey('bob'), draft, disk)
  writeTopicDraft(key, null, disk)
  assert.equal(readTopicDraft(key, disk), null)
  assert.deepEqual(readTopicDraft(topicDraftKey('bob'), disk), draft)
})

test('an invalid saved draft leaves the composer available', () => {
  const disk = localDisk()
  const key = topicDraftKey('alice')
  disk.setItem(key, '{broken')
  assert.equal(readTopicDraft(key, disk), null)
})
