import { computed, onScopeDispose, ref, watch, type Ref } from 'vue';
import { invoke } from '../../bridge.ts';
import { isImeEnter } from './format.ts';

export type Mention = { kind: 'connector' | 'expert' | 'skill'; id: string };
export interface MentionItem extends Mention { name: string; description: string; iconUrl?: string | null }
export function mentionToken(draft: string) {
  const match = /(^|\s)@([^\s@]*)$/.exec(draft);
  return match ? { start: match.index + match[1].length, query: match[2] } : null;
}
export function useComposerMentions(draft: Ref<string>, enabled: () => boolean, choose: (item: MentionItem) => Promise<boolean>, notify: (error: unknown) => void) {
  const inventory = ref<MentionItem[]>([]), index = ref(0), dismissed = ref(''), loading = ref(false);
  const selected = ref<MentionItem[]>([]);
  const selecting = ref(false);
  const token = computed(() => mentionToken(draft.value));
  let generation = 0;
  async function load() {
    const current = ++generation;
    loading.value = true; inventory.value = [];
    try { const result = await invoke<MentionItem[]>('ai_mentions'); if (current === generation) inventory.value = result; }
    catch (e) { if (current === generation) { inventory.value = []; notify(e); } }
    finally { if (current === generation) loading.value = false; }
  }
  watch(token, (now, before) => { index.value = 0; if (now && !before && enabled()) void load(); });
  onScopeDispose(() => { generation++; });
  const items = computed(() => {
    if (!token.value || !enabled()) return [];
    const query = token.value.query.toLowerCase();
    return inventory.value.filter(i => `${i.name} ${i.id} ${i.description}`.toLowerCase().includes(query)).slice(0, 12);
  });
  const open = computed(() => !!token.value && enabled() && dismissed.value !== draft.value);
  async function pick(item: MentionItem) {
    const at = token.value, original = draft.value;
    if (!at || selecting.value) return;
    selecting.value = true;
    const cleared = original.slice(0, at.start);
    draft.value = cleared;
    try {
      if (!(await choose(item))) { if (draft.value === cleared) draft.value = original; return; }
      if (item.kind === 'connector' && !selected.value.some(i => i.id === item.id)) selected.value.push(item);
    } catch (e) { if (draft.value === cleared) draft.value = original; notify(e); }
    finally { selecting.value = false; }
  }
  function keydown(event: KeyboardEvent, composing: boolean) {
    if (!open.value || isImeEnter(event, composing)) return false;
    const count = items.value.length;
    if (event.key === 'Escape') dismissed.value = draft.value;
    else if (count && event.key === 'ArrowDown') index.value = (index.value + 1) % count;
    else if (count && event.key === 'ArrowUp') index.value = (index.value + count - 1) % count;
    else if (count && (event.key === 'Enter' || event.key === 'Tab')) void pick(items.value[index.value]);
    else if (event.key !== 'Enter') return false;
    event.preventDefault(); return true;
  }
  return { items, index, open, selected, loading, selecting, pick, keydown };
}
