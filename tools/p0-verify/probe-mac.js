// macOS 探测（JXA）：目标应用是否可通过辅助功能接口看到输入框，以及“激活并粘贴”要多久。
// 用法（由 05-desktop-probe.mjs 调用）：osascript -l JavaScript probe-mac.js <应用名> <tree|paste>
// 没在真机上跑过；出错请把完整报错原样带回来。需要给“终端”（或运行它的程序）开辅助功能权限。
ObjC.import('ApplicationServices')
ObjC.import('AppKit')
ObjC.import('Foundation')

function now() { return Date.now() }

function run(argv) {
  const name = argv[0]
  const mode = argv[1] || 'tree'
  const se = Application('System Events')
  const found = se.processes.whose({ name: name })()
  if (found.length === 0) return JSON.stringify({ ok: false, error: `没有名为 ${name} 的运行中应用（先打开它，并打开一个对话页）` })
  const proc = found[0]
  const pid = proc.unixId()
  const out = { ok: true, name, pid }

  // Electron 应用默认不开无障碍树，写这个属性可以让它构建（等价于有辅助技术接入）。
  const appEl = $.AXUIElementCreateApplication(pid)
  const t0 = now()
  out.manualAccessibilitySet = $.AXUIElementSetAttributeValue(appEl, $('AXManualAccessibility'), $.kCFBooleanTrue) === 0
  delay(1.0)

  if (mode === 'tree') {
    const candidates = []
    let visited = 0
    function walk(el, depth) {
      if (depth > 9 || visited > 4000) return
      visited++
      let role = '', sub = '', desc = '', focused = false
      try { role = el.role() } catch (e) {}
      try { sub = el.subrole() } catch (e) {}
      try { desc = String(el.description() || el.title() || '').slice(0, 40) } catch (e) {}
      try { focused = !!el.focused() } catch (e) {}
      if (role === 'AXTextArea' || role === 'AXTextField' || role === 'AXComboBox' || (role === 'AXGroup' && sub === 'AXApplicationDialog')) {
        candidates.push({ role, sub, desc, focused, depth })
      }
      let kids = []
      try { kids = el.uiElements() } catch (e) {}
      for (const k of kids) walk(k, depth + 1)
    }
    let windowCount = 0
    try { windowCount = proc.windows().length } catch (e) {}
    out.windowCount = windowCount
    if (windowCount > 0) walk(proc.windows[0], 0)
    out.visited = visited
    out.candidates = candidates.slice(0, 20)
    out.candidateCount = candidates.length
    out.ms = now() - t0
    return JSON.stringify(out)
  }

  // paste：保存剪贴板 → 写入测试文字 → 激活应用 → ⌘V → 恢复剪贴板。不按回车。
  const pb = $.NSPasteboard.generalPasteboard
  const old = pb.stringForType($.NSPasteboardTypeString)
  const text = 'P0-粘贴测试 ' + new Date().toISOString()
  pb.clearContents
  pb.setStringForType($(text), $.NSPasteboardTypeString)
  const a0 = now()
  proc.frontmost = true
  delay(0.15)
  se.keystroke('v', { using: 'command down' })
  out.activateAndPasteMs = now() - a0
  delay(0.3)
  pb.clearContents
  if (old && !old.isNil()) pb.setStringForType(old, $.NSPasteboardTypeString)
  out.pastedText = text
  out.note = '请肉眼确认：这行文字是否出现在对方应用的输入框里（没有被发送）'
  return JSON.stringify(out)
}
