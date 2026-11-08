(() => {
  'use strict';

  const API = 'https://search.xiangban.xyz';
  const sourceNames = ['Solid', 'YuHuaGe', '海盗湾', 'AcgRip', 'Zero', '1337', 'BtSow', 'TorLock', 'Kitty', '磁力猫', '磁力多'];
  const form = document.querySelector('#search-form');
  const query = document.querySelector('#query');
  const source = document.querySelector('#source');
  const submit = document.querySelector('#submit');
  const results = document.querySelector('#results');
  const modal = document.querySelector('#modal');
  const subtitle = document.querySelector('#modal-subtitle');
  const modalBody = document.querySelector('#modal-body');
  const toastBox = document.querySelector('#toast');
  const state = { keyword: '', page: 0, items: [], search: null, modal: null, request: 0, opener: null };
  let toastTimer;

  function el(tag, className, text) {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (text != null) element.textContent = String(text);
    return element;
  }

  function toast(message) {
    toastBox.textContent = message;
    toastBox.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toastBox.hidden = true; }, 2600);
  }

  async function getJson(url, controller, timeoutMs) {
    const timer = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const response = await fetch(url, { signal: controller.signal, credentials: 'omit', redirect: 'error', cache: 'no-store' });
      const text = await response.text();
      if (text.length > 2_000_000) throw new Error('响应过大');
      let data;
      try { data = JSON.parse(text); } catch { throw new Error('服务返回的内容无法识别'); }
      if (!response.ok) throw new Error(typeof data.message === 'string' ? data.message : '请求失败');
      return data;
    } finally {
      clearTimeout(timer);
    }
  }

  async function loadSources() {
    let sources = sourceNames.map((title, src) => ({ src, title }));
    try {
      const data = await getJson(`${API}/api/sources`, new AbortController(), 10000);
      if (Array.isArray(data.sources) && data.sources.length) sources = data.sources;
    } catch { /* Keep the published source IDs when the directory cannot be reached. */ }
    for (const entry of sources) {
      if (!Number.isInteger(entry.src) || entry.src < 0 || entry.src > 10) continue;
      const option = el('option', null, entry.title || sourceNames[entry.src]);
      option.value = String(entry.src);
      source.append(option);
    }
  }

  function fact(name, text) { return el('span', `fact ${name}`, text); }

  function card(item, index) {
    const article = el('article', 'card');
    const hasMagnet = typeof item.magnet_url === 'string' && item.magnet_url.startsWith('magnet:');
    const canResolve = hasMagnet || (Number.isInteger(item.src) && typeof item.detail_url === 'string' && /^https?:\/\//i.test(item.detail_url));
    if (canResolve) {
      article.tabIndex = 0;
      article.setAttribute('role', 'button');
      article.setAttribute('aria-label', `${item.title || '未命名条目'}，查看磁力链接`);
      article.addEventListener('click', event => {
        if (event.target.closest('button') || String(window.getSelection?.() || '')) return;
        openMagnet(item, article);
      });
      article.addEventListener('keydown', event => {
        if (event.target !== article || !['Enter', ' '].includes(event.key)) return;
        event.preventDefault();
        openMagnet(item, article);
      });
    }
    const head = el('div', 'card-head');
    head.append(el('h2', null, item.title || '未命名条目'), el('span', 'rank', `#${index + 1}`));
    const foot = el('div', 'card-foot');
    const facts = el('div', 'facts');
    facts.append(fact('source', sourceNames[item.src] || `来源 ${item.src}`));
    if (item.size) facts.append(fact('size', item.size));
    if (item.date) facts.append(fact('date', item.date));
    if (item.download_count) facts.append(fact('downloads', `下载 ${item.download_count}`));
    if (item.sender_count) facts.append(fact('seeds', `做种 ${item.sender_count}`));
    const action = el('button', 'card-action', hasMagnet ? '复制磁力' : '获取磁力');
    action.type = 'button';
    action.disabled = !canResolve;
    action.addEventListener('click', () => openMagnet(item, action));
    foot.append(facts, action);
    article.append(head, foot);
    return article;
  }

  function empty(title, detail, action) {
    const box = el('div', 'empty');
    box.append(el('div', 'empty-dot'), el('h2', null, title), el('p', null, detail));
    if (action) {
      const button = el('button', null, action);
      button.type = 'button';
      button.addEventListener('click', () => {
        if (action === '重试搜索') search(true);
        else query.focus();
      });
      box.append(button);
    }
    results.replaceChildren(box);
  }

  function setBusy(busy) {
    submit.disabled = busy;
    submit.textContent = busy ? '搜索中' : '搜索';
  }

  function addMoreButton() {
    const button = el('button', 'load-more', '加载下一页');
    button.type = 'button';
    button.addEventListener('click', () => loadPage(state.page + 1, button));
    results.append(button);
  }

  async function loadPage(page, moreButton = null) {
    if (state.search || page > 100) return;
    const controller = new AbortController();
    state.search = controller;
    const request = ++state.request;
    if (page === 1) { results.replaceChildren(); setBusy(true); }
    else { moreButton.disabled = true; moreButton.textContent = '加载中'; }
    const url = new URL('/api/search', API);
    url.searchParams.set('q', state.keyword);
    url.searchParams.set('page', String(page));
    url.searchParams.set('src', source.value);
    url.searchParams.set('filter', '1');
    url.searchParams.set('optimize', '1');
    try {
      const data = await getJson(url, controller, 65000);
      if (request !== state.request) return;
      const incoming = Array.isArray(data.items) ? data.items : [];
      const seen = new Set(state.items.map(item => item.hash || item.detail_url || `${item.src}:${item.title}`));
      const fresh = incoming.filter(item => {
        const key = item.hash || item.detail_url || `${item.src}:${item.title}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });
      if (page === 1) state.items = [];
      state.items.push(...fresh);
      state.page = page;
      moreButton?.remove();
      if (state.items.length === 0) empty('没有找到相关结果', '换一个关键词或搜索来源试试。', '修改关键词');
      else {
        const start = state.items.length - fresh.length;
        for (let i = 0; i < fresh.length; i++) results.append(card(fresh[i], start + i));
        if (incoming.length && page < 100) addMoreButton();
      }
    } catch (error) {
      if (request !== state.request || controller.signal.aborted && error?.name === 'AbortError' && !state.search) return;
      if (page === 1) empty('搜索暂时不可用', '请稍后重试，或选择其他来源。', '重试搜索');
      else { moreButton.disabled = false; moreButton.textContent = '重试加载'; }
    } finally {
      if (state.search === controller) state.search = null;
      if (page === 1 && request === state.request) setBusy(false);
    }
  }

  function search(force = false) {
    const keyword = query.value.trim();
    if (!keyword) { query.focus(); return; }
    if (state.search) { state.search.abort(); state.search = null; }
    state.request++;
    state.keyword = keyword;
    state.page = 0;
    state.items = [];
    if (force) toastBox.hidden = true;
    loadPage(1);
  }

  function closeModal() {
    if (modal.hidden) return;
    state.modal?.abort();
    state.modal = null;
    modal.hidden = true;
    modalBody.replaceChildren();
    const opener = state.opener;
    state.opener = null;
    if (opener?.isConnected) opener.focus();
  }

  function modalMessage(message, retry) {
    modalBody.replaceChildren(el('p', 'dialog-message', message));
    if (retry) {
      const button = el('button', 'dialog-action', '重试');
      button.type = 'button';
      button.addEventListener('click', retry);
      modalBody.append(button);
    }
  }

  function readyMagnet(link) {
    const area = el('textarea', 'magnet');
    area.readOnly = true;
    area.value = link;
    area.setAttribute('aria-label', '磁力链接');
    const button = el('button', 'dialog-action', '复制磁力链接');
    button.type = 'button';
    button.addEventListener('click', async () => {
      try {
        if (window.xb?.can?.('clipboard')) await window.xb.clipboard.writeText(link);
        else if (navigator.clipboard?.writeText) await navigator.clipboard.writeText(link);
        else throw new Error('unsupported');
        toast('已复制磁力链接');
      } catch {
        area.focus();
        area.select();
        toast('链接已选中，请手动复制');
      }
    });
    modalBody.replaceChildren(area, button);
    area.focus();
    area.select();
  }

  async function openMagnet(item, opener) {
    state.opener = opener;
    subtitle.textContent = item.title || '未命名条目';
    modal.hidden = false;
    const known = typeof item.magnet_url === 'string' && item.magnet_url.startsWith('magnet:') ? item.magnet_url : null;
    if (known) { readyMagnet(known); return; }
    if (!Number.isInteger(item.src) || !item.detail_url) { modalMessage('暂无可用磁力链接'); return; }
    state.modal?.abort();
    const controller = new AbortController();
    state.modal = controller;
    modalMessage('正在获取磁力链接…');
    const url = new URL('/api/magnet', API);
    url.searchParams.set('src', String(item.src));
    url.searchParams.set('detail_url', item.detail_url);
    try {
      const data = await getJson(url, controller, 20000);
      if (modal.hidden || state.modal !== controller) return;
      if (typeof data.magnet_url !== 'string' || !data.magnet_url.startsWith('magnet:')) throw new Error('MAGNET_UNAVAILABLE');
      item.magnet_url = data.magnet_url;
      readyMagnet(data.magnet_url);
    } catch (error) {
      if (modal.hidden || state.modal !== controller) return;
      modalMessage(error?.message?.includes('MAGNET_UNAVAILABLE') ? '暂无可用磁力链接' : '获取失败，请稍后重试。', () => openMagnet(item, opener));
    } finally {
      if (state.modal === controller) state.modal = null;
    }
  }

  form.addEventListener('submit', event => { event.preventDefault(); search(); });
  source.addEventListener('change', () => { if (query.value.trim()) search(); });
  modal.addEventListener('click', event => { if (event.target.hasAttribute('data-close')) closeModal(); });
  document.addEventListener('keydown', event => { if (event.key === 'Escape' && !modal.hidden) { event.stopPropagation(); closeModal(); } });
  loadSources();
  window.xb?.ready?.();
})();
