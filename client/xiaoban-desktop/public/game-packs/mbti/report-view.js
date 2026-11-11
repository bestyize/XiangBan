(() => {
  'use strict'
  const order = ['INTJ','INTP','ENTJ','ENTP','INFJ','INFP','ENFJ','ENFP','ISTJ','ISFJ','ESTJ','ESFJ','ISTP','ISFP','ESTP','ESFP']
  const chapters = [['traits','倾向'],['strengths','特质'],['work','工作'],['relationships','关系'],['growth','成长'],['closing','尾声']]
  const actionChecks = new Map()
  const el = (tag, className, text) => {
    const node = document.createElement(tag)
    if (className) node.className = className
    if (text !== undefined) node.textContent = text
    return node
  }
  function button(text, className = '') { const node = el('button', className, text); node.type = 'button'; return node }
  function section(id, title) {
    const node = el('section', 'report-section')
    node.id = 'report-' + id; node.setAttribute('role', 'tabpanel'); node.setAttribute('aria-labelledby', 'report-tab-' + id); node.tabIndex = 0
    node.append(el('h3', 'report-section-title', title)); return node
  }
  function card(title, copy) { const node = el('article', 'report-card'); node.append(el('h4', '', title), el('p', '', copy)); return node }
  function list(items) { const node = el('ul', 'report-list'); items.forEach(text => node.append(el('li', '', text))); return node }
  function context(row) {
    const node = card(row.title + ' · ' + row.headline, row.observation)
    node.append(el('p', 'report-action', row.action))
    const detail = el('details', 'report-evidence')
    detail.append(el('summary', '', `来自你的 ${row.evidence.length} 个选择`))
    row.evidence.forEach(answer => { const p = el('p'); p.append(el('span', '', answer.prompt), el('span', 'report-answer', answer.answer)); detail.append(p) })
    node.append(detail); return node
  }
  function illustration(file, description) { const img = el('img', 'report-chapter-art'); img.src = './assets/' + file; img.alt = description; img.loading = 'lazy'; return img }
  function render(report, root, options = {}) {
    root.replaceChildren()
    const hero = el('header', 'report-hero'), identity = el('div', 'report-identity')
    identity.append(el('p', 'report-kicker', '很高兴，以这样的方式认识你'))
    const type = el('h2', 'report-type', report.type); type.id = 'resultType'; type.tabIndex = -1
    identity.append(type, el('p', 'report-role', report.profile.role), el('span', 'report-family', report.profile.family + ' · ' + report.family.name))
    let character
    if (report.type === 'INFJ') {
      character = el('img', 'report-character report-scene'); character.src = './assets/observer-jade.webp'; character.alt = 'INFJ 提灯者纸艺场景'
    } else {
      character = el('div', 'report-character'); const position = order.indexOf(report.type)
      character.style.backgroundPosition = `${position % 4 * 100 / 3}% ${Math.floor(position / 4) * 100 / 3}%`
      character.setAttribute('role', 'img'); character.setAttribute('aria-label', report.type + ' ' + report.profile.role)
    }
    hero.append(identity, character)
    const date = el('p', 'report-date'), time = el('time', '', new Date(report.completedAt).toLocaleDateString('zh-CN', {year:'numeric',month:'2-digit',day:'2-digit'}))
    time.dateTime = report.completedAt; date.append(time, el('span', '', ` · 已答 ${report.answered} / 48 道情境题`))
    root.append(hero, el('p', 'report-title', report.profile.title), date)
    const nav = el('nav', 'report-nav'); nav.setAttribute('role', 'tablist'); nav.setAttribute('aria-label', '报告章节')
    const panels = new Map(), tabs = new Map()
    function select(id, user = false) {
      if (!panels.has(id)) id = 'traits'
      panels.forEach((panel, key) => { panel.hidden = key !== id })
      tabs.forEach((tab, key) => { tab.setAttribute('aria-selected', String(key === id)); tab.tabIndex = key === id ? 0 : -1 })
      if (user) {
        options.onChapterChange?.(id)
        const scroller = root.closest('.view')
        if (scroller) scroller.scrollTop += nav.getBoundingClientRect().top - scroller.getBoundingClientRect().top
      }
    }
    chapters.forEach(([id, label], index) => {
      const tab = button(label); tab.id = 'report-tab-' + id
      tab.setAttribute('role', 'tab'); tab.setAttribute('aria-controls', 'report-' + id)
      tab.addEventListener('click', () => select(id, true))
      tab.addEventListener('keydown', event => {
        const target = event.key === 'ArrowRight' ? (index + 1) % chapters.length : event.key === 'ArrowLeft' ? (index + chapters.length - 1) % chapters.length : event.key === 'Home' ? 0 : event.key === 'End' ? chapters.length - 1 : -1
        if (target < 0) return
        event.preventDefault(); const key = chapters[target][0]; select(key, true); tabs.get(key).focus({preventScroll:true})
      })
      tabs.set(id, tab); nav.append(tab)
    })
    root.append(nav)
    const content = el('div', 'report-content')
    const traits = section('traits', '你的四维倾向')
    traits.append(el('p', 'report-section-hint', '轻点，读懂每一面'))
    const dimensions = el('div', 'report-dimensions')
    report.dimensions.forEach(d => {
      const detail = el('details', 'report-dimension'), summary = el('summary')
      const heading = el('div', 'report-dimension-heading')
      heading.append(el('strong', '', d.context), el('span', '', d.percent === 50 ? '两侧相近' : `${d.strength}% ${d.pole === d.first ? d.firstLabel : d.secondLabel}`))
      const labels = el('div', 'report-dimension-label')
      labels.append(el('span', d.pole === d.first && d.percent !== 50 ? 'dominant' : '', `${d.first} ${d.firstLabel}`), el('span', d.pole === d.second && d.percent !== 50 ? 'dominant' : '', `${d.secondLabel} ${d.second}`))
      const track = el('div', 'report-track'), fill = el('i'), marker = el('b')
      fill.style.width = (100 - d.percent) + '%'; marker.style.left = (100 - d.percent) + '%'; track.append(fill, marker)
      track.setAttribute('role', 'img'); track.setAttribute('aria-label', `${d.firstLabel} ${d.percent}%，${d.secondLabel} ${100 - d.percent}%`)
      summary.append(heading, labels, track)
      detail.append(summary, el('p', 'report-band', `${d.label} · 已答 ${d.total}/12`), el('p', '', d.description)); dimensions.append(detail)
    })
    traits.append(dimensions, el('p', 'report-intro', report.profile.description))
    const strengths = section('strengths', '你的力量与另一面'), switcher = el('div', 'report-lens'), strengthsList = el('div', 'report-card')
    const lenses = [['strengths','自然的优势'],['blindSpots','值得照看的地方']]
    const lensButtons = lenses.map(([key, label]) => {
      const tab = button(label); tab.setAttribute('aria-pressed', String(key === 'strengths'))
      tab.addEventListener('click', () => { lensButtons.forEach(item => item.setAttribute('aria-pressed', String(item === tab))); strengthsList.replaceChildren(list(report.profile[key])) })
      switcher.append(tab); return tab
    })
    strengthsList.append(list(report.profile.strengths)); strengths.append(switcher, strengthsList)
    const work = section('work', '找到适合你的节奏')
    if (report.type === 'INFJ') work.append(illustration('work.webp', '提灯者在桌前整理想法'))
    report.profile.work.forEach((copy, index) => work.append(card(['进入任务','协作推进','节奏调整'][index], copy)))
    report.contexts.filter(row => row.section === 'work').forEach(row => work.append(context(row)))
    const relationships = section('relationships', '让彼此，更靠近一点')
    if (report.type === 'INFJ') relationships.append(illustration('relations.webp', '提灯者与朋友交流'))
    relationships.append(card('关系里的你', report.profile.relationship))
    report.contexts.filter(row => row.section === 'relationships').forEach(row => relationships.append(context(row)))
    const growth = section('growth', '把认识，带回生活'), steps = el('div', 'report-card')
    const checkKey = report.type + ':' + report.completedAt
    if (!actionChecks.has(checkKey)) actionChecks.set(checkKey, new Set())
    // UI-only completion marks remain while navigating this report in the current session.
    const checked = actionChecks.get(checkKey)
    report.actions.forEach((copy, index) => {
      const label = el('label', 'report-step'), input = el('input'), text = el('span')
      input.type = 'checkbox'; input.checked = checked.has(index)
      input.addEventListener('change', () => input.checked ? checked.add(index) : checked.delete(index))
      text.append(el('small', '', 'DAY ' + ['01','03','07'][index]), el('span', '', copy)); label.append(input, text); steps.append(label)
    })
    growth.append(steps, el('p', 'report-intro', '不需要一下子改变很多。留一周时间，感受这些小尝试带来的变化。'))
    report.contexts.filter(row => row.section === 'life').forEach(row => growth.append(context(row)))
    const closing = section('closing', '写给此刻的你')
    closing.append(card('四个字母，是一个认识自己的起点。', report.profile.description), card('带走一句话', report.profile.growth[2]), el('p', 'report-intro', '这份画像记录的是此刻的倾向。生活会变化，你也拥有选择不同做法的自由。'))
    if (!report.hasAnswerSnapshot) closing.append(el('p', 'report-legacy', '这份历史记录保留了类型和四维分数。下次测评会同时留下具体情境分析。'))
    ;[traits,strengths,work,relationships,growth,closing].forEach((panel, index) => { panels.set(chapters[index][0], panel); content.append(panel) })
    root.append(content); select(options.chapter || 'traits')
    return {select}
  }
  globalThis.XiaobanMbtiReportView = {render}
})()
