(() => {
  'use strict';

  const API = 'https://search.xiangban.xyz';
  const MAX_BYTES = 2_000_000;
  const MAX_PAGE = 100;
  const STORAGE_KEY = 'wancisou.recent-searches';
  const sources = [
    ['Solid', '综合'], ['YuHuaGe', '中文'], ['海盗湾', '国际'], ['AcgRip', '动漫'],
    ['Zero', '综合'], ['1337', '国际'], ['BtSow', '综合'], ['TorLock', '国际'],
    ['Kitty', '综合'], ['磁力猫', '中文'], ['磁力多', '中文'],
  ].map(([title, role]) => ({ title, role }));

  const $ = selector => document.querySelector(selector);
  const byId = id => document.getElementById(id);
  const query = byId('query');
  const submit = byId('submit');
  const sourceButton = byId('source');
  const home = byId('home');
  const results = byId('results');
  const list = byId('result-list');
  const meta = byId('result-meta');
  const pager = byId('load-more');
  const overlay = byId('overlay');
  const sheet = byId('sheet');
  const sheetBody = byId('sheet-body');
  const toastBox = byId('toast');
  const sortButtons = [...document.querySelectorAll('[data-sort]')];
  let source = 'all';
  let sort = 'relevance';
  let search = null;
  let activeSheet = null;
  let sheetSequence = 0;
  let toastTimer;

  const paths = {
    magnet: 'M6 5v8a6 6 0 0 0 12 0V5M6 8.5h3m6 0h3',
    search: 'M17.5 10.5a7 7 0 1 0-14 0 7 7 0 0 0 14 0ZM16 16l4.5 4.5',
    close: 'm6 6 12 12M18 6 6 18',
    chevron: 'm6 9 6 6 6-6',
    film: 'M4 5h16v14H4zM8 5v14M16 5v14M4 10h4m8 0h4M4 14h4m8 0h4',
    screen: 'M3 6h18v12H3zM8 21h8m-7-18 3 3 3-3',
    box: 'm12 3 8 4.5v9L12 21l-8-4.5v-9L12 3Zm-8 4.5L12 12l8-4.5M12 12v9',
    music: 'M9 17V6l10-2v11M9 17a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Zm10-2a2.5 2.5 0 1 0-5 0 2.5 2.5 0 0 0 5 0Z',
    copy: 'M9 9h11v11H9zM15 5V4H4v11h1',
    link: 'M10 13a4 4 0 0 0 6 .5l3-3a4 4 0 0 0-5.5-5.8l-1.5 1.5m2 4.8a4 4 0 0 0-6-.5l-3 3a4 4 0 0 0 5.5 5.8l1.5-1.5',
    check: 'm5 12 4.5 4.5L19 7',
    alert: 'M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0ZM12 7v6m0 3h.01',
    clock: 'M21 12a9 9 0 1 0-18 0 9 9 0 0 0 18 0ZM12 7v5l3 2',
    empty: 'M17 10.5a6.5 6.5 0 1 0-13 0 6.5 6.5 0 0 0 13 0Zm-2 4.5 5 5M8 8l5 5m0-5-5 5',
    hot: 'M12 2.5c1.1 4.7-6 6.2-6 12.3a6 6 0 0 0 12 0c0-4.2-2.9-7.9-6-12.3Zm0 10.5c-1.7 1.8-2.4 2.6-2.4 4a2.4 2.4 0 0 0 4.8 0c0-1.4-.7-2.2-2.4-4Z',
    spinner: 'M12 3a9 9 0 1 0 9 9',
  };

  function node(tag, className, text) {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (text !== undefined) element.textContent = text;
    return element;
  }

  function icon(name) {
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    const path = document.createElementNS(ns, 'path');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('class', name === 'spinner' ? 'icon spin' : 'icon');
    svg.setAttribute('aria-hidden', 'true');
    path.setAttribute('d', paths[name]);
    svg.append(path);
    return svg;
  }

  function labelButton(button, label, glyph) {
    button.replaceChildren();
    if (glyph) button.append(icon(glyph));
    button.append(node('span', '', label));
  }

  function button(className, label, action, glyph) {
    const control = node('button', className);
    control.type = 'button';
    if (label) labelButton(control, label, glyph);
    control.addEventListener('click', () => action(control));
    return control;
  }

  function message(text, error = false) {
    toastBox.classList.toggle('is-error', error);
    toastBox.replaceChildren(icon(error ? 'alert' : 'check'), node('span', '', text));
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => toastBox.replaceChildren(), 2600);
  }

  async function getJson(path, params, controller, timeout) {
    const url = new URL(path, API);
    url.search = new URLSearchParams(params).toString();
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      controller.abort();
    }, timeout);

    try {
      const response = await fetch(url, {
        credentials: 'omit', redirect: 'error', cache: 'no-store', signal: controller.signal,
      });
      if (!response.ok) {
        controller.abort();
        throw new Error(response.status === 429 ? '请求较频繁，请稍后重试。' : '搜索服务暂时不可用。');
      }
      if (Number(response.headers.get('Content-Length')) > MAX_BYTES) {
        controller.abort();
        throw new Error('服务响应超过 2 MB，可切换为单一来源后重试。');
      }

      const reader = response.body?.getReader();
      if (!reader) throw new Error('服务返回了空响应。');
      const decoder = new TextDecoder();
      let bytes = 0;
      let body = '';
      try {
        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          bytes += value.byteLength;
          if (bytes > MAX_BYTES) {
            controller.abort();
            throw new Error('服务响应超过 2 MB，可切换为单一来源后重试。');
          }
          body += decoder.decode(value, { stream: true });
        }
        body += decoder.decode();
      } finally {
        reader.releaseLock();
      }

      try { return JSON.parse(body); }
      catch { throw new Error('服务返回的内容无法识别。'); }
    } catch (error) {
      if (timedOut) throw new DOMException('来源响应超时，请稍后重试。', 'TimeoutError');
      if (error instanceof TypeError) throw new Error('暂时无法连接搜索服务。');
      throw error;
    } finally {
      clearTimeout(timer);
    }
  }

  const text = value => typeof value === 'string' ? value.trim() : '';
  const sourceName = id => id === 'all' ? '全部来源' : sources[id]?.title || '未知来源';

  function sourceId(value) {
    const id = typeof value === 'string' && /^(?:[0-9]|10)$/.test(value) ? Number(value) : value;
    return Number.isInteger(id) && id >= 0 && id < sources.length ? id : -1;
  }

  function siteName(id, className = 'site-name') {
    const name = node('span', className);
    const dot = node('span', `site-dot site-${id}`);
    dot.setAttribute('aria-hidden', 'true');
    const title = node('span', 'truncate', sourceName(id));
    title.dataset.site = String(id);
    name.append(dot, title);
    return name;
  }

  function syncSource() {
    byId('source-label').textContent = sourceName(source);
    const dot = byId('source-dot');
    dot.className = `site-dot site-${source}`;
    dot.hidden = source === 'all';
    sourceButton.classList.toggle('is-filtered', source !== 'all');
    sourceButton.setAttribute('aria-label', `选择搜索来源，当前为${sourceName(source)}`);
    for (const control of document.querySelectorAll('[data-source]')) {
      control.setAttribute('aria-pressed', String(String(source) === control.dataset.source));
    }
  }

  async function loadSources() {
    try {
      const data = await getJson('/api/sources', {}, new AbortController(), 10_000);
      if (!Array.isArray(data?.sources)) return;
      for (const entry of data.sources) {
        const id = sourceId(entry?.src);
        if (id !== -1 && text(entry.title)) sources[id].title = text(entry.title);
      }
      for (const label of document.querySelectorAll('[data-site]')) {
        label.textContent = sourceName(label.dataset.site);
      }
      syncSource();
    } catch {
      // The built-in directory already covers the 11 public source IDs.
    }
  }

  function makeSourceGrid(grid, showRoles) {
    sources.forEach(({ role }, id) => {
      const control = button('source-option', '', () => chooseSource(id));
      control.dataset.source = String(id);
      control.append(siteName(id));
      if (showRoles) control.append(node('span', 'source-sub', role));
      grid.append(control);
    });
  }

  function chooseSource(id) {
    const changed = source !== id;
    source = id;
    syncSource();
    closeSheet();
    if (changed && search) startSearch(search.keyword);
  }

  let recent = [];
  let historyCleared = false;
  let historyWrites = Promise.resolve();

  function renderRecent() {
    const region = byId('recent');
    region.replaceChildren();
    byId('clear-history').hidden = !recent.length;
    if (!recent.length) region.append(node('p', '', '搜索后，关键词会保存在此设备。'));
    for (const keyword of recent) {
      const chip = button('soft recent-chip', keyword, () => startSearch(keyword));
      chip.lastElementChild.className = 'truncate';
      region.append(chip);
    }
  }

  function cleanHistory(value) {
    return [...new Set(value.map(word => text(word).slice(0, 128)).filter(Boolean))].slice(0, 8);
  }

  async function readHistory() {
    try {
      const saved = await window.xb?.storage?.get?.(STORAGE_KEY);
      if (Array.isArray(saved)) return saved;
    } catch {}
    try {
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY));
      if (Array.isArray(saved)) return saved;
    } catch {}
    return [];
  }

  async function writeHistory(snapshot) {
    try {
      if (window.xb?.storage?.set) {
        await window.xb.storage.set(STORAGE_KEY, snapshot);
        try { localStorage.removeItem(STORAGE_KEY); } catch {}
        return;
      }
    } catch {}
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot)); } catch {}
  }

  function updateHistory(keyword) {
    if (!keyword) historyCleared = true;
    recent = keyword ? cleanHistory([keyword, ...recent]) : [];
    renderRecent();
    // Writes wait for hydration and finish in order, including clear-then-search.
    historyWrites = historyWrites.then(() => writeHistory([...recent]));
  }

  function count(value) {
    if (typeof value === 'number') return Number.isFinite(value) ? Math.max(0, value) : 0;
    const match = text(value).replaceAll(',', '').match(/^(\d+(?:\.\d+)?)\s*([kmb万亿])?/i);
    const amount = match && Number(match[1]) * ({ k: 1e3, m: 1e6, b: 1e9, 万: 1e4, 亿: 1e8 }[match[2]?.toLowerCase()] || 1);
    return Number.isFinite(amount) ? amount : 0;
  }

  function sizeValue(value) {
    if (typeof value === 'number') return Number.isFinite(value) ? Math.max(0, value) : 0;
    const match = text(value).replaceAll(',', '').match(/^(\d+(?:\.\d+)?)\s*([KMGT]?)(?:I?B|字节)?$/i);
    if (!match) return 0;
    const power = match[2] ? 'KMGT'.indexOf(match[2].toUpperCase()) + 1 : 0;
    const bytes = Number(match[1]) * 1024 ** power;
    return Number.isFinite(bytes) ? bytes : 0;
  }

  function dateValue(value) {
    const stamp = typeof value === 'number'
      ? value * (Math.abs(value) < 1e12 ? 1000 : 1)
      : Date.parse(text(value).replace(/年|月/g, '-').replace(/日/g, ''));
    return Number.isFinite(stamp) && Math.abs(stamp) <= 8.64e15 ? stamp : 0;
  }

  function magnetLink(value) {
    const link = text(value);
    return /^magnet:\?\S+$/i.test(link) ? link : '';
  }

  function hasDetail(item) {
    return item.src !== -1 && /^https?:\/\//i.test(item.detail);
  }

  function result(raw) {
    if (!raw || typeof raw.title !== 'string') return null;
    const date = dateValue(raw.date);
    return {
      title: text(raw.title) || '未命名资源', src: sourceId(raw.src),
      size: typeof raw.size === 'number' ? `${sizeValue(raw.size)} B` : text(raw.size),
      bytes: sizeValue(raw.size), date, dateLabel: text(raw.date) || (date ? new Date(date).toISOString().slice(0, 10) : ''),
      seeds: raw.sender_count == null ? null : count(raw.sender_count),
      downloads: raw.download_count == null ? null : count(raw.download_count),
      hash: text(raw.hash), detail: text(raw.detail_url), link: magnetLink(raw.magnet_url),
    };
  }

  const number = new Intl.NumberFormat('zh-CN', { maximumFractionDigits: 1 });
  function facts(item, summary = false) {
    const row = node('div', 'facts');
    const tone = item.seeds >= 10 ? 'high' : item.seeds >= 1 ? 'mid' : 'low';
    if (item.size) row.append(node('span', 'badge size', item.size));
    row.append(node('span', `badge seed ${tone}`, `做种 ${item.seeds == null ? '未知' : number.format(item.seeds)}`));
    if (!summary && item.downloads !== null) row.append(node('span', '', `下载 ${number.format(item.downloads)}`));
    if (!summary && item.dateLabel) row.append(node('span', '', item.dateLabel));
    return row;
  }

  function updateCardAction(item) {
    if (!item.action) return;
    item.action.classList.toggle('resolve', !item.link);
    item.action.disabled = !item.link && !hasDetail(item);
    labelButton(item.action, item.link ? '复制磁力' : '获取磁力', item.link ? 'copy' : 'link');
  }

  function makeCard(item) {
    const card = node('article', 'result-card');
    const top = node('div', 'result-top');
    const title = node('h3');
    const titleButton = button('result-title', item.title, control => openMagnet(item, control));
    titleButton.disabled = !item.link && !hasDetail(item);
    titleButton.lastElementChild.dir = 'auto';
    title.append(titleButton);
    top.append(title);
    if (item.seeds >= 20) {
      const hot = node('span', 'badge hot');
      hot.append(icon('hot'), node('span', '', '热种'));
      top.append(hot);
    }
    const foot = node('div', 'result-foot');
    item.action = button('soft card-action', '', control => {
      if (item.link) void copyMagnet(item, control);
      else openMagnet(item, control);
    });
    updateCardAction(item);
    foot.append(siteName(item.src, 'site-name source-tag'), item.action);
    card.append(top, facts(item), foot);
    return card;
  }

  function skeleton() {
    const card = node('div', 'result-card skeleton');
    card.setAttribute('aria-hidden', 'true');
    const foot = node('div', 'result-foot');
    foot.append(node('span', 'skeleton-line'), node('span', 'skeleton-line'));
    card.append(node('span', 'skeleton-line'), node('span', 'skeleton-line short'), node('span', 'skeleton-line stats'), foot);
    return card;
  }

  function resultStatus(label) {
    meta.textContent = label;
    byId('status').textContent = label;
  }

  function drawResults(current) {
    const rows = [...current.items];
    const field = { size: 'bytes', seeds: 'seeds', date: 'date' }[sort];
    if (field) rows.sort((a, b) => b[field] - a[field]);
    const content = document.createDocumentFragment();
    for (const item of rows) content.append(item.card ||= makeCard(item));
    list.replaceChildren(content);
    resultStatus(`已加载 ${number.format(rows.length)} 条 · ${sourceName(current.source)}`);
  }

  function drawState(error) {
    const timedOut = error?.name === 'TimeoutError';
    const heading = timedOut ? '来源响应超时' : error ? '搜索暂时不可用' : '没有找到相关结果';
    const panel = node('div', 'state-panel');
    const actions = node('div', 'state-actions');
    if (error) {
      actions.append(
        button('primary', '重试搜索', () => startSearch(search.keyword)),
        button('soft', '切换来源', control => openSources(control)),
      );
    } else {
      actions.append(button('primary', '修改关键词', () => { query.focus(); query.select(); }));
      if (source !== 'all') actions.append(button('soft', '搜全部来源', () => chooseSource('all')));
    }
    const detail = error?.message || '试试更简短的关键词，或选择其他来源。';
    panel.append(icon(timedOut ? 'clock' : error ? 'alert' : 'empty'), node('h2', '', heading), node('p', '', detail), actions);
    list.replaceChildren(panel);
    resultStatus(heading);
  }

  function syncSearchButton() {
    byId('clear-query').hidden = !query.value;
    const busy = Boolean(search?.controller && !search.page && query.value.trim() === search.keyword);
    if (busy === (submit.getAttribute('aria-disabled') === 'true')) return;
    submit.setAttribute('aria-disabled', String(busy));
    labelButton(submit, busy ? '搜索中' : '搜索', busy ? 'spinner' : null);
  }

  function syncPager(current) {
    pager.hidden = !current?.hasMore;
    pager.setAttribute('aria-disabled', String(Boolean(current?.controller)));
    pager.textContent = current?.controller ? '正在加载…' : current?.moreError ? '重试加载下一页' : '加载下一页';
  }

  async function loadPage(page) {
    const current = search;
    if (!current || current.controller || page > MAX_PAGE) return;
    const controller = new AbortController();
    current.controller = controller;
    current.moreError = false;
    syncSearchButton();
    syncPager(current);
    list.setAttribute('aria-busy', 'true');
    if (page === 1) {
      resultStatus(`正在搜索「${current.keyword}」…`);
      list.replaceChildren(...Array.from({ length: 4 }, skeleton));
    }

    try {
      const data = await getJson('/api/search', {
        q: current.keyword, page, src: current.source, filter: 1, optimize: 1,
      }, controller, 65_000);
      if (search !== current) return;
      if (!Array.isArray(data?.items)) throw new Error('搜索结果格式异常，请重试。');
      if (!data.items.length && Array.isArray(data.sources) && data.sources.length
        && data.sources.every(entry => entry?.ok === false)) {
        throw new Error('所选来源暂未响应，请重试或切换来源。');
      }
      for (const raw of data.items) {
        const item = result(raw);
        if (!item) continue;
        const key = item.hash || item.detail || `${item.src}:${item.title}`;
        if (current.seen.has(key)) continue;
        current.seen.add(key);
        current.items.push(item);
      }
      current.page = page;
      current.hasMore = page < MAX_PAGE && data.items.length > 0 && data.has_more !== false;
      if (current.items.length) drawResults(current);
      else {
        current.hasMore = false;
        drawState();
      }
    } catch (error) {
      if (search !== current) return;
      if (page === 1) drawState(error);
      else {
        current.moreError = true;
        message('下一页加载失败，请重试', true);
      }
    } finally {
      if (search === current) {
        current.controller = null;
        list.setAttribute('aria-busy', 'false');
        syncSearchButton();
        syncPager(current);
      }
    }
  }

  function startSearch(raw) {
    const keyword = text(raw).slice(0, 128);
    if (!keyword) {
      message('请输入搜索关键词', true);
      query.focus();
      return;
    }
    if (search?.controller && search.keyword === keyword && search.source === source) return;
    search?.controller?.abort();
    search = { keyword, source, page: 0, items: [], seen: new Set(), hasMore: false };
    query.value = keyword;
    query.blur();
    home.hidden = true;
    results.hidden = false;
    byId('scroll-area').scrollTop = 0;
    updateHistory(keyword);
    void loadPage(1);
  }

  function showSheet(type, title, opener) {
    if (activeSheet) return null;
    const view = { type, opener, id: ++sheetSequence };
    try {
      history.pushState({ ...history.state, wancisouSheet: view.id }, '');
      view.pushed = true;
    } catch {}
    activeSheet = view;
    byId('sheet-title').textContent = title;
    byId('cancel-sheet').textContent = type === 'sources' ? '取消' : '关闭';
    sourceButton.setAttribute('aria-expanded', String(type === 'sources'));
    byId('workspace').inert = true;
    overlay.hidden = false;
    sheet.focus({ preventScroll: true });
    return view;
  }

  function dismissSheet() {
    if (!activeSheet) return;
    const { controller, opener } = activeSheet;
    activeSheet = null;
    controller?.abort();
    overlay.hidden = true;
    sheetBody.replaceChildren();
    byId('workspace').inert = false;
    sourceButton.setAttribute('aria-expanded', 'false');
    (opener?.isConnected ? opener : sourceButton).focus({ preventScroll: true });
  }

  function closeSheet() {
    if (!activeSheet || activeSheet.closing) return;
    if (activeSheet.pushed && history.state?.wancisouSheet === activeSheet.id) {
      activeSheet.closing = true;
      activeSheet.controller?.abort();
      try { history.back(); return; } catch {}
    }
    dismissSheet();
  }

  function openSources(opener) {
    if (!showSheet('sources', '选择搜索来源', opener)) return;
    const all = button('source-all', '', () => chooseSource('all'));
    all.dataset.source = 'all';
    const labels = node('span');
    labels.append(node('strong', '', '全部来源'), node('span', 'source-sub', '聚合 11 个公开索引站'));
    all.append(icon('magnet'), labels);
    const grid = node('div', 'source-grid');
    makeSourceGrid(grid, true);
    sheetBody.replaceChildren(all, grid);
    syncSource();
  }

  function hashHex(value) {
    if (/^[a-f0-9]{40}$/i.test(value)) return value.toUpperCase();
    if (!/^[a-z2-7]{32}$/i.test(value)) return '';
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    let buffer = 0;
    let bits = 0;
    let hex = '';
    for (const char of value.toUpperCase()) {
      buffer = (buffer << 5) | alphabet.indexOf(char);
      bits += 5;
      if (bits >= 8) {
        bits -= 8;
        hex += ((buffer >>> bits) & 255).toString(16).padStart(2, '0');
      }
    }
    return hex.toUpperCase();
  }

  function btih(item) {
    const exactTopics = new URL(item.link).searchParams.getAll('xt');
    for (const topic of exactTopics) {
      if (/^urn:btih:/i.test(topic)) {
        const hex = hashHex(topic.slice(9));
        if (hex) return hex;
      }
    }
    return hashHex(item.hash);
  }

  function drawMagnet(view, error) {
    const { item } = view;
    const title = node('h3', 'sheet-item-title', item.title);
    title.dir = 'auto';
    const restoreFocus = sheetBody.contains(document.activeElement);
    sheetBody.replaceChildren(title, facts(item, true));
    if (restoreFocus) sheet.focus({ preventScroll: true });

    if (!item.link) {
      const note = node('div', error ? 'sheet-note error' : 'sheet-note');
      note.setAttribute('role', 'status');
      note.append(icon(error ? 'alert' : 'spinner'), node('span', '', error?.message || `正在从 ${sourceName(item.src)} 获取磁力链接…`));
      sheetBody.append(note);
      if (error) sheetBody.append(button('primary sheet-primary', '重新获取', () => resolveMagnet(view)));
      return null;
    }

    const box = node('div', 'code-box');
    const hash = node('div', 'hash-row');
    hash.append(node('span', 'hash-tag', 'BTIH'), node('code', '', btih(item) || '来源未提供 BTIH 哈希'));
    const code = node('textarea', 'magnet-link', item.link);
    code.readOnly = true;
    code.spellcheck = false;
    code.setAttribute('aria-label', '完整磁力链接');
    code.addEventListener('focus', () => code.select());
    box.append(hash, code);
    sheetBody.append(box, button('primary sheet-primary', '复制磁力链接', control => copyMagnet(item, control, code), 'copy'));
    return code;
  }

  async function resolveMagnet(view) {
    if (activeSheet !== view || view.closing || view.controller) return;
    const controller = new AbortController();
    view.controller = controller;
    drawMagnet(view);
    try {
      const data = await getJson('/api/magnet', {
        src: view.item.src, detail_url: view.item.detail,
      }, controller, 20_000);
      if (activeSheet !== view || view.closing) return;
      const link = magnetLink(data?.magnet_url);
      if (!link) throw new Error('来源未返回可用的磁力链接。');
      view.item.link = link;
      updateCardAction(view.item);
      drawMagnet(view);
    } catch (error) {
      if (activeSheet === view && !view.closing) drawMagnet(view, error);
    } finally {
      view.controller = null;
    }
  }

  function openMagnet(item, opener) {
    if (!item.link && !hasDetail(item)) return null;
    const view = showSheet('magnet', '磁力详情', opener);
    if (!view) return null;
    view.item = item;
    if (item.link) return drawMagnet(view);
    void resolveMagnet(view);
    return null;
  }

  async function copyMagnet(item, control, code) {
    if (control.getAttribute('aria-disabled') === 'true') return;
    control.setAttribute('aria-disabled', 'true');
    try {
      if (window.xb?.can?.('clipboard') && window.xb.clipboard?.writeText) {
        await window.xb.clipboard.writeText(item.link);
      } else {
        await navigator.clipboard.writeText(item.link);
      }
      if (!control.isConnected) return;
      control.classList.add('is-copied');
      labelButton(control, '已复制', 'check');
      message('已复制磁力链接');
      setTimeout(() => {
        control.classList.remove('is-copied');
        labelButton(control, code ? '复制磁力链接' : '复制磁力', 'copy');
        control.removeAttribute('aria-disabled');
      }, 1700);
    } catch {
      control.removeAttribute('aria-disabled');
      if (!control.isConnected) return;
      const field = code || openMagnet(item, control);
      field?.focus();
      field?.select();
      message(field ? '链接已选中，请手动复制' : '复制失败，可打开详情手动复制', true);
    }
  }

  const dark = matchMedia('(prefers-color-scheme: dark)');
  let hostTheme = window.xb?.info?.theme;
  function applyTheme() {
    document.documentElement.dataset.theme = ['light', 'dark'].includes(hostTheme)
      ? hostTheme : dark.matches ? 'dark' : 'light';
  }
  dark.addEventListener('change', applyTheme);
  window.xb?.on?.('theme', payload => {
    hostTheme = (typeof payload === 'string' ? payload : payload?.theme) || window.xb?.info?.theme;
    applyTheme();
  });

  function syncEmbedding() {
    $('.app').classList.toggle('is-embedded', getComputedStyle($('.masthead')).display === 'none');
  }

  byId('search-form').addEventListener('submit', event => {
    event.preventDefault();
    startSearch(query.value);
  });
  query.addEventListener('input', syncSearchButton);
  byId('clear-query').addEventListener('click', () => {
    search?.controller?.abort();
    search = null;
    query.value = '';
    home.hidden = false;
    results.hidden = true;
    list.replaceChildren();
    list.removeAttribute('aria-busy');
    resultStatus('');
    syncSearchButton();
    byId('scroll-area').scrollTop = 0;
    query.focus();
  });

  for (const scene of document.querySelectorAll('[data-query]')) {
    scene.addEventListener('click', () => startSearch(scene.dataset.query));
  }
  byId('clear-history').addEventListener('click', () => {
    updateHistory();
    byId('recent-title').focus();
    message('已清空最近搜索');
  });
  for (const control of sortButtons) {
    control.addEventListener('click', () => {
      sort = control.dataset.sort;
      for (const option of sortButtons) option.setAttribute('aria-pressed', String(option === control));
      if (search?.items.length) drawResults(search);
    });
  }
  pager.addEventListener('click', () => { if (search?.hasMore) void loadPage(search.page + 1); });
  sourceButton.addEventListener('click', () => openSources(sourceButton));
  byId('close-sheet').addEventListener('click', closeSheet);
  byId('cancel-sheet').addEventListener('click', closeSheet);
  overlay.addEventListener('click', event => { if (event.target === overlay) closeSheet(); });
  window.addEventListener('popstate', dismissSheet);
  window.addEventListener('keydown', event => {
    if (!activeSheet) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopImmediatePropagation();
      closeSheet();
    }
    if (event.key === 'Tab') {
      const controls = [...sheet.querySelectorAll('button:not(:disabled), textarea')];
      const active = document.activeElement;
      const boundary = event.shiftKey ? controls[0] : controls.at(-1);
      if (active === boundary || !controls.includes(active)) {
        event.preventDefault();
        (event.shiftKey ? controls.at(-1) : controls[0])?.focus();
      }
    }
  }, true);

  const grip = byId('sheet-grip');
  let gripY = 0;
  grip.addEventListener('pointerdown', event => {
    if (!event.isPrimary || event.button !== 0) return;
    gripY = event.clientY;
    grip.setPointerCapture(event.pointerId);
  });
  grip.addEventListener('pointerup', event => {
    if (grip.hasPointerCapture(event.pointerId) && event.clientY - gripY > 36) closeSheet();
  });

  for (const marker of document.querySelectorAll('[data-icon]')) marker.replaceWith(icon(marker.dataset.icon));
  applyTheme();
  syncEmbedding();
  window.addEventListener('load', syncEmbedding);
  makeSourceGrid(byId('home-sources'), false);
  syncSource();
  renderRecent();
  historyWrites = readHistory().then(saved => {
    if (!historyCleared) recent = cleanHistory([...recent, ...saved]);
    renderRecent();
  });
  void loadSources();
  requestAnimationFrame(() => window.xb?.ready?.());
})();
