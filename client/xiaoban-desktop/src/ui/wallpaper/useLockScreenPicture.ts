import { onBeforeUnmount, onMounted, reactive } from 'vue';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';

type Result = { status: 'applied' } | { status: 'manual'; id: string };

export function useLockScreenPicture(store: ControlStore) {
  const picture = reactive({ supported: false, busy: false, frame: '', manualId: '' });
  let revision = 0;
  onMounted(async () => {
    if (!store.desktop) return;
    try { picture.supported = await invoke<boolean>('lock_screen_picture_supported') === true; }
    catch { /* An older or preview-only host does not offer this action. */ }
  });

  function close() {
    revision++;
    picture.frame = '';
    picture.manualId = '';
  }
  onBeforeUnmount(close);
  async function show(render: () => Promise<string>) {
    if (!picture.supported || picture.busy) return;
    close();
    const request = revision;
    picture.busy = true;
    try {
      const frame = await render();
      if (request === revision) picture.frame = frame;
    }
    catch (error) { store.notify(error, 'error'); }
    finally { picture.busy = false; }
  }
  async function apply() {
    if (!picture.frame || picture.busy) return;
    const request = revision;
    picture.busy = true;
    try {
      const result = await invoke<Result>('lock_screen_apply_picture', { jpeg: picture.frame });
      if (result.status === 'manual' && request === revision) picture.manualId = result.id;
      if (result.status === 'applied') {
        picture.frame = '';
        store.notify('已设为锁屏图片', 'success');
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
