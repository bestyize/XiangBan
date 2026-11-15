const starryScorePrefix = 'starry-score:'
const starryMemory = new Map()
const starryStorage = (parent !== window || window.xb.info.platform !== 'web') ? window.xb.storage : {
  keys() {
    try { return [...new Set([...Object.keys(localStorage), ...starryMemory.keys()])] } catch { return [...starryMemory.keys()] }
  },
  get(key) {
    if (starryMemory.has(key)) return starryMemory.get(key)
    try { return JSON.parse(localStorage.getItem(key)) } catch { return null }
  },
  set(key, value) {
    try {
      localStorage.setItem(key, JSON.stringify(value))
      starryMemory.delete(key)
    } catch { starryMemory.set(key, value) }
  },
}

async function starryBoardRows() {
  const keys = await starryStorage.keys()
  const records = await Promise.all(keys.filter(key => key.startsWith(starryScorePrefix)).map(key => starryStorage.get(key)))
  const best = new Map()
  for (const row of records) {
    if (!row || typeof row.handle !== 'string' || !Number.isSafeInteger(row.score)) continue
    const key = row.handle.toLowerCase()
    if (!best.has(key) || row.score > best.get(key).score) best.set(key, row)
  }
  return [...best.values()].sort((a, b) => b.score - a.score || a.handle.localeCompare(b.handle))
}

function starryReply(body, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })
}

async function starryLocalBoard(_url, options) {
  if (options.method !== 'POST') {
    const rows = await starryBoardRows()
    return starryReply({ top: rows.slice(0, 25), count: rows.length })
  }
  const { action, handle, score, token } = JSON.parse(options.body)
  if (action === 'start') {
    return starryReply({ ok: true, token: crypto.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}` })
  }
  if (typeof handle !== 'string' || !/^[\u4e00-\u9fa5A-Za-z0-9_]{1,12}$/.test(handle) || !Number.isSafeInteger(score) || score < 0 ||
      typeof token !== 'string' || !/^[a-z0-9-]{1,64}$/.test(token)) {
    return starryReply({ ok: false, message: '名号或分数无效。' }, 400)
  }
  // Each ride has its own key so recording another ride cannot replace this one.
  const key = starryScorePrefix + token
  const previous = await starryStorage.get(key)
  await starryStorage.set(key, { handle, score: Math.max(previous?.score ?? 0, score) })
  const rows = await starryBoardRows()
  const rank = rows.findIndex(row => row.handle.toLowerCase() === handle.toLowerCase()) + 1
  return starryReply({ ok: true, handle, score, best: rows[rank - 1].score, rank, top: rows.slice(0, 25), count: rows.length })
}
