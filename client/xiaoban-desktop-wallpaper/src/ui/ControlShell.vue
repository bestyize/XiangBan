<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onMounted, onBeforeUnmount, reactive, ref, watch } from 'vue';
import { NAlert, NButton, NSkeleton, useMessage } from 'naive-ui';
import { Bell, BookOpen, Check, ChevronDown, Ellipsis, Folder, FolderPlus, Gamepad2, Globe, Inbox, LogIn, LogOut, Palette, PanelLeft, Pencil, Plus, Puzzle, RefreshCw, Search, SlidersHorizontal, Sparkles, SquarePen, Trash2, Zap } from 'lucide-vue-next';
import { invoke, listen } from '../bridge.ts';
import { accent, accents, isDark, appearanceMode, appearanceModes, publishAccentTo, publishAppearanceTo, reconcile, setAccent } from './appearance.ts';
import { createControlStore } from './control-store.ts';
import type { ControlPage, SettingsTab } from './control-types.ts';
import type { SessionSummary } from './chat/types.ts';
import SettingsDialog from './components/SettingsDialog.vue';
import ChatNameDialog from './components/ChatNameDialog.vue';
import CommunityApplyDialog from './community/CommunityApplyDialog.vue';
import WorkspaceSearchDialog from './components/WorkspaceSearchDialog.vue';
import { useWorkspaceSort } from './workspace-sort.ts';
const pageLoaders = {
  'new-task': () => import('./pages/NewTaskPage.vue'),
  chat: () => import('./pages/ChatPage.vue'),
  'ai-store': () => import('./pages/AiStorePage.vue'),
  library: () => import('./pages/LibraryPage.vue'),
  studio: () => import('./pages/StudioPage.vue'),
  interactive: () => import('./pages/InteractivePage.vue'),
  games: () => import('./pages/GamesPage.vue'),
  care: () => import('./pages/CarePage.vue'),
  community: () => import('./pages/CommunityPage.vue'),
};
const NewTaskPage = defineAsyncComponent(pageLoaders['new-task']);
const ChatPage = defineAsyncComponent(pageLoaders.chat);
const AiStorePage = defineAsyncComponent(pageLoaders['ai-store']);
const LibraryPage = defineAsyncComponent(pageLoaders.library);
const StudioPage = defineAsyncComponent(pageLoaders.studio);
const InteractivePage = defineAsyncComponent(pageLoaders.interactive);
const GamesPage = defineAsyncComponent(pageLoaders.games);
const CarePage = defineAsyncComponent(pageLoaders.care);
const CommunityPage = defineAsyncComponent(pageLoaders.community);
const LoginModal = defineAsyncComponent(() => import('./components/LoginModal.vue'));

const message = useMessage();
const store = createControlStore((text, type = 'success') => message[type](String(text), { duration: type === 'error' ? 6500 : 3500 }));
const mac = computed(() => store.info.platform === 'macos' || (store.info.platform === 'preview' && typeof navigator !== 'undefined' && /Mac/i.test(navigator.platform)));
const mod = computed(() => mac.value ? '⌘' : 'Ctrl+');
/** The console draws its own title bar: macOS keeps the traffic lights over the sidebar, Windows gets drawn caption buttons. */
const win = computed(() => store.info.platform === 'windows');
type NativeWindow = { minimize(): Promise<void>; toggleMaximize(): Promise<void>; close(): Promise<void> };
function nativeWindow(): NativeWindow | null {
  return (window as unknown as { __TAURI__?: { window?: { getCurrentWindow?: () => NativeWindow } } }).__TAURI__?.window?.getCurrentWindow?.() ?? null;
}
function windowAction(action: 'minimize' | 'toggleMaximize' | 'close') { void nativeWindow()?.[action]().catch(() => {}); }

/* ── Sidebar ─────────────────────────────────────────────────────────── */
const collapsed = ref(false);
function toggleNavigation() { collapsed.value = !collapsed.value; }
const aiOpen = ref(true);
const bottomItems = [
  { id: 'studio', label: '装扮', icon: Palette },
  { id: 'games', label: '游玩', icon: Gamepad2 },
  { id: 'care', label: '提醒', icon: Bell },
  { id: 'community', label: '社区', icon: Globe },
] as const;
const aiItems = computed(() => [
  { id: 'new-task', label: '新建任务', icon: SquarePen, hint: `${mod.value}N` },
  { id: 'ai-store', label: 'AI 插件', icon: Puzzle, hint: '' },
  { id: 'library', label: '资料库', icon: BookOpen, hint: '' },
] as const);
/** 新建任务 stays lit on the chat page while no conversation is open. */
function isCurrent(id: string) {
  if (id === 'new-task') return store.page === 'new-task' || (store.page === 'chat' && !store.chat.currentId);
  return store.page === id;
}
const attention = computed(() => store.chat.attention);

interface Space { key: string; name: string; path: string; sessions: SessionSummary[] }
const RECENT = '__recent__';
const COMPANIONS = '__companions__';
const WORKSPACE_NAMES_KEY = 'xiaoban.workspace-names';
const WORKSPACES_OPEN_KEY = 'xiaoban.workspaces-open';
const REMOVED_WORKSPACES_KEY = 'xiaoban.removed-workspaces';
const WORKSPACE_ORDER_KEY = 'xiaoban.workspace-order';
const workspaceOrder = ref<string[]>((() => {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(WORKSPACE_ORDER_KEY) || '[]');
    if (Array.isArray(saved)) return [...new Set(saved.filter((path): path is string => typeof path === 'string' && !!path))];
  } catch { /* Ignore a damaged ordering preference. */ }
  return [];
})());
function saveWorkspaceOrder(paths: string[]) {
  try {
    localStorage.setItem(WORKSPACE_ORDER_KEY, JSON.stringify(paths));
    workspaceOrder.value = paths;
  } catch { store.notify('项目排序保存失败，请重试', 'error'); }
}
function readRemovedWorkspaces(): string[] {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(REMOVED_WORKSPACES_KEY) || '[]');
    if (Array.isArray(saved)) return saved.filter((path): path is string => typeof path === 'string' && !!path);
  } catch { /* Ignore an unreadable sidebar preference. */ }
  return [];
}
const removedWorkspaces = ref(new Set(readRemovedWorkspaces()));
function saveRemovedWorkspaces(paths: Set<string>): boolean {
  try {
    localStorage.setItem(REMOVED_WORKSPACES_KEY, JSON.stringify([...paths]));
    removedWorkspaces.value = paths;
    return true;
  } catch { store.notify('项目列表保存失败，请重试', 'error'); return false; }
}
function restoreWorkspace(path: string) {
  if (!removedWorkspaces.value.has(path)) return;
  const paths = new Set(removedWorkspaces.value);
  paths.delete(path);
  saveRemovedWorkspaces(paths);
}
// Selecting a folder again explicitly adds its project back to the sidebar.
watch(() => store.chat.pendingWorkspace, path => { if (path) restoreWorkspace(path); });
function readWorkspaceNames(): Record<string, string> {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(WORKSPACE_NAMES_KEY) || '{}');
    if (saved && typeof saved === 'object' && !Array.isArray(saved)) {
      return Object.fromEntries(Object.entries(saved).filter((entry): entry is [string, string] => typeof entry[1] === 'string' && !!entry[1].trim()));
    }
  } catch { /* A damaged preference must not prevent opening the sidebar. */ }
  return {};
}
const workspaceNames = ref(readWorkspaceNames());
const workspacesOpen = ref((() => { try { return localStorage.getItem(WORKSPACES_OPEN_KEY) !== 'false'; } catch { return true; } })());
watch(workspacesOpen, value => {
  try { localStorage.setItem(WORKSPACES_OPEN_KEY, String(value)); } catch { /* Keep the current window usable. */ }
});
function toggleWorkspaces() {
  workspacesOpen.value = !workspacesOpen.value;
  closeWorkspaceMenu();
  closeSessionMenu();
}
const folderName = (path: string) => Object.hasOwn(workspaceNames.value, path) ? workspaceNames.value[path]!
  : path === store.chat.workspace ? '默认分组' : path.split(/[\\/]+/).filter(Boolean).pop() || path;
