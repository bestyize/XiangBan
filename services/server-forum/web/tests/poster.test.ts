import assert from 'node:assert/strict'
import test from 'node:test'
import { qrMatrix, wrapText } from '../src/lib/poster.ts'

const measure = (s: string) => Array.from(s).length * 10

test('wraps by width and ends a truncated block with an ellipsis', () => {
  assert.deepEqual(wrapText(measure, '短标题', 100, 2), ['短标题'])
  assert.deepEqual(wrapText(measure, '一二三四五六七八九十甲乙', 50, 3), ['一二三四五', '六七八九十', '甲乙'])
  const cut = wrapText(measure, '一二三四五六七八九十甲乙丙丁', 50, 2)
  assert.equal(cut.length, 2)
  assert.equal(cut[0], '一二三四五')
  assert.ok(cut[1]!.endsWith('…'))
  assert.ok(measure(cut[1]!) <= 50, '加上省略号后也不超宽')
  assert.deepEqual(wrapText(measure, '   ', 50, 2), [])
  assert.deepEqual(wrapText(measure, '恰好五个字符', 60, 1), ['恰好五个字符'])
})

test('qr matrix is square, with the three finder patterns', () => {
  const m = qrMatrix('https://chat.xiangban.xyz/packs/12')
  assert.ok(m.length >= 21 && m.length === m[0]!.length)
  // 三个角上的定位图案：7×7 外框全黑
  const n = m.length
  for (const [r0, c0] of [[0, 0], [0, n - 7], [n - 7, 0]] as const) {
    for (let i = 0; i < 7; i++) {
      assert.ok(m[r0]![c0 + i] && m[r0 + 6]![c0 + i] && m[r0 + i]![c0] && m[r0 + i]![c0 + 6])
    }
  }
})
