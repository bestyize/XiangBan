// 验证 Codex app-server：握手、列出会话；加 --chat 时发起一轮真实对话（需要已登录 Codex）。
// 用法：node 02-codex-appserver.mjs [--chat] [--cwd <目录>]
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { append, header, save, start, verdict, which } from './lib.mjs'

const chat = process.argv.includes('--chat')
const cwdArg = process.argv.indexOf('--cwd')
const cwd = cwdArg > 0 ? process.argv[cwdArg + 1] : mkdtempSync(join(tmpdir(), 'p0-codex-'))

header('Codex app-server')
if (!which('codex')) { verdict(false, '没找到 codex 命令，先安装：npm i -g @openai/codex'); process.exit(1) }

const LOG = 'codex-appserver-messages.jsonl'
const child = start('codex', ['app-server'], { cwd })
let buf = ''
let nextId = 1
const pending = new Map()
const seenMethods = new Map()
let firstFrame = null

function send(method, params) {
  const id = nextId++
  const msg = { id, method, params }
  append(LOG, JSON.stringify({ dir: 'out', ...msg }))
  child.stdin.write(JSON.stringify(msg) + '\n')
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => { pending.delete(id); reject(new Error(`${method} 超时`)) }, 60000)
    pending.set(id, v => { clearTimeout(t); resolve(v) })
  })
}

let turnDone
const turnPromise = new Promise(r => (turnDone = r))

function handle(msg) {
  append(LOG, JSON.stringify({ dir: 'in', ...msg }))
  firstFrame ??= Object.keys(msg)
  if (msg.method) seenMethods.set(msg.method, (seenMethods.get(msg.method) ?? 0) + 1)
  if (msg.id !== undefined && !msg.method && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); return }
  // 服务端发来的请求（权限、提问）：本脚本一律拒绝，只记录。
  if (msg.id !== undefined && msg.method) {
    const decline = /requestApproval|applyPatchApproval|execCommandApproval/.test(msg.method)
      ? { decision: 'decline' }
      : {}
    child.stdin.write(JSON.stringify({ id: msg.id, result: decline }) + '\n')
  }
  if (msg.method === 'turn/completed') turnDone(msg)
}

child.stdout.on('data', d => {
  buf += d
  let i
  while ((i = buf.indexOf('\n')) >= 0) {
    const line = buf.slice(0, i).trim()
    buf = buf.slice(i + 1)
    if (!line) continue
    try { handle(JSON.parse(line)) } catch { append(LOG, JSON.stringify({ dir: 'in', _unparsed: line })) }
  }
})
let errText = ''
child.stderr.on('data', d => (errText += d))

const result = { cwd }
try {
  const init = await send('initialize', { clientInfo: { name: 'xiaoban-p0-verify', title: '小伴 P0 验证', version: '0.0.1' } })
  result.initialize = init.error ? { error: init.error } : { ok: true, keys: Object.keys(init.result ?? {}) }
  verdict(!init.error, `initialize ${init.error ? '失败：' + JSON.stringify(init.error) : '成功'}`)
  child.stdin.write(JSON.stringify({ method: 'initialized' }) + '\n')

  const list = await send('thread/list', { limit: 5 })
  result.threadList = list.error ? { error: list.error } : { ok: true, count: (list.result?.data ?? list.result?.threads ?? []).length, keys: Object.keys(list.result ?? {}) }
  verdict(!list.error, `thread/list ${list.error ? '失败：' + JSON.stringify(list.error) : '成功，键：' + Object.keys(list.result ?? {}).join(',')}`)

  if (chat) {
    const t0 = Date.now()
    const th = await send('thread/start', { cwd, approvalPolicy: 'on-request', sandbox: 'read-only' })
    const threadId = th.result?.thread?.id ?? th.result?.threadId
    result.threadStart = th.error ? { error: th.error } : { ok: true, threadId }
    verdict(!!threadId, `thread/start ${threadId ? '得到 threadId' : '失败：' + JSON.stringify(th.error ?? th.result)}`)
    if (threadId) {
      const turn = await send('turn/start', { threadId, input: [{ type: 'text', text: '只回复两个字：收到' }] })
      result.turnStart = turn.error ? { error: turn.error } : { ok: true }
      verdict(!turn.error, `turn/start ${turn.error ? '失败：' + JSON.stringify(turn.error) : '成功'}`)
      const done = await Promise.race([turnPromise, new Promise(r => setTimeout(() => r(null), 90000))])
      result.turnCompleted = !!done
      result.chatMs = Date.now() - t0
      verdict(!!done, `turn/completed ${done ? '收到，用时 ' + result.chatMs + ' ms' : '90 秒内没收到'}`)
    }
  }
} catch (e) {
  result.error = String(e)
  verdict(false, String(e))
}
result.firstFrameKeys = firstFrame
result.notificationCounts = Object.fromEntries(seenMethods)
result.stderrTail = errText.slice(-1500)
save('codex-appserver-summary.json', result)
child.kill()
setTimeout(() => process.exit(0), 300)
