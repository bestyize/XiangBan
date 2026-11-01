import { emit, invoke, listen } from '../../bridge.ts';
import type { SettingsTab } from '../control-types.ts';
export const configurationScene = () => new URLSearchParams(location.search).get('scene') === 'configuration';
export const configurationKind = () => new URLSearchParams(location.search).get('kind');
export function openSettingsWindow(tab?: SettingsTab) {
  return invoke<void>('configuration_window_open', { target: 'settings', tab: tab ?? null, provider: null });
}
export function openModelWindow(provider: string) {
  return invoke<void>('configuration_window_open', { target: 'model', tab: null, provider });
}
export function closeConfigurationWindow() { return invoke<void>('configuration_window_close'); }
export function dragConfigurationWindow(event: MouseEvent) {
  if (event.button || event.detail > 1 || (event.target as HTMLElement).closest('button,input,select,a')) return;
  event.preventDefault();
  void invoke('configuration_window_action', { action: 'drag' }).catch(() => {});
}
export function zoomConfigurationWindow(event: MouseEvent) {
  if ((event.target as HTMLElement).closest('button,input,select,a')) return;
  void invoke('configuration_window_action', { action: 'double-click' }).catch(() => {});
}
/** The event is only a wake-up. Command and arguments must be taken from the native queue,
 * never from an event payload that another webview could forge. */
export function serveConfigurationWindows(openStoreTask?: (args: Record<string, unknown>) => Promise<void>, openCloudTask?: (args: Record<string, unknown>) => Promise<void>) {
  return listen<{ id: string }>('configuration-command', async ({ id }) => {
    let request: { id: string; command: string; args: Record<string, unknown> };
    try { request = await invoke('configuration_window_take', { id }); } catch { return; }
    let ok = true, value: unknown;
    try {
      if (request.command === 'ai_store_open_task') {
        if (!openStoreTask) throw new Error('插件任务入口尚未就绪');
        await openStoreTask(request.args);
      } else if (request.command === 'library_cloud_open_task') {
        if (!openCloudTask) throw new Error('云文档任务入口尚未就绪');
        await openCloudTask(request.args);
      } else value = await invoke(request.command, request.args);
      if (value instanceof ArrayBuffer) value = Array.from(new Uint8Array(value));
      const mutation = /^ai_(market_(install|update|group_install)|store_(install|uninstall|expert_install|expert_remove|connector_(connect|disconnect|reconnect))|connector_(configure|unbind|set_enabled))$/.test(request.command);
      const authFinished = ['ai_store_connector_auth_status', 'ai_connector_baidu_status'].includes(request.command)
        && !!value && typeof value === 'object' && 'status' in value && ['connected', 'cancelled', 'error'].includes(String(value.status));
      if (mutation || authFinished) await emit('ai-store-changed').catch(() => {});
    }
    catch (error) { ok = false; value = error instanceof Error ? error.message : error; }
    await invoke('configuration_window_reply', { id: request.id, ok, value: value ?? null }).catch(() => {});
  });
}