/** 空间: 最近任务 (conversations without a folder) first, then one entry per working folder, named after the folder. */
const spaces = computed<Space[]>(() => {
  const recent: Space = { key: RECENT, name: '最近任务', path: '', sessions: [] };
  const companions: Space = { key: COMPANIONS, name: '伙伴', path: '', sessions: [] };
  const byPath = new Map<string, Space>();
  for (const session of store.chat.sessions) {
    if (session.companionId && !session.workspace?.trim()) { companions.sessions.push(session); continue; }
    const path = session.workspace?.trim();
    if (!path || removedWorkspaces.value.has(path)) { recent.sessions.push(session); continue; }
    let space = byPath.get(path);
    if (!space) { space = { key: path, name: folderName(path), path, sessions: [] }; byPath.set(path, space); }
    space.sessions.push(session);
  }
  const pending = store.chat.pendingWorkspace;
  if (pending && !removedWorkspaces.value.has(pending) && !byPath.has(pending)) byPath.set(pending, { key: pending, name: folderName(pending), path: pending, sessions: [] });
  const positions = new Map(workspaceOrder.value.map((path, index) => [path, index]));
  const projects = [...byPath.values()].sort((a, b) => (positions.get(a.path) ?? Infinity) - (positions.get(b.path) ?? Infinity));
  return [recent, ...(companions.sessions.length ? [companions] : []), ...projects];
});
const workspaceTree = ref<HTMLElement | null>(null);
const { dragging: draggingWorkspace, target: workspaceDropTarget, start: startWorkspaceSort, click: workspaceSortClick, cancel: cancelWorkspaceSort } = useWorkspaceSort(
  workspaceTree, () => spaces.value.filter(space => space.path).map(space => space.path), saveWorkspaceOrder,
);
const expanded = reactive(new Set<string>([RECENT, COMPANIONS]));
const searchOpen = ref(false);
const displayLimit = 6;
const showAllProjects = ref(false);
const expandedSessionLists = reactive(new Set<string>());
const projectSpaces = computed(() => spaces.value.filter(space => !!space.path));
const visibleSpaces = computed(() => spaces.value.filter(space => !space.path || showAllProjects.value || projectSpaces.value.indexOf(space) < displayLimit));
function visibleSessions(space: Space) { return expandedSessionLists.has(space.key) ? space.sessions : space.sessions.slice(0, displayLimit); }
function revealSession(id: string | null) {
  const space = spaces.value.find(item => item.sessions.some(session => session.id === id));
  if (!space) return;
  expanded.add(space.key);
  if (space.path && projectSpaces.value.indexOf(space) >= displayLimit) showAllProjects.value = true;
  if (space.sessions.findIndex(session => session.id === id) >= displayLimit) expandedSessionLists.add(space.key);
}
function openSearch() { cancelWorkspaceSort(); closeWorkspaceMenu(); closeSessionMenu(); closeMenu(); searchOpen.value = true; }
async function selectSearchResult(id: string) {
  aiOpen.value = true; workspacesOpen.value = true; revealSession(id);
  await openSession(id); await nextTick();
  const row = [...(workspaceTree.value?.querySelectorAll<HTMLElement>('[data-session-id]') ?? [])].find(row => row.dataset.sessionId === id);
  row?.scrollIntoView?.({ block: 'nearest' }); row?.focus({ preventScroll: true });
}
function toggleSpace(key: string) { if (expanded.has(key)) expanded.delete(key); else expanded.add(key); }
/** Keep the folder of the open conversation unfolded. */
watch(() => store.chat.currentId, revealSession);
function waiting(id: string) { const view = store.chat.views[id]; return !!view && view.approvals.length + view.questions.length > 0; }
function spaceWaiting(space: Space) { return space.sessions.some(session => waiting(session.id)); }
function hiddenSessionsWaiting(space: Space) { return !expandedSessionLists.has(space.key) && space.sessions.slice(displayLimit).some(session => waiting(session.id)); }
const hiddenProjectsWaiting = computed(() => !showAllProjects.value && projectSpaces.value.slice(displayLimit).some(spaceWaiting));

function navigate(page: ControlPage) { store.navigate(page); document.querySelector('.control-workspace')?.scrollTo(0, 0); }
function preload(page: string) {
  if (page in pageLoaders) void pageLoaders[page as keyof typeof pageLoaders]().catch(() => {});
}
async function newTask() { if (store.chat.currentId) await store.chat.select(null); navigate('new-task'); }
async function openSession(id: string) { navigate('chat'); await store.chat.select(id); }
async function addWorkspace() {
  if (!store.desktop) { store.notify('请在桌面应用中选择工作区文件夹', 'info'); return; }
  const path = await store.chat.pickWorkspace();
  if (!path) return;
  if (store.chat.currentId) await store.chat.select(null);
  if (await store.chat.switchWorkspace(path) === 'done') { restoreWorkspace(path); workspacesOpen.value = true; expanded.add(path); if (projectSpaces.value.findIndex(space => space.path === path) >= displayLimit) showAllProjects.value = true; navigate('new-task'); }
}
const workspaceMenu = ref<{ key: string; x: number; y: number; confirming: boolean } | null>(null);
const workspaceMenuEl = ref<HTMLElement | null>(null);
const menuWorkspace = computed(() => spaces.value.find(space => space.key === workspaceMenu.value?.key));
const renamingWorkspace = ref('');
const renamingWorkspaceTitle = computed(() => renamingWorkspace.value ? folderName(renamingWorkspace.value) : '');
function closeWorkspaceMenu() { workspaceMenu.value = null; }
async function openWorkspaceMenu(space: Space, event: MouseEvent, anchor?: EventTarget | null) {
  if (!space.path) return;
  cancelWorkspaceSort();
  closeMenu();
  closeSessionMenu();
  if (workspaceMenu.value?.key === space.key && anchor) { closeWorkspaceMenu(); return; }
  const box = anchor instanceof HTMLElement ? anchor.getBoundingClientRect() : null;
  workspaceMenu.value = { key: space.key, x: box ? box.left : event.clientX, y: box ? box.bottom + 4 : event.clientY, confirming: false };
  await placeWorkspaceMenu();
}
async function placeWorkspaceMenu() {
  await nextTick();
  const menu = workspaceMenu.value, el = workspaceMenuEl.value;
  if (!menu || !el) return;
  const { width, height } = el.getBoundingClientRect();
  menu.x = Math.max(8, Math.min(menu.x, window.innerWidth - width - 8));
  menu.y = Math.max(8, Math.min(menu.y, window.innerHeight - height - 8));
}
function startWorkspaceRename() {
  renamingWorkspace.value = workspaceMenu.value?.key || '';
  closeWorkspaceMenu();
}
const creatingWorkspaceSession = ref(false);
async function newWorkspaceSession() {
  const path = menuWorkspace.value?.path;
  if (!path || creatingWorkspaceSession.value) return;
  creatingWorkspaceSession.value = true;
  closeWorkspaceMenu();
  try {
    await store.chat.select(null);
    if (await store.chat.switchWorkspace(path) !== 'done') return;
    workspacesOpen.value = true;
    expanded.add(path);
    navigate('new-task');
  } catch { store.notify('新建会话失败，请重试', 'error'); }
  finally { creatingWorkspaceSession.value = false; }
}
function deleteWorkspace() {
  const space = menuWorkspace.value;
  if (!space?.path) return;
  const paths = new Set(removedWorkspaces.value);
  paths.add(space.path);
  if (!saveRemovedWorkspaces(paths)) return;
  expanded.delete(space.key);
  expanded.add(RECENT);
  closeWorkspaceMenu();
  store.notify('项目已删除，任务记录和文件已保留', 'success');
}
function submitWorkspaceRename(value: string) {
  const path = renamingWorkspace.value, name = value.trim();
  if (!path || !name) return;
  const names = { ...workspaceNames.value, [path]: name };
  try {
    localStorage.setItem(WORKSPACE_NAMES_KEY, JSON.stringify(names));
    workspaceNames.value = names;
    renamingWorkspace.value = '';
  } catch { store.notify('名称保存失败，请重试', 'error'); }
}
/*
 * A conversation's menu in the space tree (hover ⋯ or right click): rename, move to a working folder, delete.
 * Working folders took the place of the old groups; groups the host still keeps are left as they are, just not shown.
 */
