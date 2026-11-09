'use strict';

const glyphs = {
  plus: '<path d="M12 5v14M5 12h14"/>',
  minus: '<path d="M5 12h14"/>',
  right: '<path d="m9 6 6 6-6 6"/>',
  left: '<path d="m15 6-6 6 6 6"/>',
  up: '<path d="m6 15 6-6 6 6"/>',
  down: '<path d="m6 9 6 6 6-6"/>',
  power: '<path d="M12 2.5v9m4.8-6.6a8 8 0 1 1-9.6 0"/>',
  home: '<path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1z"/>',
  menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  back: '<path d="m8 4-5 5 5 5M3 9h12a5 5 0 1 1 0 10h-5"/>',
  volumeDown: '<path d="M3 9v6h4l5 4V5L7 9zm13 3h6"/>',
  volumeUp: '<path d="M3 9v6h4l5 4V5L7 9zm13 3h6m-3-3v6"/>',
  dot: '<circle cx="12" cy="12" r="3.3" fill="currentColor" stroke="none"/>',
  more: '<path d="M5 12h.01m6.99 0h.01m6.99 0h.01" stroke-width="3"/>',
  cool: '<path d="M12 2v20M3.4 7l17.2 10M3.4 17 20.6 7M9 4l3 3 3-3M9 20l3-3 3 3"/>',
  dry: '<path d="M12 3c4 5 7 8.8 7 12a7 7 0 0 1-14 0c0-3.2 3-7 7-12Z"/>',
  heat: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M2 12h2m16 0h2M5 5l2 2m10 10 2 2M5 19l2-2M17 7l2-2"/>',
  wind: '<path d="M3 8h12a2 2 0 1 0-2-2M2 12h17a2 2 0 1 1-2 2M4 16h8a2 2 0 1 1-2 2"/>',
  direction: '<path d="m4 13 8-7 8 7m-16 2a12 12 0 0 0 16 0"/>',
  edit: '<path d="m15 5 4 4M4 20l4-.8L20 7.2a1.7 1.7 0 0 0 0-2.4l-.8-.8a1.7 1.7 0 0 0-2.4 0L4.8 16Z"/>',
  remote: '<rect x="7" y="2" width="10" height="20" rx="4"/><circle cx="12" cy="8" r="1.5"/><path d="M11 14h2m-2 3h2"/>',
};
const host = window.xb;
const app = document.querySelector('#app');
const sheet = document.querySelector('#sheet');
const notice = document.querySelector('#notice');
const typeLabels = { tv: '电视', ac: '空调' };
const rankedBrands = {
  tv: ['海信', '创维', 'TCL', '小米', '长虹', '海尔', '康佳', '华为'],
  ac: ['格力', '美的', '小米', '海尔', '海信', '奥克斯'],
};
const aliases = {
  小米: 'xiaomi 米家 mijia', 红米: 'redmi', 华为: 'huawei 智慧屏', 雷鸟: 'ffalcon',
  美的: 'midea', 格力: 'gree', 海尔: 'haier', 海信: 'hisense', 奥克斯: 'aux',
  华凌: 'wahin', 科龙: 'kelon', 统帅: 'leader', 大金: 'daikin', 长虹: 'changhong',
};
const modes = { cool: '制冷', heat: '制热', dry: '除湿', air: '送风', auto: '自动' };
const modeIcons = { cool: 'cool', heat: 'heat', dry: 'dry', air: 'wind', auto: 'direction' };
const fanNames = { auto: '自动', low: '低', mid: '中', medium: '中', high: '高' };
const errors = {
  unsupported: '此手机或码集不支持这项操作',
  denied: '手机系统拒绝发射红外',
  gesture: '请再点一次按键',
  busy: '请等当前操作完成',
};
const escape = value => String(value).replace(/[&<>"']/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[ch]);
const icon = name => `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${glyphs[name]}</svg>`;
const art = type => `<svg class="art" viewBox="0 0 96 64" aria-hidden="true"><use href="#${type}-art"/></svg>`;
const home = { page: 'home' };

let route = home;
let available = null;
let readingDevices = true;
let devicesReadFailed = false;
let readingCatalog = true;
let sending = false;
let saving = false;
let tested = false;
let testedAcState = null;
let catalog = null;
let devices = [];
let noticeTimer;
const profiles = new Map();
const latest = new Map();
const acStates = new Map();

function toast(message) {
  clearTimeout(noticeTimer);
  notice.textContent = message;
  notice.hidden = false;
  noticeTimer = setTimeout(() => { notice.hidden = true; }, 1800);
}

function setCatalog(value) {
  catalog = value;
  profiles.clear();
  for (const type of Object.keys(typeLabels)) {
    for (const brand of value?.[type] || []) {
      for (const profile of brand.profiles) profiles.set(profile.id, profile);
    }
  }
}

function findBrand() {
  return catalog?.[route.type]?.find(brand => brand.name === route.brand);
}

function findDevice() {
  return devices.find(item => item.id === route.id);
}

function go(next, replace = false) {
  history[replace ? 'replaceState' : 'pushState']({ route: next }, '');
  route = next;
  tested = false;
  render();
}

function steps(active) {
  return `<div class="steps" aria-label="添加进度">${['选设备', '选品牌', '试码'].map((name, index) =>
    `<span ${index === active ? 'aria-current="step"' : ''}>${index + 1} ${name}</span>`).join('')}</div>`;
}

function header(title, isHome = false) {
  if (isHome) return `<header class="top home"><h1>遥控器</h1>
    <button class="add" data-action="add" ${!available || readingDevices || devicesReadFailed ? 'disabled' : ''}><span>${icon('plus')}添加</span></button></header>`;
  return `<header class="top"><button class="back" data-action="back" aria-label="返回上一页">${icon('left')}</button>
    <h1>${escape(title)}</h1></header>`;
}

function renderHome() {
  const cards = devices.map(item => `<div class="device">
    <button class="tile" data-action="device" data-id="${escape(item.id)}">
      ${art(item.type)}<strong>${escape(item.name)}</strong><small>${escape(item.brand)} · 红外</small>
    </button>
    <button class="edit" data-action="rename" data-id="${escape(item.id)}" aria-label="修改「${escape(item.name)}」的备注名">${icon('edit')}</button>
    </div>`).join('');
  return `${header('遥控器', true)}<main class="content section"><h2>我的遥控器</h2>
    ${available === false ? '<div class="note"><strong>此手机无法发射红外</strong><p>请在支持红外遥控的安卓手机上使用。</p></div>' : ''}
    ${devicesReadFailed ? '<p role="alert">已保存的遥控器读取失败，请重新打开遥控器。</p>' :
      cards ? `<div class="tiles">${cards}</div>` :
      !readingDevices ? `<div class="empty">${icon('remote')}<p>还没有遥控器</p></div>` : ''}
    </main>`;
}

function renderTypes() {
  const selection = Object.entries(typeLabels).map(([type, label]) => `<button class="tile category" data-action="type" data-value="${type}" ${catalog?.[type]?.length ? '' : 'disabled'}>
    ${art(type)}<strong>${label}</strong></button>`).join('');
  return `${header('添加遥控器')}<main class="content">${steps(0)}<section class="section">
    ${readingCatalog ? '<p role="status">正在读取内置码库…</p>' :
      catalog ? `<h2>选择设备类型</h2><div class="tiles">${selection}</div>` :
      '<p role="alert">内置码库读取失败，请重新打开遥控器。</p>'}
    </section></main>`;
}

function brandRows(query = '') {
  const brands = catalog?.[route.type] || [];
  const rank = rankedBrands[route.type];
  const indexed = new Map(brands.map(brand => [brand.name, brand]));
  const featured = rank.map(name => indexed.get(name)).filter(Boolean);
  const others = brands.filter(brand => !rank.includes(brand.name)).sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'));
  const keyword = query.trim().toLowerCase();
  const match = brand => brand.name.toLowerCase().includes(keyword) || aliases[brand.name]?.includes(keyword);
  const row = brand => `<button class="brand" data-action="brand" data-value="${escape(brand.name)}">${escape(brand.name)}${icon('right')}</button>`;
  if (keyword) {
    const results = [...featured, ...others].filter(match);
    return results.length ? results.map(row).join('') : '<div class="empty"><p>码库中没有可用的候选配置</p></div>';
  }
  const metric = route.type === 'tv' ? '电视出货份额' : '空调全渠道零售额份额';
  return `${featured.length ? `<div class="brand-group"><h2>2026 年 9 月 · ${metric}</h2>${featured.map(row).join('')}</div>` : ''}
    ${others.length ? `<div class="brand-group"><h2>其他品牌</h2>${others.map(row).join('')}</div>` : ''}`;
}

function renderBrands() {
  return `${header('选择' + typeLabels[route.type] + '品牌')}<main class="content">${steps(1)}
    <div class="search"><input type="search" aria-label="搜索品牌" placeholder="搜索品牌" autocomplete="off"></div>
    <div id="brand-list">${brandRows()}</div></main>`;
}

function renderPair() {
  const brand = findBrand();
  if (!brand) return `${header('试码')}<main class="content section"><p>找不到该品牌的配置。</p></main>`;
  const models = brand.profiles[route.index]?.models?.filter(name => name !== 'Unknown');
  const label = route.phase === 'volume' ? '音量 +' : '电源';
  const key = route.phase === 'volume' ? 'volumeUp' : 'power';
  const body = route.phase === 'failed'
    ? '<p>已试完该品牌的可用码集。</p><div class="actions"><button class="pill" data-action="back">换个品牌</button><button class="pill primary" data-action="retry">重新试码</button></div>'
    : route.phase === 'save'
      ? `<p>设备已响应，可以给遥控器起个备注名。</p>
        <form data-form="add"><label>备注名<input name="name" maxlength="14" value="${escape(brand.name + typeLabels[route.type])}" required></label>
        <button class="pill primary" type="submit" ${saving ? 'disabled' : ''}>保存并打开</button></form>`
      : `<p>将手机顶部对准设备，点按${label}。</p>
        <button class="test-key" aria-label="测试${label}" data-action="test" data-value="${key}">${icon(key)}</button>
        <p aria-live="polite" id="pair-state">${tested ? '设备有反应吗？' : `第 ${route.index + 1} / ${brand.profiles.length} 套遥控码`}</p>
        <div class="actions"><button class="pill" data-action="no" ${tested ? '' : 'disabled'}>没反应</button>
        <button class="pill primary" data-action="yes" ${tested ? '' : 'disabled'}>有反应</button></div>`;
  return `${header('测试' + typeLabels[route.type])}<main class="content">${steps(2)}
    <section class="pair">${art(route.type)}<h2>${escape(brand.name + typeLabels[route.type])}</h2>
    ${models?.length ? `<p>${escape(models.join(' / '))}</p>` : ''}${body}</section></main>`;
}

function keyButton(profile, key, label, glyph = '', css = '') {
  return `<button class="command ${css}" data-action="send" data-value="${key}" aria-label="${label}"
    ${available && profile?.keys.includes(key) ? '' : 'disabled'}>${glyph ? icon(glyph) : label}</button>`;
}

function renderTv(profile) {
  const key = (action, label, glyph, css) => keyButton(profile, action, label, glyph, css);
  return `<section class="tv"><div class="top-keys">
    ${key('power', '电源', 'power', 'round power')}
    ${key('home', '首页', 'home', 'round')}
    ${key('menu', '菜单', 'menu', 'round')}
    </div><div class="pad" aria-label="方向键">
    ${key('up', '向上', 'dot')}${key('left', '向左', 'dot')}${key('ok', 'OK')}
    ${key('right', '向右', 'dot')}${key('down', '向下', 'dot')}
    </div><div class="lower">
    <div class="rocker" role="group" aria-label="音量">${key('volumeDown', '音量减', 'volumeDown')}${key('volumeUp', '音量加', 'volumeUp')}</div>
    ${key('back', '电视返回', 'back', 'wide-key')}</div></section>`;
}

function acReadout(item) {
  const state = acStates.get(item.id);
  const mode = state?.on ? modes[state.mode] : '';
  const fan = state?.on ? (fanNames[state.fan?.toLowerCase()] || state.fan || '—') : '—';
  return `<div class="ac-display">
    <div class="setpoint"><output aria-label="设定温度">${state?.on ? escape(state.temperature) : '—'}</output><span>°C</span></div>
    ${mode ? `<span class="ac-mode">${icon(modeIcons[state.mode])}${mode}</span>` : ''}
    </div><div class="ac-status"><span>${icon('wind')}风速 ${escape(fan)}</span>
    <span>${icon('direction')}风向 —</span></div>`;
}

function renderAc(item, profile) {
  const key = (action, label, glyph, css) => keyButton(profile, action, label, glyph, css);
  const hasMode = profile?.keys.some(command => command in modes || command === 'mode');
  const usesSeparatePower = profile?.keys.includes('powerOn');
  return `<section class="ac"><div class="readout" id="ac-readout" aria-label="遥控器设定">
    ${acReadout(item)}
    </div><div class="key-row">
    ${usesSeparatePower ? `<button class="command power" data-action="ac-power" aria-label="电源" ${available ? '' : 'disabled'}>${icon('power')}</button>` : key('power', '电源', 'power', 'power')}
    <button class="command" data-action="modes" ${available && hasMode ? '' : 'disabled'}>模式</button>
    </div><div class="key-row">
    ${key('fan', '风速')}${key('direction', '风向')}${key('swing', '扫风')}
    </div><div class="key-row temperature">
    ${key('tempDown', '降低温度', 'minus')}<span>温度</span>${key('tempUp', '升高温度', 'plus')}
    </div><div class="key-row">
    <button disabled>定时</button><button disabled>睡眠</button>
    <button class="command" data-action="ac-details" aria-label="更多空调信息">${icon('more')}</button>
    </div></section>`;
}

function renderRemote() {
  const item = findDevice();
  if (!item) return renderHome();
  const profile = profiles.get(item.profile);
  return `${header(item.name)}<main class="content">
    ${available === false ? '<div class="section"><p>此手机无法发射红外。</p></div>' :
      !readingCatalog && !catalog ? '<div class="section"><p role="alert">内置码库读取失败，请重新打开遥控器。</p></div>' :
      !readingCatalog && !profile ? '<div class="section"><p>当前码库没有此遥控配置，请返回列表重新添加。</p></div>' : ''}
    ${item.type === 'tv' ? renderTv(profile) : renderAc(item, profile)}</main>`;
}

function render() {
  const pages = { home: renderHome, types: renderTypes, brands: renderBrands, pair: renderPair, remote: renderRemote };
  app.classList.toggle('remote', route.page === 'remote');
  const busy = route.page === 'home' ? readingDevices :
    readingCatalog && (route.page === 'types' || route.page === 'remote');
  app.setAttribute('aria-busy', busy);
  app.innerHTML = pages[route.page]();
}

async function send(profile, key, state) {
  if (sending) return false;
  sending = true;
  try {
    return await host.remote.send(profile, key, state);
  } catch (error) {
    toast(errors[error.code] || '红外发射失败，请重试');
    return false;
  } finally {
    sending = false;
  }
}

async function test(button) {
  const current = route;
  const profile = findBrand().profiles[current.index];
  const result = await send(profile.id, button.dataset.value);
  if (!result || route !== current) return;
  testedAcState = result === true ? null : { profile: profile.id, state: result };
  tested = true;
  document.querySelector('#pair-state').textContent = '设备有反应吗？';
  app.querySelector('[data-action="yes"]').disabled = false;
  app.querySelector('[data-action="no"]').disabled = false;
}

function confirmMatch() {
  if (!tested) return;
  const profile = findBrand().profiles[route.index];
  const phase = route.type === 'tv' && route.phase === 'power' && profile.keys.includes('volumeUp') ? 'volume' : 'save';
  go({ ...route, phase }, true);
}

function showRename(id) {
  const item = devices.find(device => device.id === id);
  if (!item) return;
  sheet.setAttribute('aria-label', '修改备注名');
  sheet.innerHTML = `<h2>修改备注名</h2><form class="rename" data-form="rename" data-id="${escape(id)}">
    <label>备注名<input name="name" value="${escape(item.name)}" maxlength="14" required autofocus></label>
    <div class="actions"><button class="pill" type="button" data-action="dismiss">取消</button>
    <button class="pill primary" type="submit">保存</button></div></form>`;
  sheet.showModal();
}

function showModes() {
  const profile = profiles.get(findDevice().profile);
  const choices = Object.entries(modes).filter(([key]) => profile.keys.includes(key));
  if (profile.keys.includes('mode')) choices.push(['mode', '切换模式']);
  sheet.setAttribute('aria-label', '模式');
  sheet.innerHTML = `<h2>模式</h2><div class="actions">
    ${choices.map(([key, label]) => keyButton(profile, key, label, '', 'pill')).join('')}
    <button class="pill" data-action="dismiss">取消</button></div>`;
  sheet.showModal();
}

function showAcSheet(kind) {
  const item = findDevice();
  if (!item) return;
  const profile = profiles.get(item.profile);
  const power = kind === 'ac-power';
  const title = power ? '电源' : '空调信息';
  sheet.setAttribute('aria-label', title);
  sheet.innerHTML = `<h2>${title}</h2>${power
    ? `<div class="actions">${keyButton(profile, 'powerOn', '开机', '', 'pill')}${keyButton(profile, 'powerOff', '关机', '', 'pill')}</div>`
    : `<p>${escape(latest.get(item.id) || '红外控制以空调实际响应为准')}</p>`}
    <button class="pill" data-action="dismiss">返回</button>`;
  sheet.showModal();
}

async function sendCommand(button) {
  const item = findDevice();
  const fromSheet = button.closest('dialog') ? history.state : null;
  if (!item) return;
  const result = await send(item.profile, button.dataset.value, acStates.get(item.id));
  if (!result) return;
  if (result !== true) acStates.set(item.id, result);
  const label = button.getAttribute('aria-label');
  latest.set(item.id, '已发送 · ' + label);
  if (fromSheet && sheet.open && findDevice()?.id === item.id &&
      fromSheet.modes === history.state?.modes && fromSheet.acSheet === history.state?.acSheet) history.back();
  if (findDevice()?.id === item.id) {
    const readout = document.querySelector('#ac-readout');
    if (readout) readout.innerHTML = acReadout(item);
  }
  toast('已发送 · ' + label);
}

async function handleSubmit(event) {
  event.preventDefault();
  if (saving) return;
  const form = event.target;
  const input = form.elements.namedItem('name');
  const submittedRoute = route;
  const name = input.value.trim();
  if (!name) { input.focus(); return; }
  const rename = form.dataset.form === 'rename';
  const profile = rename ? null : findBrand()?.profiles[route.index];
  if (!rename && (!profile || devices.length >= 100)) {
    toast('已达到遥控器保存上限');
    return;
  }
  const item = rename
    ? { ...devices.find(device => device.id === form.dataset.id), name }
    : { id: crypto.randomUUID(), type: route.type, brand: route.brand, profile: profile.id, name };
  const updated = rename ? devices.map(device => device.id === item.id ? item : device) : [...devices, item];
  saving = true;
  const submit = form.querySelector('[type="submit"]');
  submit.disabled = true;
  try {
    await host.storage.set('devices', updated);
    devices = updated;
    if (!rename && testedAcState?.profile === item.profile) acStates.set(item.id, testedAcState.state);
    if (route !== submittedRoute || (rename && history.state?.rename !== item.id)) {
      render();
    } else if (rename) {
      history.back();
      render();
      toast('已保存备注名');
    } else {
      window.addEventListener('popstate', () => go({ page: 'remote', id: item.id }), { once: true });
      // 设备类型、品牌和试码各占一条历史，从列表打开刚保存的遥控器。
      history.go(-3);
    }
  } catch (_) {
    toast('保存失败，请重试');
  } finally {
    saving = false;
    submit.disabled = false;
  }
}

function actions(event) {
  const button = event.target.closest('button[data-action]');
  if (!button || button.disabled || saving) return;
  const { action, value, id } = button.dataset;
  switch (action) {
    case 'add': go({ page: 'types' }); break;
    case 'type': go({ page: 'brands', type: value }); break;
    case 'brand': go({ ...route, page: 'pair', brand: value, index: 0, phase: 'power' }); break;
    case 'back': case 'dismiss': history.back(); break;
    case 'device': go({ page: 'remote', id }); break;
    case 'test': void test(button); break;
    case 'yes': confirmMatch(); break;
    case 'no':
      if (tested) {
        const next = route.index + 1;
        go(next < findBrand().profiles.length ? { ...route, index: next, phase: 'power' } : { ...route, phase: 'failed' }, true);
      }
      break;
    case 'retry': go({ ...route, index: 0, phase: 'power' }, true); break;
    case 'send': void sendCommand(button); break;
    case 'rename':
      history.pushState({ route, rename: id }, '');
      showRename(id);
      break;
    case 'modes':
      history.pushState({ route, modes: true }, '');
      showModes();
      break;
    case 'ac-power': case 'ac-details':
      history.pushState({ route, acSheet: action }, '');
      showAcSheet(action);
      break;
  }
}

history.replaceState({ route }, '');
window.addEventListener('popstate', event => {
  if (sheet.open) sheet.close();
  const state = event.state || { route: home };
  if (JSON.stringify(route) !== JSON.stringify(state.route)) {
    route = state.route || home;
    tested = false;
    render();
  }
  if (state.rename) showRename(state.rename);
  if (state.modes) showModes();
  if (state.acSheet) showAcSheet(state.acSheet);
});
app.addEventListener('input', event => {
  if (event.target.type === 'search') document.querySelector('#brand-list').innerHTML = brandRows(event.target.value);
});
app.addEventListener('click', actions);
sheet.addEventListener('click', actions);
app.addEventListener('submit', handleSubmit);
sheet.addEventListener('submit', handleSubmit);
sheet.addEventListener('cancel', event => {
  event.preventDefault();
  if (!saving) history.back();
});

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

async function initialize() {
  setTheme(host?.info?.theme || 'light');
  host?.on('theme', setTheme);
  render();
  if (!host) {
    available = false;
    readingDevices = false;
    readingCatalog = false;
    render();
    return;
  }

  await host.ready().catch(() => {});
  const loadDevices = host.storage.get('devices').then(saved => {
    if (Array.isArray(saved)) {
      devices = saved.filter(item => item && item.type in typeLabels && ['id', 'name', 'brand', 'profile'].every(key => typeof item[key] === 'string'));
    }
  }, () => {
    devicesReadFailed = true;
  }).finally(() => {
    readingDevices = false;
    render();
  });
  const checkInfrared = host.remote.supported().then(supported => {
    available = supported === true;
  }, () => {
    available = false;
  }).finally(render);
  await Promise.all([loadDevices, checkInfrared]);

  try {
    if (available) setCatalog(await host.remote.catalog());
  } catch (_) {
    setCatalog(null);
  } finally {
    readingCatalog = false;
    if (route.page === 'types' || route.page === 'remote') render();
  }
}

void initialize();
