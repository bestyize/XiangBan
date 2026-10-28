// 验证“hooks 注入”：Stop hook 返回 {decision:"block", reason:"…"} 后，会话是否带着这条新指令继续做事。
// 这是语音指令在不改变启动方式的前提下进入正在运行的会话的一种途径（参考 mcp-voice-hooks 的做法）。
// 需要已登录 Claude Code。用法：node 06-claude-stop-inject.mjs
import http from 'node:http'
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { header, jsonLines, run, save, verdict, which } from './lib.mjs'

header('Claude Code：Stop hook 注入新指令')
if (!which('claude')) { verdict(false, '没找到 claude 命令'); process.exit(1) }

const dir = mkdtempSync(join(tmpdir(), 'p0-inject-'))
mkdirSync(join(dir, '.claude'))
let stops = 0
const seen = []
const server = http.createServer((req, res) => {
  let body = ''
  req.on('data', d => (body += d))
  req.on('end', () => {
    const event = JSON.parse(body || '{}')
    seen.push({ name: event.hook_event_name, stop_hook_active: event.stop_hook_active, last: event.last_assistant_message })
    res.setHeader('content-type', 'application/json')
    if (event.hook_event_name === 'Stop' && ++stops === 1) {
      // 第一次想结束时，塞进一条“语音指令”；第二次放行，避免死循环。
      res.end(JSON.stringify({ decision: 'block', reason: '用户刚才又用语音补充了一条指令：请用 Bash 工具执行 touch p0-injected.txt ，然后只回复 全部完成' }))
    } else res.end('{}')
  })
})
await new Promise(r => server.listen(0, '127.0.0.1', r))
const port = server.address().port
writeFileSync(join(dir, '.claude', 'settings.local.json'), JSON.stringify({
  hooks: { Stop: [{ hooks: [{ type: 'http', url: `http://127.0.0.1:${port}/hook/Stop`, timeout: 30 }] }] },
  permissions: { allow: ['Bash(touch p0-injected.txt)'] },
}, null, 2))

const r = await run('claude', ['-p', '只回复两个字：好的', '--output-format', 'stream-json', '--verbose'], { cwd: dir, timeoutMs: 180000 })
server.close()
const events = jsonLines(r.stdout)
const injectedFile = existsSync(join(dir, 'p0-injected.txt'))
const results = events.filter(e => e.type === 'result')
const summary = {
  exitCode: r.code, ms: r.ms, stopEventsSeen: seen.filter(s => s.name === 'Stop'), resultCount: results.length,
  lastResult: results.at(-1)?.result ?? null, injectedFileCreated: injectedFile,
}
verdict(r.code === 0, `命令退出码 ${r.code}`)
verdict(stops >= 2, `Stop 事件触发 ${stops} 次（第一次被我们拦下并注入指令）`)
verdict(injectedFile, '会话带着注入的指令继续做了事（出现了 p0-injected.txt）')
console.log('   最终回复：', summary.lastResult)
save('claude-stop-inject-summary.json', summary)
rmSync(dir, { recursive: true, force: true })