interface SessionMenu { id: string; x: number; y: number; confirming: boolean }
const sessionMenu = ref<SessionMenu | null>(null);
const sessionMenuEl = ref<HTMLElement | null>(null);
const menuSession = computed(() => store.chat.sessions.find(item => item.id === sessionMenu.value?.id) ?? null);
const menuBusy = computed(() => !!(sessionMenu.value && store.chat.views[sessionMenu.value.id]?.running));
const workspaceTargets = computed(() => spaces.value.filter(space => space.path));
function openSessionMenu(id: string, event: MouseEvent, anchor?: EventTarget | null) {
  closeMenu();
  closeWorkspaceMenu();
  if (sessionMenu.value?.id === id && anchor) { sessionMenu.value = null; return; }
  const box = anchor instanceof HTMLElement ? anchor.getBoundingClientRect() : null;
  sessionMenu.value = { id, x: box ? box.left : event.clientX, y: box ? box.bottom + 4 : event.clientY, confirming: false };
  void placeSessionMenu();
}
/** Keeps the menu inside the window, opening upwards near the bottom. */
async function placeSessionMenu() {
  await nextTick();
  const menu = sessionMenu.value, el = sessionMenuEl.value;
  if (!menu || !el) return;
  const { width, height } = el.getBoundingClientRect();
  menu.x = Math.max(8, Math.min(menu.x, window.innerWidth - width - 8));
  menu.y = Math.max(8, Math.min(menu.y, window.innerHeight - height - 8));
}
function closeSessionMenu() { sessionMenu.value = null; }
const renamingId = ref('');
const renameBusy = ref(false);
const renamingTitle = computed(() => store.chat.sessions.find(item => item.id === renamingId.value)?.title ?? '');
function startRename() { if (sessionMenu.value) renamingId.value = sessionMenu.value.id; closeSessionMenu(); }
async function submitRename(value: string) {
  const id = renamingId.value;
  if (!id || renameBusy.value) return;
  renameBusy.value = true;
  const done = value.trim() === renamingTitle.value ? true : await store.chat.rename(id, value);
  renameBusy.value = false;
  if (done) renamingId.value = '';
}
async function moveSession(path: string | null) {
  const id = sessionMenu.value?.id;
  closeSessionMenu();
  if (!id) return;
  if (!store.desktop) { store.notify('请在桌面应用中选择工作区文件夹', 'info'); return; }
  const target = path ?? await store.chat.pickWorkspace();
  if (!target) return;
  if (await store.chat.moveToWorkspace(id, target)) { restoreWorkspace(target); expanded.add(target); store.notify(`已移动到「${folderName(target)}」`, 'success'); }
}
async function deleteSession() {
  const id = sessionMenu.value?.id;
  closeSessionMenu();
  if (!id) return;
  await store.chat.remove(id);
  if (store.page === 'chat' && !store.chat.currentId) navigate('new-task');
}

/** The first message of a new task creates a conversation: from then on it is that conversation's page. */
watch(() => store.chat.currentId, id => { if (id && store.page === 'new-task') store.navigate('chat'); });

/* ── Account menu ────────────────────────────────────────────────────── */
const menuOpen = ref(false);
const menuRoot = ref<HTMLElement | null>(null);
function toggleMenu() { menuOpen.value = !menuOpen.value; }
function closeMenu() { menuOpen.value = false; }
function onPointerDown(event: PointerEvent) {
  if (menuOpen.value && menuRoot.value && !menuRoot.value.contains(event.target as Node)) closeMenu();
  const target = event.target as Element | null;
  if (sessionMenu.value && !sessionMenuEl.value?.contains(target) && !target?.closest?.('.nav-session-more')) closeSessionMenu();
  if (workspaceMenu.value && !workspaceMenuEl.value?.contains(target) && !target?.closest?.('.nav-space-more')) closeWorkspaceMenu();
}
const displayName = computed(() => store.account?.user.username ?? '未登录');
const modeLabel = computed(() => appearanceModes.find(mode => mode.value === appearanceMode.value)?.label ?? '');
const unread = computed(() => store.website.announcements.filter(item => store.website.dismissed[item.id] !== item.revision).length);
const hasUpdate = computed(() => !!store.website.update?.available);
function openSettings(tab?: Parameters<typeof store.openSettings>[0]) { closeMenu(); store.openSettings(tab); }
function login() { closeMenu(); store.openLogin(); }
const loggingOut = ref(false);
async function handleLogout() {
  if (loggingOut.value) return;
  loggingOut.value = true;
  try {
    await store.logoutAccount();
    closeMenu();
    store.notify('已退出当前账号', 'info');
  } catch (error) {
    store.notify(error, 'error');
  } finally {
    loggingOut.value = false;
  }
}

const assistantShortcut = ref(mac.value ? '⌥ Space' : 'Alt+Space');
const openingAssistant = ref(false);
async function openAssistant() {
  closeMenu();
  if (openingAssistant.value) return;
  if (!store.desktop) { store.notify('超级小伴需要在桌面应用中使用', 'info'); return; }
  if (!store.settings.assistantEnabled) {
    store.openSettings('assistant');
    store.notify('开启超级小伴后即可使用悬浮对话', 'info'); return;
  }
  openingAssistant.value = true;
  try { await invoke('assistant_try'); }
  catch (error) { store.notify(error, 'error'); }
  finally { openingAssistant.value = false; }
}
async function openOfficialSite() {
  closeMenu();
  if (store.desktop) {
    store.notify('正在通过系统浏览器打开小伴官网...', 'info');
    try {
      await invoke('open_official_site');
    } catch (error) {
      store.notify(error, 'error');
    }
  } else {
    window.location.href = 'https://wallpaper.xiangban.xyz';
  }
}

/* ── Studio requests from desktop widgets and pets ───────────────────── */
const petSettingsRequest = ref<{ id: string } | null>(null);
function managePets(event: Event) { petSettingsRequest.value = (event as CustomEvent<{ id: string }>).detail; navigate('companion'); }
const widgetSettingsRequest = ref<{ id: string } | null>(null);
function manageWidgets(event: Event) {
  const id = (event as CustomEvent<{ id?: string }>).detail?.id;
  widgetSettingsRequest.value = typeof id === 'string' && id ? { id } : null;
  navigate('widgets');
}

