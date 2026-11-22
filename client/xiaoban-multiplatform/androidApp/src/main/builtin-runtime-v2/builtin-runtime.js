(() => {
  'use strict'
  const xb = globalThis.xb
  if (!xb) return

  const saveKey = 'save'
  const root = document.documentElement
  root.classList.add('xb-builtin-runtime')
  const reportError = error => console.error('小伴内置作品：', error)

  function layout() {
    return { safeArea: { ...xb.info.safeArea }, capsule: { ...xb.info.capsule } }
  }

  function updateSession(payload) {
    try {
      xb.game?.session?.(payload)?.catch?.(() => {})
    } catch (_) {}
  }

  /* ═══════════════════════════════════════════════════════════════════════════
     1. 方块爆破 (block-puzzle) · 无大厅单页直开型：
        - 开局自动恢复上次未完残局棋盘 + 顶部浮出「已自动恢复上次残局 · [新开一局]」胶囊
        - 顶栏与底栏去重，右上角唯一胶囊联动方案 A「暂停 + 存档 + 宿主」合一面板
     ═══════════════════════════════════════════════════════════════════════════ */
  function setupBlockPuzzle() {
    const topbar = document.querySelector('.topbar')
    const boardStage = document.querySelector('.board-stage')
    const scoreEl = document.getElementById('score')
    const bestEl = document.getElementById('best')
    const overlayEl = document.getElementById('overlay')
    const continueBtn = document.getElementById('continueButton')
    const restartBtn = document.getElementById('restartButton')
    const soundBtn = document.getElementById('soundButton')
    const musicBtn = document.getElementById('musicButton')
    if (!topbar || !scoreEl || !continueBtn || !restartBtn) return

    // 顶栏右侧（宿主胶囊旁）展示实时保存状态丸
    if (!topbar.querySelector('.xb-bp-save-pill')) {
      const pill = document.createElement('span')
      pill.className = 'xb-bp-save-pill'
      pill.innerHTML = '<i></i><span>进度实时保存</span>'
      topbar.appendChild(pill)
    }

    let resumeBanner = null
    let restoredScore = 0
    let autoRestored = false

    function removeResumeBanner() {
      if (!resumeBanner) return
      const node = resumeBanner
      resumeBanner = null
      node.classList.add('hiding')
      setTimeout(() => node.remove(), 240)
    }

    function showResumeBanner(score) {
      removeResumeBanner()
      const banner = document.createElement('div')
      banner.className = 'xb-bp-resume-banner'
      banner.innerHTML = `
        <span><i class="xb-dot"></i>已自动恢复上次残局（${score.toLocaleString('zh-CN')} 分）</span>
        <button type="button">新开一局</button>
      `
      banner.querySelector('button').addEventListener('click', () => {
        removeResumeBanner()
        restartBtn.click()
        syncBlockPuzzleSession()
      })
      if (boardStage && boardStage.parentNode) {
        boardStage.parentNode.insertBefore(banner, boardStage)
      }
      resumeBanner = banner
      setTimeout(() => {
        if (resumeBanner === banner) removeResumeBanner()
      }, 4500)
    }

    function parseNum(el) {
      return Number((el?.textContent || '0').replace(/[^0-9]/g, '')) || 0
    }

    function syncBlockPuzzleSession() {
      const score = parseNum(scoreEl)
      const best = Math.max(score, parseNum(bestEl))
      const over = overlayEl && !overlayEl.hidden
      const soundOn = soundBtn?.getAttribute('aria-pressed') !== 'false'
      const musicOn = musicBtn?.getAttribute('aria-pressed') !== 'false'

      if (resumeBanner && score !== restoredScore && score > 0) {
        removeResumeBanner()
      }

      updateSession({
        inRound: !over,
        title: `当前对局 · ${score.toLocaleString('zh-CN')} 分`,
        subtitle: `历史最高 ${best.toLocaleString('zh-CN')} 分 · 退出后下次打开自动接着玩`,
        saveBadge: '本局已存盘',
        exitHint: score > 0 ? `保留 ${score.toLocaleString('zh-CN')} 分残局` : '',
        resumeLabel: '继续游戏',
        restartLabel: '重开本局',
        confirmRestart: score > 0,
        canSelectStage: false,
        soundEnabled: soundOn,
        musicEnabled: musicOn,
      })
    }

    let autoRestoreScheduled = false
    function tryAutoRestore() {
      if (autoRestored || autoRestoreScheduled || continueBtn.hidden) return
      autoRestoreScheduled = true
      setTimeout(() => {
        autoRestoreScheduled = false
        if (autoRestored || continueBtn.hidden) return
        autoRestored = true
        continueBtn.click()
        restoredScore = parseNum(scoreEl)
        if (restoredScore > 0) {
          showResumeBanner(restoredScore)
        }
        syncBlockPuzzleSession()
      }, 45)
    }

    // 监听 restoreFromHost 完成后解锁 #continueButton 的瞬间，自动恢复残局棋盘
    const observer = new MutationObserver(() => {
      tryAutoRestore()
      syncBlockPuzzleSession()
    })
    observer.observe(continueBtn, { attributes: true, attributeFilter: ['hidden'] })
    observer.observe(scoreEl, { childList: true, characterData: true, subtree: true })
    if (bestEl) observer.observe(bestEl, { childList: true, characterData: true, subtree: true })
    if (overlayEl) observer.observe(overlayEl, { attributes: true, attributeFilter: ['hidden'] })
    if (soundBtn) observer.observe(soundBtn, { attributes: true, attributeFilter: ['aria-pressed'] })
    if (musicBtn) observer.observe(musicBtn, { attributes: true, attributeFilter: ['aria-pressed'] })

    tryAutoRestore()
    syncBlockPuzzleSession()

    xb.on('game.action', data => {
      const action = data?.action
      if (action === 'restart-round') {
        removeResumeBanner()
        restartBtn.click()
        syncBlockPuzzleSession()
      } else if (action === 'toggle-sound') {
        soundBtn?.click()
        syncBlockPuzzleSession()
      } else if (action === 'toggle-music') {
        musicBtn?.click()
        syncBlockPuzzleSession()
      }
    })
  }

  /* ═══════════════════════════════════════════════════════════════════════════
     2. 水墨剑客 (ink-blade) · 有大厅多层闯关型：
        - 第0层大厅智能续玩「继续 · 某幅」+「画卷选关」双按钮，记住上次停留的卷幅
        - 第1层选关页保留左上角 ‹ 返回卷首，并在对应画幅标出「上次停在此幅」
        - 第2层局内战斗统一由右上角方案 A 面板提供「继续挥剑 / 重开本幅 / 返回画卷 / 音效」
     ═══════════════════════════════════════════════════════════════════════════ */
  function setupInkBlade() {
    const welcomeEl = document.getElementById('welcome')
    const selectEl = document.getElementById('selectOverlay')
    const pauseEl = document.getElementById('pauseOverlay')
    const resultEl = document.getElementById('resultOverlay')
    const startBtn = document.getElementById('startButton')
    const startLabel = document.getElementById('startLabel')
    const resumeBtn = document.getElementById('resumeButton')
    const restartBtn = document.getElementById('restartButton')
    const mapBtn = document.getElementById('mapButton')
    const nextBtn = document.getElementById('nextButton')
    const soundBtn = document.getElementById('soundButton')
    const levelStrip = document.getElementById('levelStrip')
    if (!welcomeEl || !selectEl || !startBtn) return

    root.classList.add('xb-ink-blade')
    const STAGE_KEY = 'ink_blade_active_stage_v1'
    const LV = globalThis.XiabanInkLevels
    const PROGRESS = globalThis.XiabanInkProgress

    let activeStage = null // { levelId, volume, volName, sub, name }
    let hasHistoryProgress = false
    let bypassingContinue = false

    function describeLevel(levelId) {
      if (!LV || !levelId) return null
      const lvl = LV.level(levelId)
      if (!lvl) return null
      const vol = LV.volume(lvl.volume)
      return {
        levelId: lvl.id,
        volume: lvl.volume,
        volName: vol ? vol.name : `卷${lvl.volume}`,
        sub: lvl.scroll ? '长卷' : lvl.sub,
        name: lvl.name,
      }
    }

    function saveActiveStage(levelId) {
      const info = describeLevel(levelId)
      if (!info) return
      activeStage = info
      hasHistoryProgress = true
      xb.storage.set(STAGE_KEY, info).catch(() => {})
      refreshWelcomeContinueUI()
      decorateSelectCards()
      syncInkBladeSession()
    }

    function deriveStageFromSave(rawSave) {
      if (!LV || !PROGRESS || !rawSave || typeof rawSave !== 'object') return null
      const norm = PROGRESS.normalize(rawSave)
      const clearedIds = Object.keys(norm.levels || {}).filter(id => norm.levels[id]?.clears > 0)
      if (clearedIds.length === 0 && (!norm.volume || norm.volume === 1)) return null
      hasHistoryProgress = true
      const volId = norm.volume || 1
      const stages = LV.volumeLevels(volId) || []
      const unfinished = stages.find(s => PROGRESS.unlocked(norm, s.id) && !PROGRESS.cleared(norm, s.id))
      const target = unfinished || stages.filter(s => PROGRESS.unlocked(norm, s.id)).pop() || stages[0]
      return target ? describeLevel(target.id) : null
    }

    function refreshWelcomeContinueUI() {
      const card = welcomeEl.querySelector('.welcome-card')
      const actions = welcomeEl.querySelector('.welcome-actions')
      if (!card || !actions) return

      let banner = card.querySelector('.xb-ink-save-banner')
      let selectBtn = document.getElementById('xbInkSelectBtn')

      if (activeStage && hasHistoryProgress) {
        if (!banner) {
          banner = document.createElement('div')
          banner.className = 'xb-ink-save-banner'
          card.insertBefore(banner, actions)
        }
        banner.innerHTML = `<span class="xb-ink-seal">续</span><span>上次停在 <b>${activeStage.volName} · ${activeStage.sub}「${activeStage.name}」</b></span>`
        if (startLabel) startLabel.textContent = `继续 · ${activeStage.name}`

        if (!selectBtn) {
          selectBtn = document.createElement('button')
          selectBtn.type = 'button'
          selectBtn.id = 'xbInkSelectBtn'
          selectBtn.className = 'xb-ink-select-btn'
          selectBtn.textContent = '画卷选关'
          selectBtn.addEventListener('click', () => {
            bypassingContinue = true
            startBtn.click()
            bypassingContinue = false
          })
          actions.insertBefore(selectBtn, actions.children[1] || null)
        }
      } else {
        banner?.remove()
        selectBtn?.remove()
      }
    }

    function decorateSelectCards() {
      if (!levelStrip || !activeStage) return
      levelStrip.querySelectorAll('.xb-ink-stage-tag').forEach(n => n.remove())
      const card = levelStrip.querySelector(`.level-card[data-level="${activeStage.levelId}"]`)
      if (card && !card.classList.contains('locked')) {
        const tag = document.createElement('span')
        tag.className = 'xb-ink-stage-tag'
        tag.textContent = '上次停在此幅'
        card.appendChild(tag)
      }
    }

    // 拦截卷首大厅主按钮：当有存档进度时，点「继续 · 某幅」直接进入该幅战斗
    startBtn.addEventListener('click', event => {
      if (bypassingContinue || !activeStage || !hasHistoryProgress) return
      const duelBanner = document.getElementById('duelBanner')
      if (duelBanner && !duelBanner.hidden) return
      event.stopImmediatePropagation()
      event.preventDefault()

      // 先调用原生入卷展开选关列表，再在同一帧选中对应卷与画幅直接入局
      bypassingContinue = true
      startBtn.click()
      bypassingContinue = false

      const volTab = document.getElementById('volTab' + activeStage.volume)
      if (volTab && !volTab.classList.contains('active')) volTab.click()
      const targetCard = levelStrip?.querySelector(`.level-card[data-level="${activeStage.levelId}"]:not(:disabled)`)
      if (targetCard) {
        targetCard.click()
      }
    }, true)

    // 监听玩家在选关页点击任意画幅，或结算页点「下一幅」
    levelStrip?.addEventListener('click', event => {
      const card = event.target.closest('.level-card[data-level]')
      if (card && !card.disabled && card.dataset.level) {
        saveActiveStage(card.dataset.level)
      }
    }, true)

    nextBtn?.addEventListener('click', () => {
      if (activeStage && PROGRESS) {
        const nextId = PROGRESS.next(activeStage.levelId)
        if (nextId) saveActiveStage(nextId)
      }
    }, true)

    function syncInkBladeSession() {
      const inBattle = welcomeEl.hidden && selectEl.hidden && resultEl.hidden
      const soundOn = soundBtn?.getAttribute('aria-pressed') !== 'false'
      if (!inBattle) {
        updateSession({
          inRound: false,
          saveBadge: '进度已存盘',
        })
        return
      }
      const score = (document.getElementById('score')?.textContent || '0').trim()
      const areaName = (document.getElementById('areaLabel')?.textContent || activeStage?.name || '画卷').trim()
      const stagePrefix = activeStage ? `${activeStage.volName} · ${activeStage.sub}` : '当前画卷'
      const goalText = (document.getElementById('goal')?.textContent || '').trim() || '每幅三印：过(通关) · 全(少受伤) · 技(完成绝技)'

      updateSession({
        inRound: true,
        title: `${stagePrefix}「${areaName}」 · ${score} 分`,
        subtitle: goalText,
        saveBadge: '已存至本幅',
        exitHint: activeStage ? `下次可继续${activeStage.sub}` : '保留当前进度',
        resumeLabel: '继续挥剑',
        restartLabel: '重开本幅',
        confirmRestart: false,
        canSelectStage: true,
        selectStageLabel: '返回画卷',
        soundEnabled: soundOn,
      })
    }

    // 恢复上次停留的关卡
    Promise.all([
      xb.storage.get(STAGE_KEY).catch(() => null),
      xb.storage.get(saveKey).catch(() => null),
    ]).then(([savedStage, rawSave]) => {
      const derived = deriveStageFromSave(rawSave)
      if (savedStage && savedStage.levelId && describeLevel(savedStage.levelId)) {
        activeStage = describeLevel(savedStage.levelId)
        hasHistoryProgress = true
      } else if (derived) {
        activeStage = derived
      }
      refreshWelcomeContinueUI()
      decorateSelectCards()
      syncInkBladeSession()
    })

    const observer = new MutationObserver(() => {
      // 若刚通关本幅，自动将下次续玩目标推进到下一幅
      if (resultEl && !resultEl.hidden && activeStage && PROGRESS) {
        const won = document.getElementById('resultSeal')?.textContent === '品'
        if (won) {
          const nextId = PROGRESS.next(activeStage.levelId)
          if (nextId && nextId !== activeStage.levelId) {
            saveActiveStage(nextId)
          }
        }
      }
      decorateSelectCards()
      syncInkBladeSession()
    })

    for (const el of [welcomeEl, selectEl, pauseEl, resultEl]) {
      if (el) observer.observe(el, { attributes: true, attributeFilter: ['hidden'] })
    }
    if (levelStrip) observer.observe(levelStrip, { childList: true })
    const scoreNode = document.getElementById('score')
    const areaNode = document.getElementById('areaLabel')
    const goalNode = document.getElementById('goal')
    if (scoreNode) observer.observe(scoreNode, { childList: true, characterData: true, subtree: true })
    if (areaNode) observer.observe(areaNode, { childList: true, characterData: true, subtree: true })
    if (goalNode) observer.observe(goalNode, { childList: true, characterData: true, subtree: true })
    if (soundBtn) observer.observe(soundBtn, { attributes: true, attributeFilter: ['aria-pressed'] })

    // 宿主菜单关闭回到前台时，若游戏处于因 hide 触发的暂停态，自动恢复挥剑
    xb.on('show', () => {
      if (pauseEl && !pauseEl.hidden && welcomeEl.hidden && selectEl.hidden && resultEl.hidden) {
        resumeBtn?.click()
      }
      syncInkBladeSession()
    })

    xb.on('game.action', data => {
      const action = data?.action
      if (action === 'resume') {
        if (pauseEl && !pauseEl.hidden) resumeBtn?.click()
        syncInkBladeSession()
      } else if (action === 'restart-round') {
        restartBtn?.click()
        syncInkBladeSession()
      } else if (action === 'select-stage') {
        mapBtn?.click()
        syncInkBladeSession()
      } else if (action === 'toggle-sound') {
        soundBtn?.click()
        syncInkBladeSession()
      }
    })
  }

  function initEnhancements() {
    if (document.body?.dataset?.game === 'block-puzzle' || location.hostname.startsWith('official-block-puzzle.')) {
      setupBlockPuzzle()
    } else if (document.getElementById('inkMeter') || location.hostname.startsWith('official-ink-blade.')) {
      setupInkBlade()
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initEnhancements, { once: true })
  } else {
    queueMicrotask(initEnhancements)
  }

  async function connect(initialize, options = {}) {
    if (options.hide) xb.on('hide', options.hide)
    if (options.show) xb.on('show', options.show)
    if (options.layout) xb.on('safearea', () => options.layout(layout()))
    let save = null
    if (options.save) {
      try {
        save = await xb.storage.get(saveKey)
        if (save === null && xb.info.legacySave !== null) {
          save = xb.info.legacySave
          await xb.storage.set(saveKey, save)
        }
      } catch (error) {
        reportError(error)
      }
    } else {
      // The shared gameplay script finishes synchronous initialization first.
      await Promise.resolve()
    }
    document.body.classList.add('desktop-hosted')
    initialize({ ...layout(), save })
  }

  function send(type, payload = {}) {
    let result
    switch (type) {
      case 'game.ready': result = xb.ready(); break
      case 'game.start': result = xb.game.start(); break
      case 'game.progress': result = xb.game.progress({ score: payload.score }); break
      case 'game.result': result = xb.game.result({ score: payload.score }); break
      case 'game.save': result = xb.storage.set(saveKey, payload.data); break
      case 'game.exit': result = xb.exit(); break
    }
    result?.catch(reportError)
  }

  globalThis.XiaobanBuiltinRuntime = Object.freeze({
    connect(initialize, options) { connect(initialize, options).catch(reportError) },
    send,
    exportMbti(payload) { return xb.exportMbti(payload) },
  })
})()
