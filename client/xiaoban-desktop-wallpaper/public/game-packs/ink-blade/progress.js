/* 水墨剑客 · save data: per-幅 records, unlocks and seal totals. Pure functions; the host only stores the JSON. */
(() => {
  'use strict'

  const LV = globalThis.XiabanInkLevels
  const VERSION = 1

  function empty() { return { v: VERSION, volume: 1, levels: {}, prompts: [], difficulty: 'normal' } }

  const int = (value, max) => (Number.isInteger(value) && value >= 0 && value <= max ? value : 0)
  const bits = (mask) => (mask & 1) + ((mask >> 1) & 1) + ((mask >> 2) & 1)

  /** Saves come back from the host or local storage, so every field is re-validated and unknown keys dropped. */
  function normalize(raw) {
    const save = empty()
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return save
    if (raw.difficulty === 'easy') save.difficulty = 'easy'
    if (raw.volume === 2) save.volume = 2
    if (Array.isArray(raw.prompts)) {
      save.prompts = [...new Set(raw.prompts.filter((key) => typeof key === 'string' && /^[a-z]{1,16}$/.test(key)))].slice(0, 32)
    }
    const levels = raw.levels && typeof raw.levels === 'object' && !Array.isArray(raw.levels) ? raw.levels : {}
    for (const level of LV.LEVELS) {
      const entry = levels[level.id]
      if (!entry || typeof entry !== 'object') continue
      const clears = int(entry.clears, 1e6)
      if (!clears) continue
      save.levels[level.id] = { clears, best: int(entry.best, 1e9), grade: Math.max(1, int(entry.grade, 4)), seals: int(entry.seals, 7) | 1, time: int(entry.time, 1e5) }
    }
    return save
  }

  /** Folds a finished round into the save. Only clears count; returns what changed for the result screen. */
  function record(save, result) {
    const previous = save.levels[result.level] || null
    const outcome = { save, previous, gained: 0, firstClear: false, newBest: false, newGrade: false }
    if (!result.cleared || !LV.level(result.level)) return outcome
    const entry = previous ? { ...previous } : { clears: 0, best: 0, grade: 0, seals: 0, time: 0 }
    outcome.firstClear = entry.clears === 0
    outcome.gained = result.seals & ~entry.seals
    outcome.newBest = result.score > entry.best
    outcome.newGrade = result.grade > entry.grade
    entry.clears += 1
    entry.best = Math.max(entry.best, result.score)
    entry.grade = Math.max(entry.grade, result.grade)
    entry.seals |= result.seals
    entry.time = entry.time ? Math.min(entry.time, result.seconds) : result.seconds
    outcome.save = { ...save, levels: { ...save.levels, [result.level]: entry } }
    return outcome
  }

  function cleared(save, id) { return Boolean(save.levels[id]?.clears) }

  /** The first 幅 is always open; each later one opens once the one before it is cleared; 长卷 needs the 终幅. */
  function unlocked(save, id) {
    const level = LV.level(id)
    if (!level) return false
    const stages = LV.volumeLevels(level.volume).filter((entry) => !entry.scroll)
    if (level.scroll) return cleared(save, stages[stages.length - 1].id) || cleared(save, id)
    const index = stages.indexOf(level)
    return index === 0 || cleared(save, stages[index - 1].id) || cleared(save, id)
  }

  function next(id) {
    const level = LV.level(id)
    if (!level || level.scroll) return null
    const stages = LV.volumeLevels(level.volume).filter((entry) => !entry.scroll)
    return stages[stages.indexOf(level) + 1]?.id || null
  }

  function sealCount(save, volume) {
    return LV.volumeLevels(volume).reduce((sum, level) => sum + bits(save.levels[level.id]?.seals || 0), 0)
  }

  function maxSeals(volume) { return LV.volumeLevels(volume).length * 3 }

  function volumeUnlocked(save, volumeId) {
    if (volumeId === 1) return true
    if (volumeId === 2) return true
    return false
  }

  function rememberPrompt(save, key) {
    return save.prompts.includes(key) ? save : { ...save, prompts: [...save.prompts, key].slice(0, 32) }
  }

  globalThis.XiabanInkProgress = { VERSION, empty, normalize, record, cleared, unlocked, next, sealCount, maxSeals, volumeUnlocked, rememberPrompt, bits }
})()