/**
 * The colour scheme is kept by the host so the floating assistant, which cannot read this window's storage, can match.
 * The first time, what this window already had is sent up (it is the user's choice from before the host kept one);
 * after that the host's value rules, so a change made anywhere reaches every window.
 */
async function shareAppearance() {
  publishAppearanceTo(mode => { store.setAppearance(mode).catch(error => store.notify(error, 'error')); });
  // The theme colour lives in this window's storage; the host only relays it to the floating assistant.
  publishAccentTo(id => { void invoke('set_accent', { accent: id }).catch(() => {}); });
  void invoke('set_accent', { accent: accent.value }).catch(() => {});
  try {
    const kept = await invoke<{ appearance: string; appearanceKept?: boolean }>('get_appearance');
    const step = reconcile(kept, appearanceMode.value);
    if (step.send) await store.setAppearance(step.send);
    else if (step.adopt) appearanceMode.value = step.adopt;
  } catch { /* this window keeps its own copy */ }
}
/** Once, after the floating assistant first exists, say how to call it: most people never open its settings. */
async function introduceAssistant() {
  try {
    const assistant = await invoke<{ enabled: boolean; hintSeen: boolean; shortcutLabel: string; ball: boolean }>('assistant_status');
    if (assistant?.shortcutLabel) assistantShortcut.value = assistant.shortcutLabel;
    if (!assistant || assistant.hintSeen || !assistant.enabled) return;
    store.notify(`新功能：按 ${assistant.shortcutLabel} 随时呼出超级小伴${assistant.ball ? '，也可以点屏幕边缘的小球' : ''}`, 'info');
    await invoke('assistant_configure', { patch: { hintSeen: true } });
  } catch { /* the hint is a courtesy; it can wait for the next start */ }
}
/** The assistant's `open_page` tool: a page of the sidebar, a tab of 装扮, or a category of the settings dialog. */
function openPageFromAssistant(page: string, tab?: string) {
  if (page === 'settings') { store.openSettings(tab as SettingsTab | undefined); return; }
  store.closeSettings();
  if (page === 'studio' && tab) navigate(tab === 'pets' ? 'companion' : tab as ControlPage);
  else navigate(page as ControlPage);
}
const assistantListeners: (() => void)[] = [];
async function listenForAssistant() {
  const off = await Promise.all([
    listen<{ session: string | null }>('assistant-open-chat', ({ session }) => { if (session) void openSession(session); else void newTask(); }),
    listen<{ tab: 'ai' | 'assistant' }>('assistant-open-settings', ({ tab }) => { store.openSettings(tab); }),
    listen<{id:string;session:string}>('design-open', async ({id,session}) => {
      if(store.chat.sessions.some(s=>s.id===session))await openSession(session);
      else {await store.chat.select(null);navigate('chat');}
      store.designOpening=id;
    }),
    listen<{id:string;session:string;message:string}>('design-request', async ({session,message}) => {
      try{
        if(store.chat.sessions.some(s=>s.id===session))await openSession(session);
        else {await store.chat.select(null);navigate('chat');}
        if(store.chat.currentId)await store.chat.enqueue(message,[],[]);
        else await store.chat.send(message);
      }catch(e){store.notify(e,'error');}
    }),
    listen<unknown>('games-open', () => { navigate('games'); }),
    listen<{ page: string; tab?: string | null }>('control-navigate', ({ page, tab }) => openPageFromAssistant(page, tab ?? undefined)),
  ]);
  assistantListeners.push(...off);
}
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && workspaceMenu.value) { closeWorkspaceMenu(); return; }
  if (event.key === 'Escape' && sessionMenu.value) { closeSessionMenu(); return; }
  if (event.key === 'Escape' && menuOpen.value) { closeMenu(); return; }
  if (!(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) return;
  if (event.key === ',') { event.preventDefault(); openSettings(); }
  else if (event.key.toLowerCase() === 'k') { event.preventDefault(); openSearch(); }
  else if (event.key.toLowerCase() === 'n') { event.preventDefault(); void newTask(); }
}
const focus = () => store.refreshSettings().catch(error => store.notify(error, 'error'));
onMounted(() => {
  if (typeof window !== 'undefined' && store.desktop) {
    // 拦截任何 window.open，强制通过系统浏览器打开，防止 WKWebView 产生 500x500 空白窗口
    window.open = (url) => {
      if (url) void invoke('open_download_url', { url: String(url) });
      return null;
    };
  }
  void store.initialize().then(async () => { if (store.info.uiSmoke) await (await import('./ui-smoke.ts')).runUiSmoke(store); });
  window.addEventListener('focus', focus);
  window.addEventListener('widgets-manage', manageWidgets);
  window.addEventListener('pets-manage', managePets);
  window.addEventListener('keydown', onKeydown);
  document.addEventListener('pointerdown', onPointerDown, true);
  if (store.desktop) { void listenForAssistant(); void introduceAssistant(); void shareAppearance(); }
});
onBeforeUnmount(() => {
  assistantListeners.splice(0).forEach(off => off()); store.dispose();
  window.removeEventListener('focus', focus); window.removeEventListener('widgets-manage', manageWidgets); window.removeEventListener('pets-manage', managePets);
  window.removeEventListener('keydown', onKeydown); document.removeEventListener('pointerdown', onPointerDown, true);
});
const fixedPages = ['new-task', 'chat', 'ai-store', 'library', 'studio', 'games'];
</script>
<template>
  <div class="control-app" :class="{ 'is-collapsed': collapsed, 'is-mac': mac && store.desktop, 'is-win': win }">
    <div class="window-drag" data-tauri-drag-region aria-hidden="true"></div>
    <div v-if="win" class="window-controls" data-testid="window-controls">
      <button type="button" aria-label="最小化" title="最小化" @click="windowAction('minimize')"><svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true"><path d="M0 5h10" stroke="currentColor" stroke-width="1"/></svg></button>
      <button type="button" aria-label="最大化" title="最大化" @click="windowAction('toggleMaximize')"><svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true"><rect x=".5" y=".5" width="9" height="9" rx="1" fill="none" stroke="currentColor" stroke-width="1"/></svg></button>
      <button type="button" class="is-close" aria-label="关闭" title="关闭" @click="windowAction('close')"><svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true"><path d="M0 0l10 10M10 0 0 10" stroke="currentColor" stroke-width="1"/></svg></button>
    </div>
    <aside class="control-sidebar" aria-label="侧栏">
      <div class="sidebar-top" data-tauri-drag-region>
        <button type="button" class="xb-icon-btn" :title="collapsed ? '展开侧栏' : '收起侧栏'" :aria-label="collapsed ? '展开侧栏' : '收起侧栏'" :aria-expanded="!collapsed" aria-controls="control-navigation" data-testid="sidebar-toggle" @click="toggleNavigation"><PanelLeft :size="17" /></button>
        <button type="button" class="xb-icon-btn sidebar-search" aria-label="搜索" title="搜索（即将推出）" disabled><Search :size="17" /></button>
      </div>
      <div class="control-brand">
        <span class="xb-mark"><Sparkles :size="19" :stroke-width="2" /></span>
        <div class="brand-copy"><strong>小伴桌面</strong><span>v{{ store.info.version }}</span></div>
      </div>

      <nav id="control-navigation" class="sidebar-scroll" aria-label="功能分区">
        <button type="button" class="nav-item nav-group" :aria-expanded="aiOpen" data-testid="nav-ai-group" :title="'小伴 AI'" @click="collapsed ? newTask() : (aiOpen = !aiOpen)">
          <Sparkles :size="17" class="nav-icon ink" /><span class="nav-label">小伴 AI</span>
          <i v-if="attention && (!aiOpen || collapsed)" class="xb-dot nav-attention" data-testid="chat-attention" :title="`有 ${attention} 个任务等你确认`" aria-label="小伴在等你确认"></i>
          <ChevronDown :size="13" class="nav-chevron" :class="{ shut: !aiOpen }" />
        </button>
        <div v-show="aiOpen || collapsed" class="nav-sub">
          <button v-for="item in aiItems" :key="item.id" type="button" class="nav-item" :class="{ selected: isCurrent(item.id) }" :aria-current="isCurrent(item.id) ? 'page' : undefined" :data-testid="`nav-${item.id}`" :title="item.label" @pointerenter="preload(item.id)" @focus="preload(item.id)" @click="item.id === 'new-task' ? newTask() : navigate(item.id)">
            <component :is="item.icon" :size="16" class="nav-icon" /><span class="nav-label">{{ item.label }}</span><span v-if="item.hint" class="nav-hint">{{ item.hint }}</span>
          </button>
          <div class="nav-caption"><button type="button" class="workspace-toggle" :aria-expanded="workspacesOpen" aria-controls="workspace-tree" data-testid="nav-workspaces-toggle" @click="toggleWorkspaces"><span>工作空间</span><ChevronDown :size="12" class="nav-chevron" :class="{ shut: !workspacesOpen }" /></button><span class="workspace-tools"><button type="button" class="xb-icon-btn is-sm" aria-label="搜索工作空间" title="搜索工作空间" data-testid="nav-workspace-search" @click="openSearch"><Search :size="14" /></button><button type="button" class="xb-icon-btn is-sm" aria-label="新建工作区" title="新建工作区" data-testid="nav-new-workspace" @click="addWorkspace"><Plus :size="14" :stroke-width="2" /></button></span></div>
          <div v-show="workspacesOpen" id="workspace-tree" ref="workspaceTree" :class="{ 'is-sorting': draggingWorkspace }" @click.capture="workspaceSortClick">
          <div v-for="space in visibleSpaces" :key="space.key" class="nav-space-group" :data-workspace-path="space.path || undefined" :class="{ 'is-dragging': !!space.path && draggingWorkspace === space.path, 'drop-before': workspaceDropTarget?.path === space.path && !workspaceDropTarget.after, 'drop-after': workspaceDropTarget?.path === space.path && workspaceDropTarget.after }">
            <div class="nav-space-row" :class="{ 'menu-open': workspaceMenu?.key === space.key }" @contextmenu.prevent="openWorkspaceMenu(space, $event)" @dragstart.prevent>
            <button type="button" class="nav-item space-row" :class="{ 'is-sortable': space.path }" :aria-expanded="expanded.has(space.key)" :title="space.path ? `${space.path}（拖动排序）` : space.name" :data-testid="space.key === '__recent__' ? 'space-recent' : `space-${space.name}`" @pointerdown="startWorkspaceSort(space.path, $event)" @click="toggleSpace(space.key)">
              <component :is="space.key === '__recent__' ? Inbox : space.key === '__companions__' ? Sparkles : Folder" :size="16" class="nav-icon" /><span class="nav-label">{{ space.name }}</span>
              <i v-if="!expanded.has(space.key) && spaceWaiting(space)" class="xb-dot"></i>
              <span class="nav-hint">{{ space.sessions.length || '' }}</span>
            </button>
            <button v-if="space.path" type="button" class="xb-icon-btn is-sm nav-space-more" aria-haspopup="menu" :aria-expanded="workspaceMenu?.key === space.key" :aria-label="`更多操作：${space.name}`" title="更多" :data-testid="`space-more-${space.name}`" @click.stop="openWorkspaceMenu(space, $event, $event.currentTarget)"><Ellipsis :size="14" /></button>
            </div>
            <template v-if="expanded.has(space.key)">
              <div v-for="session in visibleSessions(space)" :key="session.id" class="nav-session-row" :class="{ 'menu-open': sessionMenu?.id === session.id }" @contextmenu.prevent="openSessionMenu(session.id, $event)">
                <button type="button" class="nav-session" :class="{ selected: store.page === 'chat' && store.chat.currentId === session.id }" :aria-current="store.page === 'chat' && store.chat.currentId === session.id ? 'page' : undefined" :data-testid="`session-${session.id}`" :data-session-id="session.id" :title="session.title" @click="openSession(session.id)">
                  <span>{{ session.title || '新对话' }}</span><i v-if="waiting(session.id)" class="xb-dot" aria-label="等你确认"></i>
                </button>
                <button type="button" class="xb-icon-btn is-sm nav-session-more" aria-haspopup="menu" :aria-expanded="sessionMenu?.id === session.id" :aria-label="`更多操作：${session.title || '新对话'}`" title="更多" :data-testid="`session-more-${session.id}`" @click.stop="openSessionMenu(session.id, $event, $event.currentTarget)"><Ellipsis :size="14" /></button>
              </div>
              <button v-if="space.sessions.length > displayLimit" type="button" class="workspace-more sessions-more" :aria-expanded="expandedSessionLists.has(space.key)" :data-testid="`sessions-more-${space.key}`" @click="expandedSessionLists.has(space.key) ? expandedSessionLists.delete(space.key) : expandedSessionLists.add(space.key)"><ChevronDown :size="12" /><span>{{ expandedSessionLists.has(space.key) ? '收起会话' : '展开全部会话' }}</span><i v-if="hiddenSessionsWaiting(space)" class="xb-dot" aria-label="折叠的会话中有待确认操作" /><small>{{ expandedSessionLists.has(space.key) ? `显示 ${displayLimit} 个` : `还有 ${space.sessions.length - displayLimit} 个` }}</small></button>
              <p v-if="!space.sessions.length" class="nav-empty">{{ space.key === '__recent__' ? '还没有任务' : '新任务会出现在这里' }}</p>
            </template>
          </div>
          <button v-if="projectSpaces.length > displayLimit" type="button" class="workspace-more" :aria-expanded="showAllProjects" aria-controls="workspace-tree" data-testid="workspace-projects-more" @click="cancelWorkspaceSort(); showAllProjects = !showAllProjects"><ChevronDown :size="12" /><span>{{ showAllProjects ? '收起项目' : '展开全部项目' }}</span><i v-if="hiddenProjectsWaiting" class="xb-dot" aria-label="折叠的项目中有待确认操作" /><small>{{ showAllProjects ? `显示 ${displayLimit} 个` : `还有 ${projectSpaces.length - displayLimit} 个` }}</small></button>
          </div>
        </div>
        <div class="nav-sep"></div>
        <button v-for="item in bottomItems" :key="item.id" type="button" class="nav-item" :class="{ selected: isCurrent(item.id) }" :aria-current="isCurrent(item.id) ? 'page' : undefined" :data-testid="`nav-${item.id}`" :title="item.label" @pointerenter="preload(item.id)" @focus="preload(item.id)" @click="navigate(item.id)">
          <component :is="item.icon" :size="17" class="nav-icon" /><span class="nav-label">{{ item.label }}</span>
        </button>
      </nav>

      <div ref="menuRoot" class="sidebar-bottom">
        <button type="button" class="account-chip" data-testid="account-entry" :aria-label="store.account ? `账号菜单：${displayName}` : '账号菜单（未登录）'" aria-haspopup="menu" :aria-expanded="menuOpen" @click="toggleMenu">
          <span class="account-avatar" :class="{ 'is-signed-in': store.account, 'is-placeholder': !store.account?.user.avatarUrl }"><img v-if="store.account?.user.avatarUrl" class="brand-avatar" :src="store.account.user.avatarUrl" alt="" /><Sparkles v-else :size="18" :stroke-width="2" aria-hidden="true" /></span>
          <span class="nav-label account-name">{{ displayName }}</span>
        </button>
        <button type="button" class="xb-icon-btn notices" aria-label="通知" :title="hasUpdate ? '有新版本' : unread ? `${unread} 条未读公告` : '通知'" data-testid="nav-notices" @click="openSettings('about')"><Bell :size="16" /><i v-if="hasUpdate || unread" class="xb-dot notices-dot"></i></button>

        <div v-if="menuOpen" class="xb-menu account-menu" role="menu" aria-label="账号菜单" data-testid="account-menu">
          <div v-if="store.account" class="account-head" data-testid="account-info">
            <span class="account-avatar lg" :class="{ 'is-placeholder': !store.account.user.avatarUrl }"><img v-if="store.account.user.avatarUrl" :src="store.account.user.avatarUrl" alt="" /><Sparkles v-else :size="22" :stroke-width="2" aria-hidden="true" /></span>
            <div><b data-testid="account-username">{{ store.account.user.username }}</b><span>账号 ID <span data-testid="account-uid">{{ store.account.user.uid }}</span></span></div>
          </div>
          <button v-else type="button" class="xb-menu-item account-login" role="menuitem" data-testid="account-login" @click="login">
            <span class="account-avatar lg is-placeholder"><Sparkles :size="22" :stroke-width="2" aria-hidden="true" /></span>
            <span class="login-copy"><b>登录 / 注册</b><small>登录后即可使用小伴 AI</small></span><LogIn :size="15" class="xb-menu-hint" />
          </button>
          <div class="xb-menu-sep"></div>
          <button type="button" class="xb-menu-item" role="menuitem" data-testid="nav-settings" @click="openSettings()"><SlidersHorizontal :size="16" />设置<span class="xb-menu-hint">{{ mod }},</span></button>
          <button type="button" class="xb-menu-item" role="menuitem" data-testid="nav-appearance" @click="openSettings('appearance')"><Palette :size="16" />外观<span class="xb-menu-hint">{{ modeLabel }}</span></button>
          <div class="accent-row" role="radiogroup" aria-label="主题色">
            <button v-for="item in accents" :key="item.id" type="button" role="radio" class="accent-dot" :class="{ on: accent === item.id }" :aria-checked="accent === item.id" :aria-label="item.name" :title="item.name" :data-testid="`accent-${item.id}`" :style="{ background: (isDark ? item.dark : item.light).btn, '--ring': (isDark ? item.dark : item.light).btn }" @click="setAccent(item.id)"></button>
          </div>
          <button type="button" class="xb-menu-item" role="menuitem" data-testid="open-assistant" :disabled="openingAssistant" @click="openAssistant"><Zap :size="16" />超级小伴<span class="xb-menu-hint">{{ assistantShortcut }}</span></button>
          <button type="button" class="xb-menu-item" role="menuitem" data-testid="nav-check-update" @click="openSettings('about')"><RefreshCw :size="16" />检查更新<em v-if="hasUpdate" class="xb-menu-hint update-mark" data-testid="nav-settings-new">新版本</em><i v-else-if="unread" class="xb-dot xb-menu-hint" data-testid="nav-notice-dot"></i></button>
          <button type="button" class="xb-menu-item" role="menuitem" data-testid="nav-website" @click="openOfficialSite"><Globe :size="16" />小伴官网<span class="xb-menu-hint">↗</span></button>
          <template v-if="store.account">
            <div class="xb-menu-sep"></div>
            <button type="button" class="xb-menu-item" role="menuitem" data-testid="account-logout" :disabled="loggingOut" @click="handleLogout"><LogOut :size="16" />退出登录</button>
          </template>
        </div>
      </div>
    </aside>
    <div v-if="workspaceMenu" ref="workspaceMenuEl" class="xb-menu session-menu" role="menu" aria-label="工作空间项目" data-testid="workspace-menu" :style="{ left: `${workspaceMenu.x}px`, top: `${workspaceMenu.y}px` }">
      <template v-if="!workspaceMenu.confirming">
      <button type="button" class="xb-menu-item" role="menuitem" data-testid="workspace-menu-new-session" :disabled="creatingWorkspaceSession" @click="newWorkspaceSession"><SquarePen :size="15" />新建会话</button>
      <button type="button" class="xb-menu-item" role="menuitem" data-testid="workspace-menu-rename" @click="startWorkspaceRename"><Pencil :size="15" />重命名</button>
      <div class="xb-menu-sep"></div>
      <button type="button" class="xb-menu-item is-danger" role="menuitem" data-testid="workspace-menu-delete" @click="workspaceMenu.confirming = true; placeWorkspaceMenu()"><Trash2 :size="15" />删除</button>
      </template>
      <div v-else class="session-confirm" role="alertdialog" aria-label="删除项目" data-testid="workspace-delete-confirm">
        <b>删除「{{ menuWorkspace?.name }}」？</b>
        <span>从工作空间移除此项目。{{ menuWorkspace?.sessions.length ? `${menuWorkspace.sessions.length} 个任务将保留在“最近任务”中。` : '' }}实际文件夹和文件不会删除。</span>
        <div class="session-confirm-actions">
          <button type="button" class="xb-btn is-sm" @click="closeWorkspaceMenu">取消</button>
          <button type="button" class="xb-btn is-sm is-danger" data-testid="workspace-delete-yes" @click="deleteWorkspace">删除项目</button>
        </div>
      </div>
    </div>
    <div v-if="sessionMenu" ref="sessionMenuEl" class="xb-menu session-menu" role="menu" :aria-label="`对话：${menuSession?.title || '新对话'}`" data-testid="session-menu" :style="{ left: `${sessionMenu.x}px`, top: `${sessionMenu.y}px` }">
      <template v-if="!sessionMenu.confirming">
        <button type="button" class="xb-menu-item" role="menuitem" data-testid="session-menu-rename" @click="startRename"><Pencil :size="15" />重命名</button>
        <div class="xb-menu-sep"></div>
        <div class="xb-menu-label">移动到工作区</div>
        <button v-for="space in workspaceTargets" :key="space.key" type="button" class="xb-menu-item" role="menuitemradio" :aria-checked="menuSession?.workspace === space.path" :disabled="menuBusy" :title="space.path" :data-testid="`session-move-${space.name}`" @click="moveSession(space.path)">
          <Folder :size="15" /><span class="session-menu-name">{{ space.name }}</span><Check v-if="menuSession?.workspace === space.path" :size="14" class="xb-menu-check" />
        </button>
        <button type="button" class="xb-menu-item" role="menuitem" :disabled="menuBusy" data-testid="session-move-pick" @click="moveSession(null)"><FolderPlus :size="15" />选择文件夹…</button>
        <div class="xb-menu-sep"></div>
        <button type="button" class="xb-menu-item is-danger" role="menuitem" :disabled="menuBusy" :title="menuBusy ? '回答结束后才能删除' : undefined" data-testid="session-menu-delete" @click="sessionMenu.confirming = true; placeSessionMenu()"><Trash2 :size="15" />删除</button>
      </template>
      <div v-else class="session-confirm" role="alertdialog" aria-label="删除对话" data-testid="session-delete-confirm">
        <b>删除「{{ menuSession?.title || '新对话' }}」？</b>
        <span>对话记录会从这台电脑上清除，无法恢复。</span>
        <div class="session-confirm-actions">
          <button type="button" class="xb-btn is-sm" @click="closeSessionMenu">取消</button>
          <button type="button" class="xb-btn is-sm is-danger" data-testid="session-delete-yes" @click="deleteSession">删除</button>
        </div>
      </div>
    </div>
    <ChatNameDialog :show="!!renamingId || !!renamingWorkspace" :title="renamingWorkspace ? '重命名项目' : '重命名对话'" :placeholder="renamingWorkspace ? '给这个项目起个名字' : '给这个对话起个名字'" :initial="renamingWorkspace ? renamingWorkspaceTitle : renamingTitle" :max-length="60" confirm-text="保存" :busy="renameBusy" @close="renamingId = ''; renamingWorkspace = ''" @submit="value => renamingWorkspace ? submitWorkspaceRename(value) : submitRename(value)" />
    <div class="control-main">
      <main class="control-workspace" :class="{ 'fixed-page-workspace': fixedPages.includes(store.page) }">
        <NAlert v-if="store.initialError" class="workspace-alert" type="error" title="桌面配置暂时无法读取">{{ store.initialError }} <NButton size="small" @click="store.initialize()">重试</NButton></NAlert>
        <div v-if="!store.ready" class="workspace-loading xb-page"><NSkeleton height="32px" width="220px" /><NSkeleton height="18px" width="320px" /><NSkeleton height="380px" :sharp="false" /></div>
        <template v-else-if="!store.initialError"><KeepAlive :max="5">
          <NewTaskPage v-if="store.page === 'new-task'" :store="store" />
          <ChatPage v-else-if="store.page === 'chat'" :store="store" />
          <AiStorePage v-else-if="store.page === 'ai-store'" :store="store" />
          <LibraryPage v-else-if="store.page === 'library'" :store="store" />
          <StudioPage v-else-if="store.page === 'studio'" :store="store" :widget-settings-request="widgetSettingsRequest" :pet-settings-request="petSettingsRequest" @widget-settings-opened="widgetSettingsRequest = null" @pet-settings-opened="petSettingsRequest = null" />
          <InteractivePage v-else-if="store.page === 'interactive'" :store="store" />
          <GamesPage v-else-if="store.page === 'games'" :store="store" />
          <CarePage v-else-if="store.page === 'care'" :store="store" />
          <CommunityPage v-else-if="store.page === 'community'" :store="store" />
        </KeepAlive></template>
      </main>
    </div>
    <SettingsDialog :store="store" @open-site="openOfficialSite" />
    <WorkspaceSearchDialog v-model:show="searchOpen" :desktop="store.desktop" :spaces="spaces" @select="selectSearchResult" />
    <LoginModal :store="store" />
    <CommunityApplyDialog :store="store" />
  </div>
