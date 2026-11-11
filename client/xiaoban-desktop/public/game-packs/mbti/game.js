(() => {
  'use strict'

  const runtime = globalThis.XiaobanBuiltinRuntime

  const rules = globalThis.XiaobanMbtiRules
  const nonce = new URLSearchParams(location.search).get('hostNonce')
  const byId = id => document.getElementById(id)
  const app = byId('app')
  const views = ['home', 'quiz', 'review', 'result', 'share']
  let save = rules.emptySave()
  let view = 'home'
  let hostReady = false
  let dirty = false
  let restoringHistory = false
  let reportChapter = 'traits'

  function rememberPage(replace = false) {
    const state = { mbti: true, page: view, chapter: reportChapter }
    window.history[replace ? 'replaceState' : 'pushState'](state, '')
  }
  // One history entry per page/section, not one for every answer.
  rememberPage(true)

  const systemTheme = window.matchMedia?.('(prefers-color-scheme: dark)')
  function applyTheme() {
    const theme = globalThis.xb?.info?.theme
    document.documentElement.dataset.theme = theme === 'dark' || (theme !== 'light' && systemTheme?.matches) ? 'dark' : 'light'
  }
  applyTheme()
  systemTheme?.addEventListener?.('change', applyTheme)
  globalThis.xb?.on?.('theme', applyTheme)

  function post(type, payload = {}) {
    if (runtime) { if (hostReady) runtime.send(type, payload); return }
    if (!hostReady || !nonce || window.parent === window) return
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...payload }, '*')
  }

  // Installed synchronously during script evaluation, before the iframe load event.
  function applyLayout(message) {
    if (message.safeArea && typeof message.safeArea === 'object') {
      for (const side of ['top', 'right', 'bottom', 'left']) {
        const value = message.safeArea[side]
        if (typeof value === 'number' && Number.isFinite(value) && value >= 0) {
          document.documentElement.style.setProperty('--host-' + side, Math.min(value, 160) + 'px')
        }
      }
    }
    if (message.capsule && typeof message.capsule === 'object') {
      for (const key of ['top', 'right', 'width', 'height']) {
        const value = message.capsule[key]
        if (typeof value === 'number' && Number.isFinite(value) && value >= 0) {
          document.documentElement.style.setProperty('--host-capsule-' + key, value + 'px')
        }
      }
    }
  }

  function initialize(message) {
    applyLayout(message)
    if (hostReady) { post('game.ready'); return }
    const restored = rules.normalizeSave(message.save)
    // A late initialization must not replace answers already entered in this session.
    if (!dirty) save = restored
    else if (!save.lastResult) save.lastResult = restored.lastResult
    hostReady = true
    post('game.ready')
    if (dirty) post('game.save', { data: save })
    if (view === 'home') renderHome()
  }

  if (!runtime) window.addEventListener('message', event => {
    const message = event.data
    if (!nonce || window.parent === window || event.source !== window.parent || !message ||
      message.channel !== 'xiaban.game-host' || message.protocol !== 1 || message.nonce !== nonce || message.type !== 'host.init') return
    initialize(message)
  })

  function persist() {
    dirty = true
    post('game.save', { data: save })
  }

  function show(next, focusId) {
    const changed = view !== next
    view = next
    if (changed && !restoringHistory) rememberPage()
    document.body.classList.toggle('is-report', next === 'result' || next === 'share')
    views.forEach(id => { byId(id).hidden = id !== next })
    byId('homeButton').hidden = next === 'home'
    byId('pageLabel').textContent = {home:'向内看', quiz:'认识自己的旅程', review:'留一点时间给自己', result:'你的性格画像', share:'留住此刻的自己'}[next]
    byId(next).scrollTop = 0
    if (focusId) byId(focusId).focus({ preventScroll: true })
  }

  function formatDate(value, includeYear = false) {
    const date = new Date(value)
    return (includeYear ? date.getFullYear() + ' 年 ' : '') + (date.getMonth() + 1) + ' 月 ' + date.getDate() + ' 日'
  }

  function renderHome() {
    const progress = save.progress
    byId('startButton').hidden = !!progress
    byId('resumeButton').hidden = !progress
    byId('resetButton').hidden = !progress
    if (progress) byId('resumeButton').textContent = '继续上次（已答 ' + rules.score(progress.answers).answered + ' 题）'
    const result = save.lastResult
    byId('lastResultButton').hidden = !result
    if (result) byId('lastResultMeta').textContent = result.type + ' · ' + globalThis.XiaobanMbtiProfiles[result.type].role + ' / ' + formatDate(result.completedAt)
    byId('storageNote').textContent = hostReady ? '答案与结果仅保存在这台设备上。' : '独立预览：进度仅在本次打开期间保留。'
    byId('storageNote').hidden = hostReady
  }

  function goHome() {
    renderHome()
    show('home', 'homeTitle')
  }

  function start() {
    save.progress = { current: 0, answers: rules.normalizeAnswers([]), mode: 'all' }
    persist()
    renderQuestion()
  }

  function renderQuestion() {
    const progress = save.progress
    if (!progress) return goHome()
    const question = rules.questions[progress.current]
    const answered = rules.score(progress.answers).answered
    byId('progressText').textContent = '已答 ' + answered + ' 题'
    byId('questionNumber').textContent = String(progress.current + 1).padStart(2, '0') + ' / 48'
    byId('progressBar').value = answered
    byId('questionEyebrow').textContent = progress.mode === 'missing' ? '补答 · 也可以暂时留白' : ['关于你的能量', '关于你的观察', '关于你的决定', '关于你的节奏'][question.dimension]
    byId('questionTitle').textContent = question.prompt
    for (let choice = 0; choice < 2; choice++) {
      byId('optionText' + choice).textContent = question.options[choice].text
      byId('option' + choice).setAttribute('aria-pressed', String(progress.answers[progress.current] === choice))
      byId('option' + choice).disabled = false
    }
    byId('previousButton').disabled = progress.current === 0
    byId('nextButton').disabled = progress.answers[progress.current] === null
    byId('nextButton').textContent = nextIndex(progress.current) >= 48 ? '查看结果 →' : '下一题 →'
    show('quiz', 'questionTitle')
  }

  function choose(choice) {
    if (view !== 'quiz' || !save.progress) return
    const progress = save.progress
    progress.answers[progress.current] = choice
    // Persist the selected answer immediately; explicit Next keeps it editable.
    const result = rules.score(progress.answers)
    persist()
    byId('progressText').textContent = '已答 ' + result.answered + ' 题'
    byId('progressBar').value = result.answered
    for (let index = 0; index < 2; index++) {
      byId('option' + index).setAttribute('aria-pressed', String(index === choice))
    }
    byId('nextButton').disabled = false
    byId('announcement').textContent = '已选择：' + rules.questions[progress.current].options[choice].text
  }

  function nextIndex(from) {
    let next = from + 1
    if (save.progress.mode === 'missing') {
      while (next < 48 && save.progress.answers[next] !== null) next++
    }
    return next
  }

  function advance() {
    if (view !== 'quiz') return
    const next = nextIndex(save.progress.current)
    if (next >= 48) return finishPass()
    save.progress.current = next
    persist()
    renderQuestion()
  }

  function previous() {
    if (view !== 'quiz' || save.progress.current === 0) return
    save.progress.current--
    persist()
    renderQuestion()
  }

  function finishPass() {
    if (rules.score(save.progress.answers).answered === 48) return finish()
    persist()
    renderReview()
  }

  function renderReview() {
    const result = rules.score(save.progress.answers)
    const missing = 48 - result.answered
    byId('reviewSummary').textContent = '已答 ' + result.answered + ' 题，还有 ' + missing + ' 题留白。可以补答，也可以只用已答题查看倾向。'
    byId('reviewNotice').textContent = result.type
      ? '未答题不计分；题目越少，结果越容易受单次选择影响。'
      : '有维度尚未作答。四个维度各至少答 1 题后，才能形成完整结果；建议补完再看。'
    byId('fillButton').textContent = '补答剩余 ' + missing + ' 题'
    byId('finishButton').disabled = !result.type
    show('review', 'reviewTitle')
  }

  function fillMissing() {
    save.progress.current = save.progress.answers.indexOf(null)
    save.progress.mode = 'missing'
    persist()
    renderQuestion()
  }

  function finish() {
    if (!save.progress) return
    const result = rules.score(save.progress.answers)
    if (!result.type) return renderReview()
    save.lastResult = { type: result.type, percentages: result.percentages, totals: result.totals, completedAt: new Date().toISOString(), answers: save.progress.answers.slice() }
    save.progress = null
    reportChapter = 'traits'
    persist()
    renderResult()
  }

  function renderResult() {
    const result = save.lastResult
    if (!result) return
    const report = globalThis.XiaobanMbtiReport.build(result)
    if (!report) return goHome()
    globalThis.XiaobanMbtiReportView.render(report, byId('reportBody'), {
      chapter: reportChapter,
      onChapterChange(chapter) {
        if (reportChapter === chapter) return
        reportChapter = chapter
        if (!restoringHistory) rememberPage()
      },
    })
    byId('shareButton').textContent=globalThis.xb?.info?.platform==='desktop'?'保存人格海报':'把这份认识，留成一张卡'
    show('result', 'resultType')
  }

  let shareSequence = 0, preparedPoster = null, preparedLink = null
  async function openShare() {
    if (!save.lastResult) return
    const sequence=++shareSequence
    preparedPoster=null; preparedLink=null
    for (const id of ['savePoster','sharePoster','shareLink']) byId(id).disabled=true
    byId('posterPreview').removeAttribute('src')
    byId('posterStatus').textContent='正在准备图片…'
    const desktop=globalThis.xb?.info?.platform==='desktop'
    byId('share').classList.toggle('desktop-share',desktop)
    byId('shareTitle').textContent=desktop?'保存人格海报':'分享我的结果'
    byId('sharePoster').hidden=desktop;byId('shareLink').hidden=desktop
    show('share','shareTitle')
    try {
      preparedLink=globalThis.XiaobanMbtiShare.link(save.lastResult)
      const image=await globalThis.XiaobanMbtiShare.poster(save.lastResult)
      if(sequence!==shareSequence || view!=='share') return
      preparedPoster=image;byId('posterPreview').src=image;byId('posterStatus').textContent=''
      for (const id of ['savePoster','sharePoster','shareLink']) byId(id).disabled=false
    } catch(error) { if(sequence===shareSequence) byId('posterStatus').textContent='图片准备失败，请返回报告后重试。' }
  }
  async function exportResult(action) {
    if (!preparedPoster || !preparedLink) return
    byId('posterStatus').textContent=''
    try {
      const payload={action,image:action==='link'?'':preparedPoster,url:action==='link'?preparedLink:''}
      if (runtime?.exportMbti) {
        const value=await runtime.exportMbti(payload)
        if(value==='cancelled') byId('posterStatus').textContent='已取消保存。'
        else if(value==='saved') byId('posterStatus').textContent='图片已保存。'
      } else if (nonce && window.parent!==window) {
        window.parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce,type:'mbti.export',...payload},'*')
      } else if(action==='save') {
        const a=document.createElement('a');a.href=preparedPoster;a.download='小伴人格画像-'+save.lastResult.type+'.png';document.body.append(a);a.click();a.remove()
      } else if(navigator.share) {
        const data=action==='link'?{title:'我的小伴人格画像',url:preparedLink}:{title:'我的小伴人格画像',files:[new File([await (await fetch(preparedPoster)).blob()],'小伴人格画像.png',{type:'image/png'})]}
        await navigator.share(data)
      } else byId('posterStatus').textContent='当前环境不支持分享，请保存图片后分享。'
    } catch(error) { if(error?.name!=='AbortError') byId('posterStatus').textContent='操作未完成，请重试。' }
  }

  byId('startButton').addEventListener('click', start)
  byId('resetButton').addEventListener('click', start)
  byId('retryButton').addEventListener('click', start)
  byId('resumeButton').addEventListener('click', renderQuestion)
  byId('homeButton').addEventListener('click', goHome)
  byId('lastResultButton').addEventListener('click', renderResult)
  byId('shareButton').addEventListener('click', openShare)
  byId('shareBack').addEventListener('click', () => {shareSequence++;window.history.back()})
  byId('savePoster').addEventListener('click', () => exportResult('save'))
  byId('sharePoster').addEventListener('click', () => exportResult('image'))
  byId('shareLink').addEventListener('click', () => exportResult('link'))
  byId('previousButton').addEventListener('click', previous)
  byId('skipButton').addEventListener('click', advance)
  byId('nextButton').addEventListener('click', () => {
    if (save.progress && save.progress.answers[save.progress.current] !== null) advance()
  })
  byId('fillButton').addEventListener('click', fillMissing)
  byId('finishButton').addEventListener('click', finish)
  byId('reviewPreviousButton').addEventListener('click', () => {
    save.progress.current = 47
    save.progress.mode = 'all'
    persist()
    renderQuestion()
  })
  for (let choice = 0; choice < 2; choice++) byId('option' + choice).addEventListener('click', () => choose(choice))
  window.addEventListener('keydown', event => {
    if (view !== 'quiz' || event.repeat || event.ctrlKey || event.metaKey || event.altKey) return
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes(event.target?.tagName)) return
    const actions = { '1': () => choose(0), '2': () => choose(1), ArrowLeft: previous, ArrowRight: advance }
    if (actions[event.key]) { event.preventDefault(); actions[event.key]() }
  })

  window.addEventListener('popstate', event => {
    shareSequence++
    restoringHistory = true
    const state = event.state?.mbti ? event.state : {page:'home'}
    reportChapter = state.chapter || 'traits'
    if (state.page === 'quiz' && save.progress) renderQuestion()
    else if (state.page === 'review' && save.progress) renderReview()
    else if (state.page === 'result' && save.lastResult) renderResult()
    else if (state.page === 'share' && save.lastResult) openShare()
    else goHome()
    rememberPage(true)
    restoringHistory = false
  })

  renderHome()
  if (runtime) runtime.connect(initialize, { save: true, layout: applyLayout })

})()
