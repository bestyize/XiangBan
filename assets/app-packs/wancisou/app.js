(() => {
  'use strict';

  const API = 'https://search.xiangban.xyz';
  const LIMIT = 2_000_000;
  const HISTORY_KEY = 'wancisou.recent-searches';
  const names = ['Solid', 'YuHuaGe', '海盗湾', 'AcgRip', 'Zero', '1337', 'BtSow', 'TorLock', 'Kitty', '磁力猫', '磁力多'];
  const roles = ['综合', '综合', '综合', '动漫', '综合', '国际', '综合', '国际', '综合', '中文', '中文'];
  const sources = names.map((title, src) => ({ src, title, role: roles[src] }));
  const $ = selector => document.querySelector(selector);
  const app = $('#app');
  const shell = $('#app-shell');
  const form = $('#search-form');
  const query = $('#query');
  const clearQuery = $('#clear-query');
  const submit = $('#submit');
  const sourceButton = $('#source');
  const results = $('#results');
  const overlay = $('#sheet-overlay');
  const toastBox = $('#toast');
  const state = {
    keyword: '', source: 'all', searchedSource: 'all', sort: 'relevance', page: 0,
    items: [], history: [], historyRevision: 0, historyWrites: Promise.resolve(),
    view: 'home', busy: false, hasMore: false, moreError: false,
    search: null, request: 0, magnet: null, sheet: null, opener: null, closing: false,
  };
  let toastTimer;

  function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = String(text);
    return node;
  }

  // All SVG geometry is static. API strings only ever reach textContent.
  const paths = {
    search: ['M16 16l4.2 4.2', 'M17.5 11a6.5 6.5 0 1 0-13 0 6.5 6.5 0 0 0 13 0'],
    close: ['M6 6l12 12M18 6 6 18'],
    copy: ['M11.5 9h6A2.5 2.5 0 0 1 20 11.5v6a2.5 2.5 0 0 1-2.5 2.5h-6A2.5 2.5 0 0 1 9 17.5v-6A2.5 2.5 0 0 1 11.5 9Z', 'M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1'],
    link: ['M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71', 'M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71'],
    check: ['m5 12.5 4.5 4.5L19 7.5'],
    alert: ['M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0', 'M12 8v4.5M12 16.2h.01'],
    clock: ['M20.5 12a8.5 8.5 0 1 0-17 0 8.5 8.5 0 0 0 17 0', 'M12 7.5v4.8l3 1.8'],
    searchOff: ['M16.5 10.5a6 6 0 1 0-12 0 6 6 0 0 0 12 0', 'm15 15 4.5 4.5M8.5 8.5l4 4M12.5 8.5l-4 4'],
    flame: ['M12.5 3.5c.8 2.6-.7 4.4-2.2 6-1.4 1.5-2.3 3-2.3 5a4.5 4.5 0 0 0 9 0c0-3.2-2-6.1-4.5-11Z', 'M11 15.2a1.8 1.8 0 0 0 3.6 0c0-1.1-.8-2-1.8-3.1-1 1.1-1.8 2-1.8 3.1Z'],
    film: ['M5.5 5h13A2.5 2.5 0 0 1 21 7.5v9a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 16.5v-9A2.5 2.5 0 0 1 5.5 5Z', 'M7 5v14M17 5v14M3 9.5h4M3 14.5h4M17 9.5h4M17 14.5h4'],
    tv: ['M5.5 6h13A2.5 2.5 0 0 1 21 8.5v7a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 15.5v-7A2.5 2.5 0 0 1 5.5 6Z', 'm8.5 3 3.5 3 3.5-3M9 21h6'],
    box: ['M12 3 4 7.5v9L12 21l8-4.5v-9L12 3Z', 'M4 7.5 12 12l8-4.5M12 12v9'],
    music: ['M9 17.5V6l10-2v11.5', 'M9 17.5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0M19 15.5a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0'],
    magnet: ['M6.5 6.5v5.8a5.5 5.5 0 0 0 11 0V6.5', 'M5 6.5h3M16 6.5h3M6.5 10h3M14.5 10h3'],
    spinner: ['M12 3a9 9 0 1 0 9 9'],
  };

  function svgNode(tag, attributes) {
    const node = document.createElementNS('http://www.w3.org/2000/svg', tag);
    for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
    return node;
  }

  function icon(name) {
    const svg = svgNode('svg', { class: name === 'spinner' ? 'icon spin' : 'icon', viewBox: '0 0 24 24', 'aria-hidden': 'true' });
    for (const d of paths[name]) svg.append(svgNode('path', { d }));
    return svg;
  }

  function button(className, label, action, iconName) {
    const node = el('button', className);
    node.type = 'button';
    if (iconName) node.append(icon(iconName));
    node.append(el('span', null, label));
    node.addEventListener('click', action);
    return node;
  }

  function sourceName(src = state.source) {
    return src === 'all' ? '全部来源' : sources[Number(src)]?.title || '未知来源';
  }

  function dot(src) { return el('span', `site-dot site-dot--${Number.isInteger(src) && src >= 0 && src <= 10 ? src : 4}`); }

  function sourceTag(src) {
    const tag = el('span', 'source-tag');
    tag.append(dot(src), el('span', null, sourceName(String(src))));
    return tag;
  }

  function toast(text, error = false) {
    toastBox.replaceChildren(icon(error ? 'alert' : 'check'), el('span', null, text));
    toastBox.classList.toggle('is-error', error);
    toastBox.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toastBox.hidden = true; }, 2600);
  }

  async function getJson(url, controller, timeoutMs) {
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetch(url, { signal: controller.signal, credentials: 'omit', redirect: 'error', cache: 'no-store' });
      if (Number(response.headers.get('content-length')) > LIMIT) {
        await response.body?.cancel();
        throw new Error('响应过大，请选择单个来源检索。');
      }
      const reader = response.body?.getReader();
      let text;
      if (reader) {
        const decoder = new TextDecoder();
        let size = 0;
        text = '';
        try {
          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            size += value.byteLength;
            if (size > LIMIT) { await reader.cancel(); throw new Error('响应过大，请选择单个来源检索。'); }
            text += decoder.decode(value, { stream: true });
          }
          text += decoder.decode();
        } finally { reader.releaseLock(); }
      } else {
        text = await response.text();
        if (new TextEncoder().encode(text).byteLength > LIMIT) throw new Error('响应过大，请选择单个来源检索。');
      }
      let data;
      try { data = JSON.parse(text); } catch { throw new Error('服务返回的内容无法识别，请重试。'); }
      if (!response.ok) throw new Error(response.status === 429 ? '搜索请求较多，请稍后重试。' : '搜索服务暂时不可用，请稍后重试。');
      if (!data || typeof data !== 'object') throw new Error('服务返回的内容无法识别，请重试。');
      return data;
    } finally { clearTimeout(timer); }
  }

  async function loadSources() {
    try {
      const data = await getJson(`${API}/api/sources`, new AbortController(), 10000);
      if (!Array.isArray(data.sources)) return;
      for (const entry of data.sources) {
        if (Number.isInteger(entry?.src) && sources[entry.src] && typeof entry.title === 'string' && entry.title.trim()) {
          sources[entry.src].title = entry.title.slice(0, 40);
        }
      }
      updateToolbar();
      if (state.view === 'home') renderHome();
      if (state.sheet?.type === 'sources') renderSources();
    } catch { /* Published source IDs remain available when the directory is down. */ }
  }

  function cleanHistory(value) {
    return Array.isArray(value) ? [...new Set(value.filter(x => typeof x === 'string').map(x => x.trim().slice(0, 128)).filter(Boolean))].slice(0, 8) : [];
  }

  async function loadHistory() {
    const revision = state.historyRevision;
    let value;
    try { if (window.xb?.storage?.get) value = await window.xb.storage.get(HISTORY_KEY); } catch { /* Try browser storage. */ }
    if (value == null) {
      try { value = JSON.parse(localStorage.getItem(HISTORY_KEY) || '[]'); } catch { value = []; }
    }
    if (revision !== state.historyRevision) return;
    state.history = cleanHistory(value);
    if (state.view === 'home') renderHome();
  }

  function saveHistory(value) {
    state.history = cleanHistory(value);
    state.historyRevision++;
    const snapshot = [...state.history];
    // Serialize host writes so a late search write cannot resurrect cleared history.
    state.historyWrites = state.historyWrites.then(async () => {
      let stored = false;
      try {
        if (window.xb?.storage?.set) { await window.xb.storage.set(HISTORY_KEY, snapshot); stored = true; }
      } catch { /* Fall back to browser storage, then the in-memory state. */ }
      try {
        if (stored) localStorage.removeItem(HISTORY_KEY);
        else localStorage.setItem(HISTORY_KEY, JSON.stringify(snapshot));
      } catch { /* Opaque origins and full storage still retain this session's history. */ }
    });
  }

  function renderHome() {
    const hero = el('section', 'hero-card');
    hero.setAttribute('aria-label', '跨站磁力聚合');
    const art = svgNode('svg', { class: 'hero-art', viewBox: '0 0 148 148', fill: 'none', 'aria-hidden': 'true' });
    for (const [r, opacity] of [[56, '.09'], [38, '.14']]) art.append(svgNode('circle', { cx: 96, cy: 52, r, stroke: '#FFFFFF', 'stroke-opacity': opacity, 'stroke-width': 1.2 }));
    const magnet = svgNode('g', { transform: 'translate(68, 22) rotate(14 28 28)' });
    magnet.append(svgNode('path', { d: 'M12 10v20a16 16 0 0 0 32 0V10', stroke: '#FF7589', 'stroke-width': 9, 'stroke-linecap': 'round' }), svgNode('path', { d: 'M8.5 10h7M40.5 10h7', stroke: '#FFF2E0', 'stroke-width': 7.5, 'stroke-linecap': 'round' }));
    art.append(magnet);
    const content = el('div', 'hero-content');
    const pills = el('div', 'hero-pills');
    for (const label of ['11 索引源', '按需解析', '一键复制']) pills.append(el('span', 'hero-pill', label));
    content.append(el('h2', null, '跨站磁力聚合检索'), el('p', null, '汇聚 11 个公开索引站 · 体积与做种健康度直观可读'), pills);
    hero.append(art, content);
    const scenes = el('section', 'scene-grid');
    scenes.setAttribute('aria-label', '快捷探索');
    for (const [label, word, tone, glyph] of [
      ['电影 4K', '星际穿越 4K', 'crimson', 'film'], ['动漫剧集', '进击的巨人', 'cobalt', 'tv'],
      ['开源软件', 'Ubuntu 24.04', 'emerald', 'box'], ['无损音乐', 'FLAC 无损', 'amber', 'music'],
    ]) {
      const tile = button('scene-tile', label, () => runSearch(false, word));
      tile.lastChild.className = 'scene-label';
      const mark = el('span', `scene-icon is-${tone}`);
      mark.append(icon(glyph)); tile.prepend(mark); scenes.append(tile);
    }
    const recent = el('section', 'home-section');
    recent.setAttribute('aria-label', '最近搜索');
    const head = el('div', 'section-head');
    head.append(el('h3', null, '最近搜索'));
    if (state.history.length) head.append(button('text-action', '清空', () => { saveHistory([]); renderHome(); toast('已清空最近搜索'); }));
    const chips = el('div', 'chip-wrap');
    for (const word of state.history) chips.append(button('query-chip', word, () => runSearch(false, word), 'clock'));
    if (!state.history.length) chips.append(el('p', 'history-empty', '搜索记录仅保存在本机'));
    recent.append(head, chips);
    const info = el('section', 'info-card');
    info.setAttribute('aria-label', '已接入索引来源');
    const infoHead = el('div', 'info-card-head');
    infoHead.append(el('h3', 'info-card-title', '已接入 11 个公开磁力源'), el('span', 'info-card-hint', '点选切换'));
    const grid = el('div', 'info-card-sources');
    for (const entry of sources) {
      const chip = button(`info-source-chip${state.source === String(entry.src) ? ' is-active' : ''}`, entry.title, () => pickSource(String(entry.src)));
      chip.prepend(dot(entry.src)); chip.setAttribute('aria-pressed', String(state.source === String(entry.src))); grid.append(chip);
    }
    info.append(infoHead, grid, el('p', 'info-card-note', '仅检索公开索引并复制磁力链接，不下载 BT 文件。'));
    results.replaceChildren(hero, scenes, recent, info);
  }

  function count(value) {
    if (typeof value === 'number') return Number.isFinite(value) ? Math.max(0, value) : 0;
    const text = String(value ?? '').replace(/,/g, '').trim();
    const match = text.match(/^(\d+(?:\.\d+)?)\s*([kmb万亿])?/i);
    return match ? Number(match[1]) * ({ k: 1e3, m: 1e6, b: 1e9, 万: 1e4, 亿: 1e8 }[match[2]?.toLowerCase()] || 1) : 0;
  }

  function seeds(item) { return count(item.sender_count ?? item.seeds); }
  function downloads(item) { return count(item.download_count ?? item.downloads); }

  function sizeBytes(item) {
    if (Number.isFinite(item.size_bytes)) return item.size_bytes;
    const match = String(item.size || '').replace(/,/g, '').match(/([\d.]+)\s*([KMGTPE]?)(?:I?B|字节)?\s*$/i);
    return match ? Number(match[1]) * 1024 ** (match[2] ? 'KMGTPE'.indexOf(match[2].toUpperCase()) + 1 : 0) : 0;
  }

  function dateValue(item) {
    const value = Date.parse(String(item.date || '').replace(/年|月/g, '-').replace(/日/g, ''));
    return Number.isFinite(value) ? value : 0;
  }

  function facts(item, className = 'result-facts') {
    const row = el('div', className);
    if (item.size) row.append(el('span', 'size-pill', item.size));
    const amount = seeds(item);
    const seed = el('span', `seed-badge ${amount >= 10 ? 'high' : amount >= 1 ? 'mid' : 'low'}`);
    seed.append(el('span', 'health-dot'), el('span', null, `做种 ${item.sender_count == null && item.seeds == null ? '未知' : amount.toLocaleString('zh-CN')}`));
    row.append(seed);
    if (downloads(item)) row.append(el('span', 'fact-meta', `下载 ${downloads(item).toLocaleString('zh-CN')}`));
    if (item.date) row.append(el('span', 'fact-meta', item.date));
    return row;
  }

  function knownMagnet(item) {
    return typeof item.magnet_url === 'string' && /^magnet:\?/i.test(item.magnet_url) && item.magnet_url.length <= 10000 ? item.magnet_url : null;
  }

  function canResolve(item) {
    return Boolean(knownMagnet(item) || Number.isInteger(item.src) && item.src >= 0 && item.src <= 10 && typeof item.detail_url === 'string' && /^https?:\/\//i.test(item.detail_url));
  }

  function card(item) {
    const article = el('article', 'result-card');
    const head = el('div', 'result-head');
    const title = el('h2', 'result-title');
    const titleButton = button('result-title', item.title || '未命名条目', () => openMagnet(item, titleButton));
    titleButton.disabled = !canResolve(item); titleButton.setAttribute('aria-label', `${item.title || '未命名条目'}，查看磁力详情`);
    title.append(titleButton); head.append(title);
    if (seeds(item) >= 20) { const hot = el('span', 'hot-badge'); hot.append(icon('flame'), el('span', null, '热种')); head.append(hot); }
    const foot = el('div', 'result-foot');
    const hasMagnet = knownMagnet(item);
    const action = button(`card-btn${hasMagnet ? '' : ' is-resolve'}`, hasMagnet ? '复制磁力' : '获取磁力', () => {
      if (knownMagnet(item)) copyMagnet(knownMagnet(item), action, null, item);
      else openMagnet(item, action);
    }, hasMagnet ? 'copy' : 'link');
    action.disabled = !canResolve(item);
    foot.append(sourceTag(item.src), action);
    article.append(head, facts(item), foot);
    return article;
  }

  function skeleton() {
    const box = el('div', 'skel-card'); box.setAttribute('aria-hidden', 'true');
    for (const name of ['t1', 't2', 'facts']) box.append(el('div', `skel-bar skel-${name}`));
    const foot = el('div', 'skel-foot'); foot.append(el('div', 'skel-bar skel-tag'), el('div', 'skel-bar skel-btn')); box.append(foot);
    return box;
  }

  function renderLoading() {
    const meta = el('div', 'results-meta'); const status = el('span', 'meta-status');
    status.append(icon('spinner'), el('span', null, `正在${state.source === 'all' ? '跨站' : sourceName()}检索「${state.keyword}」…`));
    meta.append(status); results.replaceChildren(meta, ...Array.from({ length: 4 }, skeleton));
  }

  function renderEmpty(errorMessage) {
    const error = state.view === 'error';
    const filtered = state.source !== 'all';
    const box = el('section', 'state-view');
    const mark = el('div', `state-icon${error ? ' is-error' : ''}`); mark.append(icon(error ? 'alert' : 'searchOff'));
    const actions = el('div', 'state-actions');
    actions.append(button('btn-primary-pill', error ? '重试搜索' : '修改关键词', () => error ? runSearch(true) : (query.focus(), query.select())));
    if (error) actions.append(button('btn-soft', '切换来源', () => openSources(document.activeElement)));
    else if (filtered) actions.append(button('btn-soft', '搜全部来源', () => pickSource('all')));
    box.append(mark, el('h2', null, error ? '搜索暂时不可用' : filtered ? '当前来源暂无匹配' : '没有找到相关结果'), el('p', null, error ? errorMessage : filtered ? `${sourceName()} 下没有找到「${state.keyword}」，可切回全部来源查看。` : '换一个更简短的关键词，或切换其他搜索来源再试。'), actions);
    results.replaceChildren(box);
  }

  function renderResults() {
    const meta = el('div', 'results-meta');
    const sourceCount = new Set(state.items.map(item => item.src)).size;
    meta.append(el('span', null, `已加载 ${state.items.length} 条 · ${state.source === 'all' ? `${sourceCount} 个来源` : sourceName()}`));
    if (state.source !== 'all') meta.append(button('reset-source-link', '切回全部', () => pickSource('all')));
    const list = [...state.items];
    const compare = state.sort === 'size' ? (a, b) => sizeBytes(b) - sizeBytes(a) : state.sort === 'seeds' ? (a, b) => seeds(b) - seeds(a) || downloads(b) - downloads(a) : state.sort === 'date' ? (a, b) => dateValue(b) - dateValue(a) : () => 0;
    list.sort((a, b) => compare(a, b) || a.rank - b.rank);
    const fragment = document.createDocumentFragment(); fragment.append(meta);
    for (const item of list) fragment.append(card(item));
    if (state.hasMore) {
      const more = button('load-more', state.busy ? '正在加载…' : state.moreError ? '重试加载下一页' : '加载下一页', () => loadPage(state.page + 1));
      more.disabled = state.busy; fragment.append(more);
    } else fragment.append(el('p', 'results-meta end-note', '已显示当前可用结果'));
    results.replaceChildren(fragment);
  }

  function updateToolbar() {
    const filtered = state.source !== 'all';
    $('#source-label').textContent = sourceName(); $('#source-dot').hidden = !filtered;
    sourceButton.classList.toggle('is-filtered', filtered);
    sourceButton.setAttribute('aria-label', `选择搜索来源，当前为${sourceName()}`);
    clearQuery.hidden = !query.value;
    for (const tab of document.querySelectorAll('[data-sort]')) {
      const selected = tab.dataset.sort === state.sort;
      tab.setAttribute('aria-selected', String(selected)); tab.tabIndex = selected ? 0 : -1;
    }
    submit.disabled = state.busy;
    if (state.busy) submit.replaceChildren(icon('spinner'), el('span', null, '搜索中'));
    else submit.textContent = '搜索';
    results.setAttribute('aria-busy', String(state.busy));
  }

  function dedupeKeys(item) {
    const hash = btih(knownMagnet(item), item.hash);
    return [hash && `hash:${hash}`, item.detail_url && `detail:${item.src}:${item.detail_url}`, !hash && !item.detail_url && `title:${item.src}:${item.title}`].filter(Boolean);
  }

  async function loadPage(page) {
    if (state.busy || page > 100) return;
    const controller = new AbortController();
    const request = ++state.request;
    state.search = controller; state.busy = true; state.moreError = false;
    updateToolbar();
    if (page === 1) { state.view = 'loading'; renderLoading(); results.scrollTop = 0; }
    else renderResults();
    const url = new URL('/api/search', API);
    for (const [key, value] of Object.entries({ q: state.keyword, page, src: state.source, filter: 1, optimize: 1 })) url.searchParams.set(key, String(value));
    try {
      const data = await getJson(url, controller, 65000);
      if (request !== state.request) return;
      if (!Array.isArray(data.items)) throw new Error('搜索结果格式异常，请重试。');
      if (!data.items.length && Array.isArray(data.sources) && data.sources.length && data.sources.every(entry => entry.ok === false)) {
        throw new Error('所选来源暂时未能返回结果，请重试或切换来源。');
      }
      const seen = new Set(state.items.flatMap(dedupeKeys));
      for (const item of data.items) {
        if (!item || typeof item !== 'object' || !Number.isInteger(item.src) || item.src < 0 || item.src > 10 || typeof item.title !== 'string') continue;
        const keys = dedupeKeys(item);
        if (keys.some(key => seen.has(key))) continue;
        keys.forEach(key => seen.add(key));
        state.items.push({ ...item, rank: state.items.length });
      }
      state.page = page;
      state.hasMore = data.has_more !== false && data.items.length > 0 && page < 100;
      state.busy = false;
      state.view = state.items.length ? 'results' : 'empty';
      if (state.items.length) renderResults(); else renderEmpty();
    } catch (error) {
      if (request !== state.request) return;
      state.busy = false;
      if (page === 1) {
        state.view = 'error';
        renderEmpty(controller.signal.aborted ? '来源响应超时，请稍后重试或选择单个站点检索。' : error.message);
      } else { state.moreError = true; renderResults(); toast('下一页加载失败，请重试', true); }
    } finally {
      if (state.search === controller) state.search = null;
      if (request === state.request) { state.busy = false; updateToolbar(); }
    }
  }

  function runSearch(force = false, raw = query.value) {
    const keyword = raw.trim().slice(0, 128);
    if (!keyword) { query.focus(); toast('请输入搜索关键词', true); return; }
    if (!force && state.busy && keyword === state.keyword && state.source === state.searchedSource) return;
    state.request++; state.search?.abort(); state.search = null; state.busy = false;
    query.value = keyword; query.blur();
    state.keyword = keyword; state.searchedSource = state.source; state.page = 0; state.items = [];
    saveHistory([keyword, ...state.history.filter(word => word !== keyword)]);
    loadPage(1);
  }

  function pickSource(src) {
    if (src !== 'all' && !sources[Number(src)]) return;
    const changed = state.source !== src; state.source = src; updateToolbar();
    if (state.sheet) closeSheet();
    if (state.view === 'home') renderHome();
    else if (changed && state.keyword) runSearch(true, state.keyword);
  }

  function showSheet(type, opener, item) {
    if (state.closing) return false;
    state.magnet?.abort(); state.magnet = null;
    if (!state.sheet) {
      state.opener = opener;
      try { history.pushState({ ...history.state, wancisouSheet: true }, ''); } catch { /* Restricted previews may have no history access. */ }
    }
    state.sheet = { type, item };
    overlay.hidden = false; shell.inert = true;
    sourceButton.setAttribute('aria-expanded', String(type === 'sources'));
    return true;
  }

  function dismissSheet() {
    state.magnet?.abort(); state.magnet = null; state.sheet = null; state.closing = false;
    overlay.hidden = true; overlay.replaceChildren(); shell.inert = false;
    sourceButton.setAttribute('aria-expanded', 'false');
    if (state.opener?.isConnected) state.opener.focus({ preventScroll: true });
    state.opener = null;
  }

  function closeSheet() {
    if (!state.sheet || state.closing) return;
    if (history.state?.wancisouSheet) { state.closing = true; history.back(); }
    else dismissSheet();
  }

  function sheetFrame(title) {
    const existing = overlay.querySelector('.bottom-sheet');
    if (existing && existing.querySelector('#sheet-title')?.textContent === title) {
      const body = existing.querySelector('.sheet-body');
      body.replaceChildren(); existing.querySelector('.sheet-footer')?.remove();
      return { sheet: existing, body };
    }
    const sheet = el('section', 'bottom-sheet');
    sheet.setAttribute('role', 'dialog'); sheet.setAttribute('aria-modal', 'true'); sheet.setAttribute('aria-labelledby', 'sheet-title'); sheet.tabIndex = -1;
    const head = el('header', 'sheet-head'); const heading = el('div', 'sheet-head-title');
    const mark = el('span', 'sheet-mini-badge'); mark.append(icon('magnet'));
    const h2 = el('h2', null, title); h2.id = 'sheet-title'; heading.append(mark, h2);
    const close = button('sheet-close', '', closeSheet, 'close'); close.setAttribute('aria-label', '关闭');
    head.append(heading, close);
    const body = el('div', 'sheet-body'); sheet.append(el('div', 'sheet-handle'), head, body);
    overlay.replaceChildren(sheet);
    return { sheet, body };
  }

  function sheetFooter(sheet, label) {
    const footer = el('div', 'sheet-footer'); footer.append(button('sheet-cancel', label, closeSheet)); sheet.append(footer);
  }

  function openSources(opener = sourceButton) {
    if (!showSheet('sources', opener)) return;
    renderSources(); overlay.querySelector('.bottom-sheet').focus();
  }

  function renderSources() {
    const { sheet, body } = sheetFrame('选择搜索来源');
    const group = el('div'); group.setAttribute('role', 'radiogroup'); group.setAttribute('aria-label', '搜索来源');
    const all = button('source-all-btn', '', () => pickSource('all'));
    all.replaceChildren(); all.setAttribute('role', 'radio'); all.setAttribute('aria-checked', String(state.source === 'all'));
    const left = el('span', 'source-all-left'); const mark = el('span', 'source-all-icon'); mark.append(icon('magnet'));
    const text = el('span'); text.append(el('b', null, '全部来源'), el('small', null, '跨站聚合 11 个公开磁力站点'));
    left.append(mark, text); all.append(left); if (state.source === 'all') all.append(icon('check'));
    const grid = el('div', 'source-grid');
    for (const entry of sources) {
      const choice = button('source-item', '', () => pickSource(String(entry.src)));
      choice.replaceChildren(); choice.setAttribute('role', 'radio'); choice.setAttribute('aria-checked', String(state.source === String(entry.src)));
      const name = el('span', 'source-item-name'); name.append(dot(entry.src), el('span', null, entry.title));
      choice.append(name, el('span', 'source-item-role', entry.role)); grid.append(choice);
    }
    group.append(all, grid); body.append(group); sheetFooter(sheet, '取消');
  }

  // BTIH may be returned as 40 hex digits or 32 base32 digits. Display full hex.
  function btih(link, fallback) {
    let value = '';
    try { value = new URL(link).searchParams.getAll('xt').find(x => /^urn:btih:/i.test(x))?.slice(9) || ''; } catch { /* Use an explicit hash when available. */ }
    if (!value && typeof fallback === 'string') value = fallback;
    if (/^[a-f\d]{40}$/i.test(value)) return value.toUpperCase();
    if (!/^[a-z2-7]{32}$/i.test(value)) return '';
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let buffer = 0, bits = 0, hex = '';
    for (const char of value.toUpperCase()) {
      buffer = (buffer << 5) | alphabet.indexOf(char); bits += 5;
      if (bits >= 8) { bits -= 8; hex += ((buffer >> bits) & 255).toString(16).padStart(2, '0'); }
    }
    return hex.toUpperCase();
  }

  function renderMagnet(status, reason) {
    const item = state.sheet.item;
    const { sheet, body } = sheetFrame('磁力链接');
    const tags = facts(item, 'sheet-item-facts'); tags.prepend(sourceTag(item.src));
    body.append(el('h3', 'sheet-item-title', item.title || '未命名条目'), tags);
    if (status === 'resolving') {
      const box = el('div', 'sheet-resolving'); const row = el('div', 'sheet-resolving-row');
      row.append(icon('spinner'), el('span', null, `正在向 ${sourceName(String(item.src))} 解析磁力链接…`));
      box.append(row, el('div', 'skel-bar skel-t1'), el('div', 'skel-bar skel-t2'));
      const cta = button('sheet-primary-btn', '正在解析…', () => {}, 'spinner'); cta.disabled = true;
      body.append(box, cta); sheetFooter(sheet, '关闭');
    } else if (status === 'error') {
      const box = el('div', 'sheet-error-box'); const text = el('div');
      text.append(el('b', null, '暂时无法获取该磁力链接'), el('span', null, reason)); box.append(icon('alert'), text);
      body.append(box, button('sheet-retry-btn', '重新获取', resolveMagnet)); sheetFooter(sheet, '关闭');
    } else {
      const link = knownMagnet(item); const hash = btih(link, item.hash);
      const box = el('div', 'magnet-box'); const head = el('div', 'magnet-box-head');
      head.append(el('span', 'magnet-tag', 'BTIH'), el('span', 'magnet-hash', hash || '来源未提供 BTIH 哈希'));
      const code = el('textarea', 'magnet-code'); code.readOnly = true; code.value = link; code.spellcheck = false; code.setAttribute('aria-label', '完整磁力链接');
      code.addEventListener('focus', () => code.select()); box.append(head, code);
      const copy = button('sheet-primary-btn', '复制磁力链接', () => copyMagnet(link, copy, code, item), 'copy');
      body.append(box, copy, el('p', 'sheet-caption', '仅检索并复制链接，使用前请自行核对来源与授权。'));
    }
  }

  function openMagnet(item, opener) {
    if (!canResolve(item) || !showSheet('magnet', opener, item)) return;
    if (knownMagnet(item)) renderMagnet('ready'); else resolveMagnet();
    overlay.querySelector('.bottom-sheet').focus();
  }

  async function resolveMagnet() {
    if (state.sheet?.type !== 'magnet') return;
    const sheetState = state.sheet; const item = sheetState.item;
    state.magnet?.abort(); const controller = new AbortController(); state.magnet = controller;
    renderMagnet('resolving');
    const url = new URL('/api/magnet', API);
    url.searchParams.set('src', String(item.src)); url.searchParams.set('detail_url', item.detail_url);
    try {
      const data = await getJson(url, controller, 20000);
      if (state.sheet !== sheetState || state.magnet !== controller) return;
      const link = knownMagnet(data);
      if (!link) throw new Error('来源暂未返回可用磁力链接，请重新获取。');
      item.magnet_url = link;
      const refocus = overlay.contains(document.activeElement);
      renderMagnet('ready'); if (refocus) overlay.querySelector('.bottom-sheet').focus();
      if (state.view === 'results') renderResults();
    } catch (error) {
      if (state.sheet !== sheetState || state.magnet !== controller) return;
      const refocus = overlay.contains(document.activeElement);
      renderMagnet('error', controller.signal.aborted ? '来源站点响应超时，请重新获取。' : error.message);
      if (refocus) overlay.querySelector('.bottom-sheet').focus();
    } finally { if (state.magnet === controller) state.magnet = null; }
  }

  async function copyMagnet(link, target, code, item) {
    if (!link || target.disabled) return;
    target.disabled = true;
    try {
      if (window.xb?.clipboard?.writeText && (!window.xb.can || window.xb.can('clipboard'))) await window.xb.clipboard.writeText(link);
      else if (navigator.clipboard?.writeText) await navigator.clipboard.writeText(link);
      else throw new Error('unsupported');
      target.classList.add('is-copied'); target.classList.remove('is-resolve');
      target.replaceChildren(icon('check'), el('span', null, '已复制')); toast('已复制磁力链接');
      setTimeout(() => { if (target.isConnected) { target.classList.remove('is-copied'); target.replaceChildren(icon('copy'), el('span', null, code ? '复制磁力链接' : '复制磁力')); } }, 1800);
    } catch {
      if (!code) {
        openMagnet(item, target);
        code = overlay.querySelector('.magnet-code');
      }
      code?.focus(); code?.select(); toast('复制未完成，链接已选中，请手动复制', true);
    } finally { target.disabled = false; }
  }

  let hostTheme = ['light', 'dark'].includes(window.xb?.info?.theme) ? window.xb.info.theme : null;
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  function applyTheme() { document.documentElement.dataset.theme = hostTheme || (media.matches ? 'dark' : 'light'); }
  media.addEventListener?.('change', applyTheme);
  window.xb?.on?.('theme', payload => {
    const value = typeof payload === 'string' ? payload : payload?.theme;
    hostTheme = ['light', 'dark'].includes(value) ? value : ['light', 'dark'].includes(window.xb?.info?.theme) ? window.xb.info.theme : null;
    applyTheme();
  });
  function updateEmbedding() { app.classList.toggle('is-embedded', getComputedStyle($('.masthead')).display === 'none'); }
  window.addEventListener('resize', updateEmbedding);
  window.addEventListener('pageshow', updateEmbedding);
  window.addEventListener('load', updateEmbedding);
  form.addEventListener('submit', event => { event.preventDefault(); runSearch(); });
  query.addEventListener('input', () => { clearQuery.hidden = !query.value; });
  clearQuery.addEventListener('click', () => { query.value = ''; clearQuery.hidden = true; query.focus(); });
  sourceButton.addEventListener('click', () => openSources(sourceButton));
  const sortTabs = [...document.querySelectorAll('[data-sort]')];
  for (const tab of sortTabs) {
    tab.addEventListener('click', () => { state.sort = tab.dataset.sort; updateToolbar(); if (state.view === 'results') renderResults(); });
    tab.addEventListener('keydown', event => {
      if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
      event.preventDefault();
      const index = event.key === 'Home' ? 0 : event.key === 'End' ? sortTabs.length - 1 : (sortTabs.indexOf(tab) + (event.key === 'ArrowRight' ? 1 : -1) + sortTabs.length) % sortTabs.length;
      sortTabs[index].click(); sortTabs[index].focus();
    });
  }
  overlay.addEventListener('click', event => { if (event.target === overlay) closeSheet(); });
  window.addEventListener('popstate', () => { if (state.sheet) dismissSheet(); });
  document.addEventListener('keydown', event => {
    if (!state.sheet) return;
    if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); closeSheet(); }
    if (event.key === 'Tab') {
      const controls = [...overlay.querySelectorAll('button:not(:disabled), textarea, [tabindex="0"]')];
      const first = controls[0], last = controls.at(-1);
      if (event.shiftKey && (document.activeElement === first || !controls.includes(document.activeElement))) { event.preventDefault(); last?.focus(); }
      else if (!event.shiftKey && (document.activeElement === last || !controls.includes(document.activeElement))) { event.preventDefault(); first?.focus(); }
    }
  });
  applyTheme(); updateEmbedding(); updateToolbar(); renderHome();
  loadHistory(); loadSources();
  window.xb?.ready?.();
})();
