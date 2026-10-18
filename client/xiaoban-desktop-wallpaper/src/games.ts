import { applyUiTokens } from './ui/design-tokens';
import './games.css';
import { desktop, invoke } from './bridge.ts';
import { GameSession, validSave, sameJson, type GameManifest } from './games/protocol.ts';
import { coverUrl, entryUrl, loadBuiltinCatalog, readGames } from './games/catalog.ts';

declare global { interface Window { __XIAOBAN_GAME_TOKEN__?: string; __XIAOBAN_GAME_SMOKE__?: boolean } }

export async function mountGameSpace() {
  applyUiTokens();
  document.body.className = 'game-space';
  const app = document.querySelector<HTMLElement>('#app')!;
  app.innerHTML = '<main class="games-shell"><header class="games-toolbar"><div><span id="games-status" role="status">正在准备游戏目录…</span></div></header><section id="games-hub"><div class="games-intro"><p>给自己一小段轻松时光</p><h1>今天想玩点什么？</h1><span>本地小游戏，断网也能玩。需要存档的游戏，进度保存在这台电脑。</span></div><div id="games-cards" class="games-grid"></div></section><section id="games-player" hidden><div id="game-stage"></div></section><div id="games-failure" class="games-failure" hidden><h2>游戏需要重试</h2><p id="games-error"></p><button id="games-retry">重试</button><button id="games-cancel">返回游戏面板</button><button id="games-force-close" hidden>放弃未保存进度并退出</button></div></main>';
  const node = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
  let manifests: GameManifest[] = [];
  let session: GameSession | null = null;
  let current: GameManifest | null = null;
  let frame: HTMLIFrameElement | null = null;
  let timer = 0;
  let generation = 0;
  let pendingSave: { id: string; data: Record<string, unknown> } | null = null;
  let saveChain = Promise.resolve();
  let saveFailure: string | null = null;
  const security = new Map<string, boolean>();
  const gameInvoke = <T = unknown>(command: string, args: Record<string, unknown> = {}) => invoke<T>(command, { ...args, hostToken: window.__XIAOBAN_GAME_TOKEN__ });
  const status = (text: string) => { node('games-status').textContent = text; };
  const readSave = async (id: string) => {
    if (desktop) return gameInvoke<Record<string, unknown> | null>('load_game_save', { gameId: id });
    const raw = localStorage.getItem(`xiaoban-game-${id}`);
    if (!raw) return null;
    const data = JSON.parse(raw); if (!validSave(data)) throw new Error('本地存档格式不正确，原存档已保留。'); return data;
  };
  const flush = async () => {
    saveChain = saveChain.then(async () => {
      const job = pendingSave;
      if (!job) return;
      try {
        if (desktop) await gameInvoke('save_game_save', { gameId: job.id, data: job.data });
        else localStorage.setItem(`xiaoban-game-${job.id}`, JSON.stringify(job.data));
        if (pendingSave === job) pendingSave = null;
        saveFailure = null;
      } catch (e) { saveFailure = String(e); status(`存档未保存：${e}。请重试保存后退出。`); }
    });
    await saveChain;
    return !saveFailure;
  };
  const destroyPlayer = () => { generation++; clearTimeout(timer); session = null; frame?.remove(); frame = null; node('game-stage').replaceChildren(); };
  const hub = async () => {
    if (!await flush()) { node<HTMLButtonElement>('games-retry').textContent = '重试保存'; node('games-error').textContent = saveFailure; node('games-failure').hidden = false; node('games-force-close').hidden = false; return; }
    destroyPlayer(); current = null; node('games-player').hidden = true; node('games-hub').hidden = false; node('games-failure').hidden = true; status('选择游戏 · Esc 退出');
  };
  const fail = (error: unknown) => { clearTimeout(timer); frame?.remove(); frame = null; session = null; node('games-error').textContent = String(error); node('games-failure').hidden = false; status('加载失败，可重试或退出'); };
  const launch = async (game: GameManifest) => {
    if (!await flush()) return;
    destroyPlayer(); current = game; const token = generation;
    node('games-hub').hidden = true; node('games-player').hidden = false; node('games-failure').hidden = true; node('games-force-close').hidden = true; node<HTMLButtonElement>('games-retry').textContent = '重试'; status(`正在准备 ${game.name}…`);
    try {
      const save = game.save ? await readSave(game.id) : null;
      if (generation !== token) return;
      const nonce = crypto.randomUUID();
      const iframe = document.createElement('iframe');
      iframe.title = game.name; iframe.sandbox.add('allow-scripts');
      // Native host is already fullscreen. Nested browser fullscreen can consume Escape before JS.
      iframe.allow = 'autoplay'; iframe.referrerPolicy = 'no-referrer';
      iframe.src = `${entryUrl(game)}?hostNonce=${encodeURIComponent(nonce)}&desktop=1${window.__XIAOBAN_GAME_SMOKE__ ? '&desktopSmoke=1' : ''}`;
      frame = iframe;
      iframe.addEventListener('load', () => {
        if (!iframe.contentWindow || generation !== token) return;
        session = new GameSession(game.id, nonce, iframe.contentWindow);
        iframe.contentWindow.postMessage({ channel: 'xiaban.game-host', protocol: 1, type: 'host.init', nonce, save, safeArea: { top: 0, bottom: 0, left: 0, right: 0 } }, '*');
        iframe.contentWindow.focus();
      });
      iframe.addEventListener('error', () => fail('游戏资源加载失败。'));
      node('game-stage').append(iframe);
      timer = window.setTimeout(() => { if (generation === token && !session?.ready) fail('游戏未能完成加载，请重试。'); }, 15000);
    } catch (e) { if (generation === token) fail(e); }
  };
  const close = async () => {
    if (!await flush()) { node('games-error').textContent = saveFailure; node<HTMLButtonElement>('games-retry').textContent = '重试保存'; node('games-failure').hidden = false; node('games-force-close').hidden = false; return; }
    destroyPlayer();
    try { if (desktop) await gameInvoke('close_game_space'); else { await hub(); status('浏览器预览：已退出游戏'); } } catch (e) { fail(e); }
  };
  window.addEventListener('message', event => {
    const message = session?.accept(event); if (!message) return;
    if (message.type === 'game.ready') { clearTimeout(timer); status(current?.name ?? '游戏已就绪'); }
    if (message.type === 'game.exit') void hub();
    if (message.type === 'game.save' && current?.save) { pendingSave = { id: current.id, data: message.data as Record<string, unknown> }; void flush(); }
    if (message.type === 'game.result') status(`${current?.name} · 本局 ${session?.record?.score ?? 0} 分`);
    if (message.type === 'game.security' && window.__XIAOBAN_GAME_SMOKE__ && current) security.set(current.id, message.nativeAccess === false && message.parentAccess === false);
    // Desktop has no account wallet or website sharing. The shipped adapter hides those controls.
    if (message.type === 'game.wallet') frame?.contentWindow?.postMessage({ channel:'xiaban.game-host', protocol:1, type:'host.wallet', nonce:session!.nonce, requestId:message.requestId, error:'桌面版不使用账户糖果币' }, '*');
  });
  window.addEventListener('keydown', e => { if (e.key === 'Escape' && !current) { e.preventDefault(); void close(); } });
  window.addEventListener('game-space-close', () => { void close(); });
  node('games-cancel').onclick = () => { void hub(); };
  node('games-force-close').onclick = () => { destroyPlayer(); pendingSave = null; saveFailure = null; void close(); };
  node('games-retry').onclick = () => { if (saveFailure) void flush().then(ok => { if (ok) void hub(); }); else if (current) void launch(current); else void loadCatalog(); };
  async function loadCatalog() {
    try {
      manifests = readGames(desktop ? await gameInvoke<unknown>('get_game_catalog') : (await loadBuiltinCatalog()).games);
      if (!manifests.length) throw new Error('没有找到可玩的游戏');
      node('games-cards').replaceChildren(...manifests.map(game => {
        const button = document.createElement('button'); button.className = 'game-card';
        const image = document.createElement('img'); image.src = coverUrl(game); image.alt = ''; image.loading = 'lazy';
        const name = document.createElement('strong'); name.textContent = game.name;
        const description = document.createElement('span'); description.textContent = game.description;
        const badge = document.createElement('small'); badge.textContent = game.save ? '本地存档 · 点击开始' : '离线可玩 · 点击开始';
        button.append(image, name, description, badge); button.onclick = () => { void launch(game); }; return button;
      })); status('选择游戏 · Esc 退出');
    } catch (e) { fail(e); }
  }
  await loadCatalog();
  const launchId = async (id: unknown) => { const game = manifests.find(item => item.id === id); if (game) await launch(game); };
  window.addEventListener('game-launch', event => { void launchId((event as CustomEvent).detail); });
  await launchId(new URLSearchParams(location.search).get('game'));
  const smokeEscape = () => frame?.contentWindow?.postMessage({channel:'xiaban.game-host',protocol:1,type:'host.smoke.escape',nonce:session?.nonce}, '*');
  if (window.__XIAOBAN_GAME_SMOKE__) {
    const ready: string[] = [];
    try {
      const selectedId = new URLSearchParams(location.search).get('game');
      if (selectedId) {
        const deadline = Date.now() + 18000;
        while ((!session || !(session as GameSession).ready) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 40));
        if ((current as GameManifest | null)?.id !== selectedId || !(session as GameSession | null)?.ready) throw new Error('指定游戏没有直接启动');
      }
      const stage = node('game-stage').getBoundingClientRect();
      const fullViewport = Math.abs(stage.top) < 1 && Math.abs(stage.height - innerHeight) < 1 && Math.abs(stage.width - innerWidth) < 1;
      if (selectedId && !fullViewport) throw new Error('游戏未占满窗口');
      for (const game of manifests) {
        await launch(game);
        const deadline = Date.now() + 18000;
        while ((!session || !(session as GameSession).ready || !security.has(game.id)) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 40));
        if (!(session as GameSession | null)?.ready || security.get(game.id) !== true) throw new Error(`${game.id} 未通过ready/iframe权限检查`);
        ready.push(game.id);
        smokeEscape();
        const exitDeadline=Date.now()+3000;
        while (current && Date.now()<exitDeadline) await new Promise(resolve=>setTimeout(resolve,30));
        if (current) throw new Error(`${game.id} Esc没有返回游戏面板`);
      }
      const saved = { desktopSmoke: true, chapters: { bamboo: 2 } };
      await gameInvoke('save_game_save', { gameId:'ink-blade', data:saved });
      const restored = await gameInvoke('load_game_save', { gameId:'ink-blade' });
      if (!sameJson(restored, saved)) throw new Error(`本地存档往返不一致：写入 ${JSON.stringify(saved)}；读回 ${JSON.stringify(restored)}`);
      let rejected = false;
      try { await invoke('get_settings'); } catch { rejected = true; }
      if (!rejected) throw new Error('游戏宿主越权命令未被拒绝');
      await gameInvoke('game_smoke_report', { report: { ready, directLaunch: selectedId, fullViewport, security: Object.fromEntries(security), escapeReturned: true, saveRoundTrip: true, controlRejected: true } });
    } catch (error) { await gameInvoke('game_smoke_report', { report: { ready, error: String(error) } }); }
  }
}
