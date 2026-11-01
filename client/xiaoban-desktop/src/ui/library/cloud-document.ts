import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
export interface CloudContentState { coverage: 'full' | 'partial' | 'empty'; note: string; preview: string; bytes: number; updatedMs: number }
export interface CloudSnapshot { text: string; state: CloudContentState }
export function openCloudDocument(id: string) { return invoke<void>('configuration_window_open', { target: 'cloud', provider: id, tab: null }); }
/** Resolve content in the trusted host; child windows cannot inject a task body. */
export async function acceptCloudTask(store: ControlStore, args: Record<string, unknown>) {
  if (typeof args.id !== 'string' || !['ask', 'summary'].includes(String(args.mode))) throw new Error('云文档提问参数无效');
  const task = await invoke<{ draft: string; attachments: NonNullable<ControlStore['creationAttachments']> }>('library_cloud_question', { id: args.id, mode: args.mode });
  await store.chat.select(null);
  store.creationAttachments = task.attachments;
  store.creationDraft = task.draft;
  store.navigate('new-task');
}
