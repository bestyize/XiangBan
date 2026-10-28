// 验证 Claude Code 的 hooks：http 类型与 command 类型各跑一次真实对话（需要已登录 Claude Code）。
// 只在临时项目目录里写 .claude/settings.local.json，不碰用户全局设置。
// 用法：node 03-claude-hooks.mjs [http|command|both]
import http from 'node:http'
import { mkdirSync, mkdtempSync, writeFileSync, existsSync, readFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { append, header, jsonLines, run, save, verdict, which } from './lib.mjs'

const mode = process.argv[2] ?? 'both'
const here = dirname(fileURLToPath(import.meta.url))
const EVENTS = ['SessionStart', 'UserPromptSubmit', 'PreToolUse', 'PermissionRequest', 'PostToolUse', 'Notification', 'Stop', 'SessionEnd']
// 用会创建文件的命令，保证需要权限确认（只读命令如 echo 会被自动放行，不触发 PermissionRequest）。
const PROMPT = '用 Bash 工具执行命令 touch p0-hook-test.txt ，然后只回复两个字：完成'

header('Claude Code hooks')
if (!which('claude')) { verdict(false, '没找到 claude 命令'); process.exit(1) }

async function oneRun(kind) {
  const dir = mkdtempSync(join(tmpdir(), `p0-claude-${kind}-`))
  mkdirSync(join(dir, '.claude'))
  const received = []
  let server = null
  let hooks = {}
  if (kind === 'http') {
    server = http.createServer((req, res) => {
      let body = ''
      req.on('data', d => (body += d))
      req.on('end', () => {
        let event = {}
        try { event = JSON.parse(body) } catch { event = { _unparsed: body } }
        received.push({ at: Date.now(), path: req.url, event })
        append('claude-http-hooks.jsonl', JSON.stringify({ at: Date.now(), path: req.url, event }))
        res.setHeader('content-type', 'application/json')
        if (event.hook_event_name === 'PermissionRequest') {
          res.end(JSON.stringify({ hookSpecificOutput: { hookEventName: 'PermissionRequest', decision: { behavior: 'allow' } } }))
        } else res.end('{}')
      })
    })
    await new Promise(r => server.listen(0, '127.0.0.1', r))
    const port = server.address().port
    for (const e of EVENTS) hooks[e] = [{ hooks: [{ type: 'http', url: `http://127.0.0.1:${port}/hook/${e}`, timeout: 30 }] }]
  } else {
    for (const e of EVENTS) hooks[e] = [{ hooks: [{ type: 'command', command: process.execPath, args: [join(here, 'hook-cmd.mjs'), e], timeout: 30 }] }]
  }
  writeFileSync(join(dir, '.claude', 'settings.local.json'), JSON.stringify({ hooks }, null, 2))
  const cmdLog = join(here, 'out', 'claude-cmd-hooks.jsonl')
  if (kind === 'command' && existsSync(cmdLog)) rmSync(cmdLog)

  console.log(`\n--- ${kind} 类型 hook，工作目录 ${dir}`)
  const r = await run('claude', ['-p', PROMPT, '--output-format', 'stream-json', '--verbose'], { cwd: dir, timeoutMs: 180000 })
  save(`claude-stream-${kind}.jsonl`, r.stdout)
  const events = jsonLines(r.stdout)
  const toolRan = existsSync(join(dir, 'p0-hook-test.txt'))
  const got = kind === 'http' ? received.map(x => x.event.hook_event_name) : (existsSync(cmdLog) ? jsonLines(readFileSync(cmdLog, 'utf8')).map(x => x.event.hook_event_name) : [])
  server?.close()
  const summary = {
    kind, exitCode: r.code, ms: r.ms, stderrTail: r.stderr.slice(-800),
    hookEventsReceived: [...new Set(got)],
    hookEventCounts: got.reduce((m, e) => ((m[e] = (m[e] ?? 0) + 1), m), {}),
    streamEventTypes: [...new Set(events.map(e => `${e.type}${e.subtype ? '/' + e.subtype : ''}`))],
    toolRanAfterHookAllow: toolRan,
    sessionId: events.find(e => e.session_id)?.session_id ?? null,
    transcriptPathSeen: (kind === 'http' ? received.map(x => x.event.transcript_path) : []).find(Boolean) ?? null,
    permissionDenials: events.find(e => e.type === 'result')?.permission_denials ?? null,
  }
  verdict(summary.exitCode === 0, `${kind}：命令退出码 ${r.code}，用时 ${r.ms} ms`)
  verdict(summary.hookEventsReceived.includes('Stop'), `${kind}：收到 Stop 事件`)
  verdict(summary.hookEventsReceived.includes('PermissionRequest'), `${kind}：收到 PermissionRequest 事件`)
  verdict(toolRan, `${kind}：hook 返回“允许”后工具真的执行了（临时目录里出现 p0-hook-test.txt）`)
  console.log('   收到的事件：', summary.hookEventCounts)
  rmSync(dir, { recursive: true, force: true })
  return summary
}

const results = []
if (mode === 'http' || mode === 'both') results.push(await oneRun('http'))
if (mode === 'command' || mode === 'both') results.push(await oneRun('command'))
save('claude-hooks-summary.json', results)
