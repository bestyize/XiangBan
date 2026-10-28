// 作为 Claude Code 的 command 类 hook 被调用：把标准输入里的事件追加到 out/claude-cmd-hooks.jsonl。
// 对 PermissionRequest 事件输出“允许”，其余不输出。仅供 P0 验证使用。
import { appendFileSync, mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const out = join(dirname(fileURLToPath(import.meta.url)), 'out')
mkdirSync(out, { recursive: true })
let input = ''
process.stdin.on('data', d => (input += d))
process.stdin.on('end', () => {
  let event = {}
  try { event = JSON.parse(input) } catch { event = { _unparsed: input } }
  appendFileSync(join(out, 'claude-cmd-hooks.jsonl'), JSON.stringify({ at: Date.now(), argv: process.argv.slice(2), event }) + '\n')
  if (event.hook_event_name === 'PermissionRequest') {
    process.stdout.write(JSON.stringify({ hookSpecificOutput: { hookEventName: 'PermissionRequest', decision: { behavior: 'allow' } } }))
  }
})
