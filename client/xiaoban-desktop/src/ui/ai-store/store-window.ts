import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import type { Skill } from './catalog.ts';
import type { InstalledConnector } from './connector-plugins.ts';
import { closeConfigurationWindow } from '../configuration/windows.ts';

export interface StoreEntry { slug: string; name: string; description: string; version?: string; namespace?: string; category?: string | null; iconUrl?: string | null; variant?: Skill['variant'] }
export interface StoreSelection { view: 'skill' | 'registry' | 'connector' | 'group' | 'update'; kind: 'skills' | 'experts' | 'connectors'; source: 'market' | 'hub'; item: StoreEntry }
export interface StoreTask { kind: 'skill' | 'pack' | 'example'; id: string; example?: number }
export const storeWindowScene = () => new URLSearchParams(location.search).get('scene') === 'configuration' && new URLSearchParams(location.search).get('kind') === 'store';
export const useNativeStoreWindow = () => !!window.__TAURI__ && !storeWindowScene();
export function fitConnectorWindow(element?: HTMLElement | null) {
  if (!element || !window.__TAURI__ || !storeWindowScene()) return;
  const chrome = document.querySelector('.configuration-chrome')?.getBoundingClientRect().height ?? 36;
  void invoke('configuration_window_action', { action: 'fit-content', height: Math.ceil(element.getBoundingClientRect().height + chrome + 34) }).catch(() => {});
}
export function openStoreWindow(selection: StoreSelection) {
  // Only public catalog identity goes into the URL. Never serialize a connector, config or credential.
  const { slug, name, description, version, namespace, category, iconUrl, variant } = selection.item;
  return invoke<void>('configuration_window_open', { target: 'store', tab: null, provider: JSON.stringify({ ...selection, item: { slug, name, description, version, namespace, category, iconUrl, variant } }) });
}
export async function openStoreTask(task: StoreTask) {
  await invoke('ai_store_open_task', { task });
  await closeConfigurationWindow();
}
/** Executed only after the console consumes an authenticated native request. Resolve installed
 * identity here so a child window cannot inject an expert prompt or arbitrary task text. */
export async function acceptStoreTask(store: ControlStore, args: Record<string, unknown>) {
  const task = args.task as StoreTask | undefined;
  if (!task || typeof task.id !== 'string') throw new Error('插件任务无效');
  let expert: ControlStore['chat']['pendingExpert'] | undefined;
  let text: string | undefined;
  if (task.kind === 'skill') {
    const item = (await invoke<Skill[]>('ai_store_installed')).find(s => s.slug === task.id);
    if (!item) throw new Error('请先安装插件');
    if (item.plugin || item.marketKind === 'expert') expert = { plugin: item.plugin ? item.slug : '', workflow: item.plugin ? undefined : item.slug, name: item.name, profession: item.profession ?? '', description: item.description, agent: item.agent, quickPrompts: item.quickPrompts, tags: item.tags, hasAvatar: item.hasAvatar, kind: item.pluginKind };
    else text = `请读取已安装的技能「${item.slug}」，按其中的流程帮助我完成以下任务：`;
  } else if (task.kind === 'pack') {
    const item = (await invoke<{ slug: string; name: string; description: string; entrySkill: string; ready: boolean }[]>('ai_store_experts_installed')).find(s => s.slug === task.id && s.ready);
    if (!item) throw new Error('专家工作流尚未就绪');
    expert = { plugin: '', workflow: item.entrySkill, name: item.name, profession: '', description: item.description, hasAvatar: false };
  } else if (task.kind === 'example') {
    const item = (await invoke<InstalledConnector[]>('ai_connector_installed')).find(s => s.id === task.id && s.status === 'connected');
    if (!item || !Number.isInteger(task.example) || task.example! < 0 || !item.examples[task.example!]) throw new Error('连接器或示例已不可用');
    text = item.examples[task.example!];
  } else throw new Error('插件任务类型无效');
  await store.chat.select(null);
  if (expert) store.chat.pendingExpert = expert;
  if (text) store.creationDraft = text;
  store.navigate('new-task');
}
