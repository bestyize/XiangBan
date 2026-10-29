import { applyUiTokens } from './ui/design-tokens';
import './games.css';
import { desktop, invoke } from './bridge.ts';
import { GameSession, validSave, sameJson, type GameManifest } from './games/protocol.ts';
import { entryUrl, loadBuiltinCatalog, readGames } from './games/catalog.ts';

declare global { interface Window { __XIAOBAN_GAME_TOKEN__?: string; __XIAOBAN_GAME_SMOKE__?: boolean } }

/** The full-screen window runs exactly one game: entering starts it, and exiting closes the window. */
export async function mountGameSpace() {
  applyUiTokens();
  document.body.className = 'game-space';
  const app = document.querySelector<HTMLElement>('#app')!;
  app.innerHTML = '<main class="games-shell"><header class="games-toolbar"><div><span id="games-status" role="status">正在准备游戏…</span></div></header><section id="games-player"><div id="game-stage"></div></section><div id="games-failure" class="games-failure" hidden><h2>游戏需要重试</h2><p id="games-error"></p><button id="games-retry">重试</button><button id="games-exit">退出游戏</button><button id="games-force-close" hidden>放弃未保存进度并退出</button></div></main>';
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
  const fail = (error: unknown) => { clearTimeout(timer); frame?.remove(); frame = null; session = null; node('games-error').textContent = String(error); node('games-failure').hidden = false; status('游戏未能继续'); };
  /** Leaves the game for good: the native host closes this window instead of showing a game list. */
  const exit = async () => {
    if (!await flush()) { node('games-error').textContent = saveFailure; node<HTMLButtonElement>('games-retry').textContent = '重试保存'; node('games-failure').hidden = false; node('games-force-close').hidden = false; return; }
    destroyPlayer(); current = null;
    try { if (desktop) await gameInvoke('close_game_space'); else location.assign('/'); } catch (e) { fail(e); }
  };
  const launch = async (game: GameManifest) => {
    if (!await flush()) return;
    destroyPlayer(); current = game; const token = generation;
    node('games-failure').hidden = true; node('games-force-close').hidden = true; node<HTMLButtonElement>('games-retry').textContent = '重试'; status(`正在准备 ${game.name}…`);
    try {
      const save = game.save ? await readSave(game.id) : null;
      if (generation !== token) return;
      const nonce = crypto.randomUUID();
      const iframe = document.createElement('iframe');
      iframe.title = game.name; iframe.sandbox.add('allow-scripts');
      // The frame fills the native game window, which the player can resize or take full screen. Nested browser
      // fullscreen can consume Escape before JS, so it is never requested here.
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
  window.addEventListener('message', event => {
    const message = session?.accept(event); if (!message) return;
    if (message.type === 'game.ready') { clearTimeout(timer); status(current?.name ?? '游戏已就绪'); }
    if (message.type === 'game.exit') void exit();
    if (message.type === 'game.save' && current?.save) { pendingSave = { id: current.id, data: message.data as Record<string, unknown> }; void flush(); }
    if (message.type === 'game.result') status(`${current?.name} · 本局 ${session?.record?.score ?? 0} 分`);
    if (message.type === 'game.security' && window.__XIAOBAN_GAME_SMOKE__ && current) security.set(current.id, message.nativeAccess === false && message.parentAccess === false);
    // Desktop has no account wallet or website sharing. The shipped adapter hides those controls.
    if (message.type === 'game.wallet') frame?.contentWindow?.postMessage({ channel:'xiaban.game-host', protocol:1, type:'host.wallet', nonce:session!.nonce, requestId:message.requestId, error:'桌面版不使用账户糖果币' }, '*');
  });
  window.addEventListener('keydown', e => { if (e.key === 'Escape' && !current) { e.preventDefault(); void exit(); } });
  window.addEventListener('game-space-close', () => { void exit(); });
  node('games-exit').onclick = () => { void exit(); };
  node('games-force-close').onclick = () => { destroyPlayer(); pendingSave = null; saveFailure = null; current = null; void exit(); };
  node('games-retry').onclick = () => { if (saveFailure) void flush().then(ok => { if (ok) void exit(); }); else if (current) void launch(current); else void exit(); };
  async function loadCatalog() {
    try {
      manifests = readGames(desktop ? await gameInvoke<unknown>('get_game_catalog') : (await loadBuiltinCatalog()).games);
      if (!manifests.length) throw new Error('没有找到可玩的游戏');
      return true;
    } catch (e) { fail(e); return false; }
  }
  const launchId = async (id: unknown) => { const game = manifests.find(item => item.id === id); if (game) await launch(game); };
  window.addEventListener('game-launch', event => { void launchId((event as CustomEvent).detail); });
  const requested = new URLSearchParams(location.search).get('game');
  if (await loadCatalog()) {
    if (requested) { if (!manifests.some(game => game.id === requested)) fail('没有找到这款本地游戏，它可能已被删除。'); else await launchId(requested); }
    else if (!window.__XIAOBAN_GAME_SMOKE__) fail('请从游戏空间选择要玩的游戏。');
  }
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
      const fillWindow = Math.abs(stage.top) < 1 && Math.abs(stage.height - innerHeight) < 1 && Math.abs(stage.width - innerWidth) < 1;
      if (selectedId && !fillWindow) throw new Error('游戏没有铺满游戏窗口');
      for (const game of manifests) {
        await launch(game);
        const deadline = Date.now() + 18000;
        while ((!session || !(session as GameSession).ready || !security.has(game.id)) && Date.now() < deadline) await new Promise(resolve => setTimeout(resolve, 40));
        if (!(session as GameSession | null)?.ready || security.get(game.id) !== true) throw new Error(`${game.id} 未通过ready/iframe权限检查`);
        ready.push(game.id);
      }
      const saved = { desktopSmoke: true, chapters: { bamboo: 2 } };
      await gameInvoke('save_game_save', { gameId:'ink-blade', data:saved });
      const restored = await gameInvoke('load_game_save', { gameId:'ink-blade' });
      if (!sameJson(restored, saved)) throw new Error(`本地存档往返不一致：写入 ${JSON.stringify(saved)}；读回 ${JSON.stringify(restored)}`);
      let rejected = false;
      try { await invoke('get_settings'); } catch { rejected = true; }
      if (!rejected) throw new Error('游戏宿主越权命令未被拒绝');
      await gameInvoke('game_smoke_report', { report: { ready, directLaunch: selectedId, fillWindow, security: Object.fromEntries(security), saveRoundTrip: true, controlRejected: true } });
    } catch (error) { await gameInvoke('game_smoke_report', { report: { ready, error: String(error) } }); }
  }
}
