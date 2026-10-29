import { invoke, listen } from '../../bridge.ts';
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
/** The event is only a wake-up. Command and arguments must be taken from the native queue,
 * never from an event payload that another webview could forge. */
export function serveConfigurationWindows() {
  return listen<{ id: string }>('configuration-command', async ({ id }) => {
    let request: { id: string; command: string; args: Record<string, unknown> };
    try { request = await invoke('configuration_window_take', { id }); } catch { return; }
    let ok = true, value: unknown;
    try { value = await invoke(request.command, request.args); if (value instanceof ArrayBuffer) value = Array.from(new Uint8Array(value)); }
    catch (error) { ok = false; value = error instanceof Error ? error.message : error; }
    await invoke('configuration_window_reply', { id: request.id, ok, value: value ?? null }).catch(() => {});
  });
}
