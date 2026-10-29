import { nextTick } from 'vue';
import { invoke, type Settings } from '../bridge';
import type { CareStatus } from '../care-model';
import type { ControlPage, ControlStore } from './control-types';

// Only called after the native isolated profile explicitly advertises this capability.
// This checks packaged assets and read-only IPC without editing preferences or opening games.
export async function runUiSmoke(store: ControlStore): Promise<void> {
  if (!store.desktop || !store.info.uiSmoke) return;
  const originalPage = store.page;
  const errors: string[] = [];
  const onError = (event: ErrorEvent) => errors.push(event.message || 'Unspecified frontend error');
  const onRejection = (event: PromiseRejectionEvent) => errors.push(String(event.reason));
  window.addEventListener('error', onError); window.addEventListener('unhandledrejection', onRejection);
  const mounted: string[] = [];
  const wait = async (predicate: () => boolean, marker: string) => {
    const deadline = Date.now() + 3500;
    do {
      await nextTick();
      if (errors.length) throw new Error(errors.join('; '));
      if (predicate()) return;
      await new Promise(resolve => setTimeout(resolve, 40));
    } while (Date.now() < deadline);
    throw new Error(`UI smoke timed out: ${marker}`);
  };
  let report: Record<string, unknown>;
  try {
    if (!store.ready || store.initialError) throw new Error(`Native initialization failed: ${store.initialError || 'not ready'}`);
    const native = await invoke<Settings>('get_settings');
    if (typeof native.companionEnabled !== 'boolean' || native.fps !== store.settings.fps || native.size !== store.settings.size) throw new Error('Native settings were not bound to the Vue control store');
    if (!store.info.version || store.info.platform === 'preview') throw new Error('Native application identity missing');
    // The studio kinds are tabs of 装扮 and "settings" is a dialog now; the old ids still route there.
    for (const page of ['wallpapers', 'effects', 'companion', 'games', 'care', 'settings'] as ControlPage[]) {
      if (page === 'settings') store.openSettings('general'); else store.navigate(page);
      await wait(() => !!document.querySelector(`[data-testid="${page}-page"]`), `mount ${page}`);
      mounted.push(page);
      if (page === 'companion') {
        await wait(() => document.querySelector('[data-testid="companion-enabled"]')?.getAttribute('aria-checked') === String(store.settings.companionEnabled), 'companion setting binding');
      }
      if (page === 'games') await wait(() => document.querySelectorAll('[data-testid="games-page"] .game-card').length === 8, 'eight packaged game manifests');
      if (page === 'settings') {
        await wait(() => {
          const switches = document.querySelectorAll('[data-testid="settings-page"] [role="switch"]');
          return switches[0]?.getAttribute('aria-checked') === String(store.settings.autostart) && switches[1]?.getAttribute('aria-checked') === String(store.settings.paused);
        }, 'startup and animation setting bindings');
      }
    }
    const care = await invoke<CareStatus>('get_care_settings');
    if (!Array.isArray(care.rules) || !care.config || care.error) throw new Error(`Native care read failed: ${care.error || 'invalid response'}`);
    const nativeContextMenuReady = await (await import('./wallpaper/context-menu')).probeWallpaperNativeMenu();
    if (!nativeContextMenuReady) throw new Error('Native material context-menu API unavailable');
    report = { ok: true, marker: 'vue-native-control-readonly', pages: mounted, gamePackCount: 8, careRuleCount: care.rules.length, localWallpaperCount: store.library.items.length, configBound: true, nativeContextMenuReady };
  } catch (error) {
    report = { ok: false, marker: 'vue-native-control-readonly', pages: mounted, error: error instanceof Error ? error.message : String(error) };
  } finally {
    store.closeSettings(); store.navigate(originalPage); await nextTick();
    window.removeEventListener('error', onError); window.removeEventListener('unhandledrejection', onRejection);
  }
  await invoke('ui_smoke_report', { report });
}
