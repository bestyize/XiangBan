import { reactive } from 'vue';
import type { LibraryDoc } from './library-store.ts';
type Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;
export interface DownloadProgress { id: string; received: number; total: number }
/** The native host resolves identity and credentials; views send only a saved reference ID. */
export function createCloudDownloader(invoke: Invoke, saved: (doc: LibraryDoc) => Promise<void> | void) {
  const state = reactive({
    id: '', received: 0, total: 0, cancelling: false, error: '', message: '',
    progress(event: DownloadProgress) { if (event.id === state.id && event.received >= 0 && event.total >= event.received) { state.received = event.received; state.total = event.total; } },
    label() { return state.cancelling ? '取消中…' : state.total ? `下载中 ${Math.min(100, Math.floor(state.received / state.total * 100))}% · ${(state.received / 1048576).toFixed(1)} / ${(state.total / 1048576).toFixed(1)} MB` : '下载中…'; },
    async download(id: string) {
      if (state.id) return;
      state.id = id; state.received = state.total = 0; state.cancelling = false; state.error = state.message = '';
      try { const doc = await invoke<LibraryDoc>('library_cloud_download', { id }); await saved(doc); state.message = '原文件已保存到“我的文档”'; }
      catch (error) { if (String(error).includes('下载已取消')) state.message = '下载已取消'; else state.error = String(error); }
      finally { state.id = ''; state.cancelling = false; }
    },
    async cancel() {
      if (!state.id || state.cancelling) return;
      state.cancelling = true;
      try { await invoke('library_cloud_download_cancel', { id: state.id }); }
      catch (error) { state.error = String(error); state.cancelling = false; }
    },
  });
  return state;
}
