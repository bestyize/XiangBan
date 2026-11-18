import { onBeforeUnmount, onMounted, reactive } from 'vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

type Result = { status: 'applied' | 'desktop' } | { status: 'manual'; id: string };
type PictureMode = 'lockScreen' | 'desktop';

export function useLockScreenPicture(store: ControlStore) {
  const picture = reactive({ mode: null as PictureMode | null, busy: false, frame: '', manualId: '' });
  let targetMonitor: string | undefined;
  let revision = 0;
  onMounted(async () => {
    if (!store.desktop) return;
    try {
      const mode = await invoke<PictureMode | null>('lock_screen_picture_mode');
      if (mode === 'lockScreen' || mode === 'desktop') picture.mode = mode;
    }
    catch { /* An older or preview-only host does not offer this action. */ }
  });

  function close() {
    revision++;
    picture.frame = '';
    picture.manualId = '';
    targetMonitor = undefined;
  }
  onBeforeUnmount(close);
  async function show(render: () => Promise<string>, monitor?: string) {
    if (!picture.mode || picture.busy) return;
    close();
    const request = revision;
    picture.busy = true;
    try {
      const frame = await render();
      if (request === revision) {
        picture.frame = frame;
        targetMonitor = monitor;
      }
    }
    catch (error) { store.notify(error, 'error'); }
    finally { picture.busy = false; }
  }
  async function apply() {
    if (!picture.frame || picture.busy) return;
    const request = revision;
    picture.busy = true;
    try {
      const args = targetMonitor === undefined ? { jpeg: picture.frame } : { jpeg: picture.frame, monitor: targetMonitor };
      const result = await invoke<Result>('lock_screen_apply_picture', args);
      if (result.status === 'manual' && request === revision) picture.manualId = result.id;
      if (result.status === 'applied' || result.status === 'desktop') {
        picture.frame = '';
        store.notify(result.status === 'desktop' ? '已设为系统桌面底图' : '已设为锁屏图片', 'success');
      }
    } catch (error) { store.notify(error, 'error'); }
    finally { picture.busy = false; }
  }
  async function save() {
    if (!picture.manualId || picture.busy) return;
    picture.busy = true;
    try {
      if (await invoke<boolean>('lock_screen_save_picture', { id: picture.manualId })) {
        store.notify('图片已保存，请在系统锁屏设置中选择它', 'info');
        picture.frame = '';
        picture.manualId = '';
      }
    } catch (error) { store.notify(error, 'error'); }
    finally { picture.busy = false; }
  }
  return { picture, show, apply, save, close };
}