</template>
<style>
.workspace-tools{display:flex;align-items:center;gap:3px}
.workspace-more{display:flex;align-items:center;gap:6px;min-height:28px;width:calc(100% - 28px);margin:4px 0 4px 22px;padding:5px 9px;border:0;border-radius:6px;background:none;color:var(--ui-text-secondary);font:inherit;font-size:11px;text-align:left;cursor:pointer}.workspace-more:hover{background:var(--ui-surface);color:var(--ui-text-primary)}.workspace-more svg{flex:none;color:var(--ui-text-tertiary)}.workspace-more[aria-expanded=true] svg{transform:rotate(180deg)}.workspace-more small{margin-left:auto;white-space:nowrap;font-size:10px;color:var(--ui-text-tertiary)}.sessions-more{margin-left:35px;width:calc(100% - 41px)}
/* Sidebar (方案四): tint background, 264 wide, soft + ink selection. */
.control-sidebar{position:relative;display:flex;flex-direction:column;min-height:0;padding:0 12px 12px;background:var(--ui-tint);color:var(--ui-text-primary)}
.sidebar-top{height:48px;flex-shrink:0;display:flex;align-items:center;justify-content:flex-end;gap:2px}
.sidebar-top .xb-icon-btn:disabled{opacity:.45}
.control-brand{display:flex;align-items:center;gap:10px;padding:2px 6px 16px;flex-shrink:0}
.brand-copy{display:flex;flex-direction:column;line-height:1.25;white-space:nowrap;min-width:0}
.brand-copy strong{font-size:15px;font-weight:600}
.brand-copy span{font-size:11px;color:var(--ui-text-secondary)}
.sidebar-scroll{flex:1;min-height:0;display:flex;flex-direction:column;gap:1px;overflow-y:auto;overflow-x:hidden;margin:0 -4px;padding:0 4px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.nav-item{position:relative;display:flex;align-items:center;gap:9px;width:100%;height:32px;flex-shrink:0;padding:0 10px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-primary);font-size:14px;text-align:left;cursor:pointer}
.nav-item:hover{background:var(--ui-surface-hover)}
.nav-item .nav-icon{flex-shrink:0;color:var(--ui-text-secondary)}
.nav-item .nav-icon.ink{color:var(--ui-ink)}
.nav-item.selected{background:var(--ui-soft);color:var(--ui-ink);font-weight:500}
.nav-item.selected .nav-icon{color:var(--ui-ink)}
.nav-label{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.nav-hint{margin-left:auto;font-size:11.5px;color:var(--ui-text-secondary);white-space:nowrap}
.nav-group{font-weight:600}
.nav-group .nav-attention{margin-left:auto}
.nav-chevron{flex-shrink:0;color:var(--ui-text-tertiary);transition:transform 160ms}
.nav-chevron.shut{transform:rotate(-90deg)}
.nav-sub{display:flex;flex-direction:column;gap:1px;padding-left:10px}
.nav-caption{display:flex;align-items:center;height:30px;margin-top:6px;padding:0 4px 0 10px;font-size:12px;font-weight:500;color:var(--ui-text-secondary)}
.nav-caption .xb-icon-btn{margin-left:auto}
.workspace-toggle{display:flex;align-items:center;gap:6px;flex:1;align-self:stretch;padding:0;border:0;background:none;color:inherit;font:inherit;text-align:left;cursor:pointer}
.workspace-toggle:hover{color:var(--ui-text-primary)}
.workspace-toggle .nav-chevron{margin-left:0}
.nav-space-row{position:relative;display:flex;align-items:center}
.space-row.is-sortable{cursor:grab;touch-action:pan-y;user-select:none}
#workspace-tree.is-sorting,#workspace-tree.is-sorting *{cursor:grabbing;user-select:none}
.nav-space-group{position:relative}
.nav-space-group.is-dragging{opacity:.4}
.nav-space-group.drop-before::before,.nav-space-group.drop-after::after{content:'';position:absolute;left:10px;right:4px;height:2px;border-radius:2px;background:var(--ui-btn);pointer-events:none;z-index:1}
.nav-space-group.drop-before::before{top:0}
.nav-space-group.drop-after::after{bottom:0}
.nav-space-row .space-row{flex:1;min-width:0}
.nav-space-more{position:absolute;right:4px;opacity:0;pointer-events:none;background:var(--ui-surface);color:var(--ui-text-secondary)}
.is-collapsed .nav-space-more{display:none}
.nav-space-row:hover .nav-space-more,.nav-space-row:focus-within .nav-space-more,.nav-space-row.menu-open .nav-space-more{opacity:1;pointer-events:auto}
.nav-space-row:hover .space-row,.nav-space-row:focus-within .space-row,.nav-space-row.menu-open .space-row{padding-right:32px}
.nav-space-row:hover .nav-hint,.nav-space-row:focus-within .nav-hint,.nav-space-row.menu-open .nav-hint{display:none}
.space-row .xb-dot{margin-left:auto}
.space-row .xb-dot+.nav-hint{margin-left:6px}
.nav-session{display:flex;align-items:center;gap:8px;width:100%;height:30px;flex-shrink:0;padding:0 10px 0 36px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-primary);opacity:.86;font-size:13px;text-align:left;cursor:pointer}
.nav-session:hover{background:var(--ui-surface-hover);opacity:1}
.nav-session>span{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.nav-session.selected{background:var(--ui-soft);color:var(--ui-ink);opacity:1;font-weight:500}
.nav-session-row{position:relative;flex-shrink:0}
.nav-session-row>.nav-session-more{position:absolute;top:3px;right:4px;opacity:0;pointer-events:none}
.nav-session-row:hover .nav-session-more,.nav-session-row:focus-within .nav-session-more,.nav-session-row.menu-open .nav-session-more{opacity:1;pointer-events:auto}
.nav-session-row:hover .nav-session,.nav-session-row:focus-within .nav-session,.nav-session-row.menu-open .nav-session{padding-right:32px}
.nav-session-row.menu-open .nav-session{background:var(--ui-surface-hover);opacity:1}
.nav-session-row.menu-open .nav-session.selected{background:var(--ui-soft)}
.session-menu{position:fixed;width:232px;z-index:60}
.session-menu .xb-menu-item:disabled{opacity:.45;cursor:not-allowed}
.session-menu-name{flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.session-confirm{display:flex;flex-direction:column;gap:4px;padding:8px 10px}
.session-confirm b{font-size:13.5px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.session-confirm span{font-size:12px;line-height:1.5;color:var(--ui-text-secondary)}
.session-confirm-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:8px}
.nav-empty{margin:0;padding:4px 10px 4px 36px;font-size:12px;color:var(--ui-text-tertiary)}
.nav-sep{height:1px;flex-shrink:0;margin:8px 10px;background:var(--ui-line)}

.sidebar-bottom{position:relative;display:flex;align-items:center;gap:6px;padding-top:10px;border-top:1px solid var(--ui-line);flex-shrink:0}
.account-chip{flex:1;min-width:0;display:flex;align-items:center;gap:10px;height:40px;padding:0 6px;border:0;border-radius:9px;background:transparent;color:var(--ui-text-primary);text-align:left;cursor:pointer}
.account-chip:hover,.account-chip[aria-expanded=true]{background:var(--ui-surface-hover)}
.account-name{font-size:13px;font-weight:500}
.account-avatar{width:30px;height:30px;flex-shrink:0;border-radius:50%;overflow:hidden;background:#fff;box-shadow:0 0 0 1px rgba(0,0,0,.08);display:grid;place-items:center}
.account-avatar.is-signed-in{box-shadow:0 0 0 2px var(--ui-btn)}
.account-avatar img{width:100%;height:100%;object-fit:cover}
.account-avatar.is-placeholder{color:#fff;background:linear-gradient(160deg,var(--ui-btn-hi),var(--ui-btn));box-shadow:inset 0 1px 0 rgba(255,255,255,.3),0 2px 6px rgba(0,0,0,.12)}
.account-avatar.lg{width:38px;height:38px}
.notices-dot{position:absolute;top:6px;right:6px;width:6px;height:6px}
.account-menu{position:absolute;left:-2px;bottom:52px;width:300px;padding:8px;z-index:50}
.account-head{display:flex;align-items:center;gap:10px;padding:8px 10px 12px}
.account-head>div{display:flex;flex-direction:column;min-width:0}
.account-head b{font-size:14px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.account-head>div>span{font-size:12px;color:var(--ui-text-secondary);overflow-wrap:anywhere}
.account-login{min-height:54px;padding:6px 10px}
.login-copy{display:flex;flex-direction:column;line-height:1.35}
.login-copy b{font-size:14px;font-weight:600;color:var(--ui-ink)}
.login-copy small{font-size:12px;color:var(--ui-text-secondary)}
.account-menu .xb-menu-item:disabled{opacity:.5}
.update-mark{color:var(--ui-ink)!important;font-weight:500;font-style:normal}
.accent-row{display:flex;align-items:center;gap:9px;padding:4px 10px 10px 36px}
.accent-dot{width:14px;height:14px;padding:0;border:0;border-radius:50%;box-shadow:inset 0 0 0 1px rgba(0,0,0,.1);cursor:pointer}
:root[data-theme=dark] .accent-dot{box-shadow:inset 0 0 0 1px rgba(255,255,255,.18)}
.accent-dot.on,:root[data-theme=dark] .accent-dot.on{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 3.5px var(--ring)}
.accent-dot:hover{transform:scale(1.12)}

/* Whole sidebar folded to an icon rail. */
.is-collapsed .control-sidebar{padding:0 8px 12px}
.is-collapsed .sidebar-top{justify-content:center}
.window-drag{position:absolute;top:0;left:0;right:0;height:8px;z-index:20}
/* Native macOS traffic lights need 88px even when the navigation is folded. */
.control-app.is-mac.is-collapsed{--ui-sidebar-collapsed:88px}
.is-mac .sidebar-top{padding-left:68px}
.is-mac.is-collapsed .sidebar-top{height:72px;padding-left:0;align-items:flex-end}
/* Keep Windows caption controls inside the content panel, without a separate title strip. */
.is-win .control-main{margin-top:8px;padding-top:26px}
.is-win .sidebar-top{height:48px}
.is-win .window-drag{top:8px;left:var(--ui-sidebar-width);right:122px;height:26px}
.is-win.is-collapsed .window-drag{left:var(--ui-sidebar-collapsed)}
.is-win .window-controls{top:8px;right:8px;height:26px;overflow:hidden;border-top-right-radius:12px}
.is-win .window-controls button{width:38px;height:26px}
.window-controls{position:absolute;top:0;right:0;z-index:21;display:flex;height:34px}
.window-controls button{width:46px;height:34px;border:0;padding:0;background:transparent;color:var(--ui-text-secondary);display:grid;place-items:center;cursor:default}
.window-controls button:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.window-controls button.is-close:hover{background:#E81123;color:#fff}
.is-collapsed .sidebar-search,.is-collapsed .brand-copy,.is-collapsed .nav-label,.is-collapsed .nav-hint,.is-collapsed .nav-chevron,.is-collapsed .nav-caption,.is-collapsed .space-row,.is-collapsed .nav-session-row,.is-collapsed .nav-session,.is-collapsed .nav-empty,.is-collapsed .notices{display:none}
.is-collapsed .control-brand{justify-content:center;padding:2px 0 16px}
.is-collapsed .nav-sub{padding-left:0}
.is-collapsed .nav-item{justify-content:center;padding:0;height:36px}
.is-collapsed .nav-group .nav-attention{position:absolute;top:6px;right:8px;margin:0}
.is-collapsed .account-chip{justify-content:center;padding:0}
.is-collapsed .sidebar-scroll{margin:0;padding:0}
</style>


