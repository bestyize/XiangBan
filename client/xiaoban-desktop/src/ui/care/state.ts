import { reactive, ref, toRaw } from 'vue';
import { invoke, listen } from '../../bridge';
import { careRuleError, newCareRule, type CareConfig, type CareKind, type CareRule, type CareStatus } from '../../care-model';
import { formatLocalDateTime } from './schedule';

export const clone = <T>(value: T): T => JSON.parse(JSON.stringify(toRaw(value)));
export function emptyCareStatus(): CareStatus {
  return { config: { enabled: true, sound: false, onceSound: true, voice: false, onceVoice: true, quietEnabled: false, quietStart: '22:00', quietEnd: '08:00', weatherSource: { provider: 'open-meteo', forecastUrl: '', geocodingUrl: '' } }, rules: [], pendingCount: 0, weather: null, weatherError: null, error: null, systemNotificationsAvailable: false, activityAvailable: false };
}
export function createCareState(native: boolean, notify: (message: unknown, type?: 'success' | 'error' | 'info') => void) {
  const status = ref(emptyCareStatus());
  const preferences = reactive(clone(status.value.config));
  const preferencesDirty = ref(false);
  const draft = ref(newCareRule());
  const editing = ref(false);
  const busy = ref(false);
  const runningAgentIds = ref<string[]>([]);
  let alive = true;
  let unlisten: (() => void) | undefined;
  function receive(next: CareStatus) {
    if (!alive) return;
    status.value = next;
    if (!preferencesDirty.value) Object.assign(preferences, clone(next.config));
  }
  async function run(action: () => Promise<void>) {
    if (busy.value) return;
    busy.value = true;
    try { await action(); } catch (error) { if (alive) notify(error instanceof Error ? error.message : String(error), 'error'); }
    finally { busy.value = false; }
  }
  async function initialize() {
    if (!native) return;
    await run(async () => {
      receive(await invoke<CareStatus>('get_care_settings'));
      const stop = await listen<CareStatus>('care-settings-changed', receive);
      if (alive) unlisten = stop; else stop();
    });
  }
  function open(kind: CareKind = 'custom', rule?: CareRule) {
    if (rule) {
      draft.value = clone(rule);
    } else {
      const fresh = newCareRule(kind);
      fresh.sound = status.value.config.sound;
      fresh.voice = status.value.config.voice;
      fresh.sticky = false;
      draft.value = fresh;
    }
    editing.value = true;
  }
  function openPreset(preset: Partial<CareRule> & { kind?: CareKind }) {
    const base = newCareRule(preset.kind ?? 'custom');
    const merged: CareRule = {
      ...base,
      ...clone(preset),
      id: '',
      revision: 0,
      notBefore: 0,
    };
    if (merged.repeat === 'once' && !merged.onceAt) {
      merged.onceAt = formatLocalDateTime(new Date(Date.now() + 60 * 60_000));
    }
    draft.value = merged;
    editing.value = true;
  }
  function validation(rule: CareRule) {
    if (rule.kind === 'weather' && status.value.config.weatherSource.provider === 'disabled') return '请先启用并保存天气来源，再添加天气提醒';
    return careRuleError(rule);
  }
  async function upsert(rule: CareRule) {
    if (native) receive(await invoke<CareStatus>('upsert_care_rule', { rule: clone(rule) }));
    else {
      const next = clone(rule); next.id ||= `preview-${Date.now()}-${Math.random().toString(36).slice(2)}`;
      status.value.rules = [...status.value.rules.filter(item => item.id !== next.id), next];
    }
  }
  async function save() {
    const rule = clone(draft.value); rule.text = rule.text.trim();
    const error = validation(rule); if (error) { notify(error, 'error'); return; }
    await run(async () => { await upsert(rule); editing.value = false; notify(native ? '提醒已保存' : '本次预览已添加；不保存、不在后台触发', 'success'); });
  }
  async function quickAdd(rule: CareRule) {
    const next = clone(rule); next.text = next.text.trim();
    const error = validation(next); if (error) { notify(error, 'error'); return false; }
    let ok = false;
    await run(async () => { await upsert(next); ok = true; notify(native ? '提醒已添加' : '本次预览已添加；不保存、不在后台触发', 'success'); });
    return ok;
  }
  async function savePreferences(silent = false) {
    const config: CareConfig = clone(preferences);
    if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(config.quietStart) || !/^([01]\d|2[0-3]):[0-5]\d$/.test(config.quietEnd)) { notify('请选择有效的免打扰时间', 'error'); return false; }
    if (config.weatherSource.provider === 'custom' && [config.weatherSource.forecastUrl, config.weatherSource.geocodingUrl].some(url => { try { return new URL(url).protocol !== 'https:'; } catch { return true; } })) { notify('天气与城市搜索来源都需要有效的 HTTPS 地址', 'error'); return false; }
    let saved = false;
    await run(async () => {
      if (native) receive(await invoke<CareStatus>('configure_care', { config })); else status.value.config = config;
      preferencesDirty.value = false; Object.assign(preferences, clone(status.value.config));
      saved = true;
      if (!silent) notify(native ? '提醒偏好已保存' : '仅更新本次预览偏好', 'success');
    });
    return saved;
  }
  async function remove(rule: CareRule) { await run(async () => { if (native) receive(await invoke<CareStatus>('delete_care_rule', { id: rule.id })); else status.value.rules = status.value.rules.filter(item => item.id !== rule.id); notify('提醒已删除，待展示任务已取消', 'success'); }); }
  async function removeMany(rulesToDelete: CareRule[]) {
    if (!rulesToDelete.length) return;
    await run(async () => {
      for (const rule of rulesToDelete) {
        if (native) receive(await invoke<CareStatus>('delete_care_rule', { id: rule.id }));
        else status.value.rules = status.value.rules.filter(item => item.id !== rule.id);
      }
      notify(`已清空 ${rulesToDelete.length} 条已完成提醒`, 'success');
    });
  }
  async function toggle(rule: CareRule, enabled: boolean) { await run(() => upsert({ ...clone(rule), enabled })); }
  async function patchRule(rule: CareRule, patch: Partial<CareRule>, message?: string) {
    await run(async () => {
      await upsert({ ...clone(rule), ...patch });
      if (message) notify(message, 'success');
    });
  }
  async function duplicateRule(rule: CareRule) {
    const copy = clone(rule);
    copy.id = '';
    copy.revision = 0;
    copy.notBefore = 0;
    if (copy.text) copy.text = `${copy.text}（副本）`.slice(0, copy.kind === 'agent' ? 1000 : 240);
    await run(async () => {
      await upsert(copy);
      notify('已创建规则副本', 'success');
    });
  }
  async function runAgentNow(rule: CareRule) {
    if (rule.kind !== 'agent') return;
    if (!native) {
      notify('立即试跑需要桌面程序；浏览器预览不调用 AI 模型', 'info');
      return;
    }
    if (runningAgentIds.value.includes(rule.id)) return;
    runningAgentIds.value = [...runningAgentIds.value, rule.id];
    try {
      receive(await invoke<CareStatus>('run_care_agent_now', { id: rule.id }));
      notify('已触发后台试跑，完成后将弹出摘要并写入聊天', 'success');
    } catch (error) {
      if (alive) notify(error instanceof Error ? error.message : String(error), 'error');
    } finally {
      setTimeout(() => {
        if (alive) runningAgentIds.value = runningAgentIds.value.filter(id => id !== rule.id);
      }, 2500);
    }
  }
  async function preview(rule: CareRule) {
    const error = validation(rule); if (error) { notify(error, 'error'); return; }
    if (!native) { notify(`文案预览：${rule.text || '天气提醒需桌面程序查询真实天气'}。浏览器不会显示桌面气泡。`, 'info'); return; }
    await run(async () => { await invoke('preview_care_rule', { rule: clone(rule) }); notify('已显示气泡预览，不生成到期任务', 'success'); });
  }
  return {
    status, preferences, preferencesDirty, draft, editing, busy, runningAgentIds,
    receive, initialize, open, openPreset, save, quickAdd, savePreferences, remove, removeMany,
    toggle, patchRule, duplicateRule, runAgentNow, preview, run,
    dispose() { alive = false; unlisten?.(); },
  };
}
