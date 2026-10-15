import { reactive, ref, toRaw } from 'vue';
import { invoke, listen } from '../../bridge';
import { careRuleError, newCareRule, type CareConfig, type CareKind, type CareRule, type CareStatus } from '../../care-model';

export const clone = <T>(value: T): T => JSON.parse(JSON.stringify(toRaw(value)));
export function emptyCareStatus(): CareStatus {
  return { config: { enabled: true, sound: false, voice: false, quietEnabled: false, quietStart: '22:00', quietEnd: '08:00', weatherSource: { provider: 'open-meteo', forecastUrl: '', geocodingUrl: '' } }, rules: [], pendingCount: 0, weather: null, weatherError: null, error: null, systemNotificationsAvailable: false, activityAvailable: false };
}
export function createCareState(native: boolean, notify: (message: unknown, type?: 'success' | 'error' | 'info') => void) {
  const status = ref(emptyCareStatus());
  const preferences = reactive(clone(status.value.config));
  const preferencesDirty = ref(false);
  const draft = ref(newCareRule());
  const editing = ref(false);
  const busy = ref(false);
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
  function open(kind: CareKind = 'custom', rule?: CareRule) { draft.value = clone(rule ?? newCareRule(kind)); editing.value = true; }
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
  async function savePreferences() {
    const config: CareConfig = clone(preferences);
    if (!/^([01]\d|2[0-3]):[0-5]\d$/.test(config.quietStart) || !/^([01]\d|2[0-3]):[0-5]\d$/.test(config.quietEnd)) { notify('请选择有效的免打扰时间', 'error'); return; }
    if (config.weatherSource.provider === 'custom' && [config.weatherSource.forecastUrl, config.weatherSource.geocodingUrl].some(url => { try { return new URL(url).protocol !== 'https:'; } catch { return true; } })) { notify('天气与城市搜索来源都需要有效的 HTTPS 地址', 'error'); return; }
    await run(async () => {
      if (native) receive(await invoke<CareStatus>('configure_care', { config })); else status.value.config = config;
      preferencesDirty.value = false; Object.assign(preferences, clone(status.value.config));
      notify(native ? '提醒偏好已保存' : '仅更新本次预览偏好', 'success');
    });
  }
  async function remove(rule: CareRule) { await run(async () => { if (native) receive(await invoke<CareStatus>('delete_care_rule', { id: rule.id })); else status.value.rules = status.value.rules.filter(item => item.id !== rule.id); notify('提醒已删除，待展示任务已取消', 'success'); }); }
  async function toggle(rule: CareRule, enabled: boolean) { await run(() => upsert({ ...clone(rule), enabled })); }
  async function preview(rule: CareRule) {
    const error = validation(rule); if (error) { notify(error, 'error'); return; }
    if (!native) { notify(`文案预览：${rule.text || '天气提醒需桌面程序查询真实天气'}。浏览器不会显示桌面气泡。`, 'info'); return; }
    await run(async () => { await invoke('preview_care_rule', { rule: clone(rule) }); notify('已显示气泡预览，不生成到期任务', 'success'); });
  }
  return { status, preferences, preferencesDirty, draft, editing, busy, receive, initialize, open, save, savePreferences, remove, toggle, preview, run, dispose() { alive = false; unlisten?.(); } };
}
