<script setup lang="ts">
/**
 * 输入框右上角的连接器：已选的连接器画成重叠的小图标（最多 3 个，其余折成 +N），点开逐个开关。
 * 开关只决定这个对话里「优先使用」哪些连接器；连接和授权仍在 AI 插件里完成，没连上的这里只给「去连接」。
 */
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Plug } from 'lucide-vue-next';
import { loadConnectorInventory, type ConnectorInventoryEntry } from '../chat/connector-inventory.ts';
import { connectorIcon } from '../ai-store/connector-icons.ts';
import { hueStyle } from '../ai-store/hues.ts';

interface Picked { id: string; name: string; iconUrl?: string | null }

const props = defineProps<{ selected: Picked[]; desktop: boolean; disabled?: boolean; down?: boolean }>();
const emit = defineEmits<{ toggle: [item: { id: string; name: string }]; manage: [] }>();

const root = ref<HTMLElement | null>(null);
const open = ref(false);
const loading = ref(false);
const failed = ref(false);
const warning = ref('');
const status = ref<ConnectorInventoryEntry[]>([]);
let generation = 0;

const VISIBLE = 3;
const EMPTY_ICON = 'data:image/svg+xml,';
function iconOf(item: Picked) {
  const own = connectorIcon(item.id);
  if (own.length > EMPTY_ICON.length) return own;
  return item.iconUrl?.startsWith('data:image/') ? item.iconUrl : null;
}
const stack = computed(() => props.selected.slice(0, VISIBLE).map(item => ({ ...item, src: iconOf(item), style: hueStyle(undefined, item.id) })));
const extra = computed(() => Math.max(0, props.selected.length - VISIBLE));
const label = computed(() => (props.selected.length ? `已选 ${props.selected.length} 个连接器，点击管理` : '选择连接器'));

const picked = computed(() => new Set(props.selected.map(item => item.id)));
const rows = computed(() => {
  const known = new Set(status.value.map(s => s.id));
  const listed = status.value.map(s => ({ id: s.id, name: s.name, ready: s.connected && s.enabled, detail: s.detail || (s.connected && s.enabled ? `已连接 · ${s.toolCount} 个工具` : s.enabled ? '尚未连接' : '已停用') }));
  // A connector chosen earlier and since removed stays listed, so it can still be dropped from the choice.
  const gone = props.selected.filter(item => !known.has(item.id)).map(item => ({ id: item.id, name: item.name, ready: false, detail: '已不可用' }));
  return [...listed, ...gone].map(row => ({ ...row, on: picked.value.has(row.id), src: iconOf(row), style: hueStyle(undefined, row.id) }));
});

async function load() {
  const current = ++generation;
  loading.value = true; failed.value = false; warning.value = '';
  try {
    const result = props.desktop ? await loadConnectorInventory() : { items: [], warning: '' };
    if (current === generation) { status.value = result.items; warning.value = result.warning; }
  } catch { if (current === generation) { status.value = []; failed.value = true; } }
  finally { if (current === generation) loading.value = false; }
}
function close() { open.value = false; generation++; }
function toggleOpen() { if (props.disabled) return; if (open.value) close(); else { open.value = true; void load(); } }
function onOutside(event: PointerEvent) { if (root.value && !root.value.contains(event.target as Node)) close(); }
function onKey(event: KeyboardEvent) { if (event.key === 'Escape') close(); }
watch(open, now => {
  if (now) { document.addEventListener('pointerdown', onOutside); document.addEventListener('keydown', onKey); }
  else { document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); }
});
onBeforeUnmount(() => { generation++; document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); });
function manage() { close(); emit('manage'); }
</script>

<template>
  <div v-if="selected.length || open" ref="root" class="connector-cluster" data-testid="chat-connectors">
    <button type="button" class="cluster-btn" data-testid="chat-connectors-toggle" :aria-label="label" :title="label" aria-haspopup="dialog" :aria-expanded="open" :disabled="disabled" @click="toggleOpen">
      <Plug v-if="!selected.length" :size="15" aria-hidden="true" />
      <span v-else class="tiles">
        <span v-for="item in stack" :key="item.id" class="tile" :style="item.style" :title="item.name"><img v-if="item.src" :src="item.src" alt="" /><template v-else>{{ item.name.slice(0, 1) }}</template></span>
        <span v-if="extra" class="tile more">+{{ extra }}</span>
      </span>
    </button>
    <div v-if="open" class="cluster-pop" :class="{ down }" role="dialog" aria-label="连接器" data-testid="chat-connectors-panel">
      <div class="pop-head"><b>连接器</b><span>已选 {{ selected.length }} 个</span></div>
      <div class="pop-list">
        <div v-if="failed" class="pop-empty" role="status">读取连接器失败，请稍后重试 <button type="button" class="row-link" @click="load">重试</button></div>
        <div v-else-if="loading && !rows.length" class="pop-empty" role="status">正在读取…</div>
        <div v-else-if="!rows.length" class="pop-empty">还没有连接器，到 AI 插件里添加</div>
        <div v-if="warning" class="pop-empty" role="status">{{ warning }} <button type="button" class="row-link" @click="load">重试</button></div>
        <div v-for="row in rows" :key="row.id" class="pop-row" :data-testid="`chat-connector-${row.id}`">
          <span class="tile big" :style="row.style" aria-hidden="true"><img v-if="row.src" :src="row.src" alt="" /><template v-else>{{ row.name.slice(0, 1) }}</template></span>
          <span class="row-text"><strong>{{ row.name }}</strong><small>{{ row.detail }}</small></span>
          <button v-if="row.ready || row.on" type="button" role="switch" class="switch" :class="{ on: row.on }" :aria-checked="row.on" :aria-label="`优先使用 ${row.name}`" @click="emit('toggle', { id: row.id, name: row.name })"><i /></button>
          <button v-else type="button" class="row-link" @click="manage">去连接</button>
        </div>
      </div>
      <div class="pop-foot"><button type="button" class="foot-link" data-testid="chat-connectors-manage" @click="manage">管理连接器…</button><span>仅本次对话</span></div>
    </div>
  </div>
</template>

<style scoped>
.connector-cluster{position:relative;display:flex;flex:none}
.cluster-btn{display:flex;align-items:center;height:28px;padding:0 8px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);cursor:pointer}
.cluster-btn:hover,.cluster-btn[aria-expanded=true]{background:var(--ui-control)}
.cluster-btn:disabled{opacity:.5;cursor:not-allowed}
.cluster-btn:focus-visible,.switch:focus-visible,.row-link:focus-visible,.foot-link:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
.tiles{display:flex;align-items:center}
.tile{--h:#8E8E93;flex:none;display:grid;place-items:center;width:20px;height:20px;margin-left:-6px;border-radius:50%;overflow:hidden;background:color-mix(in srgb,var(--h) 16%,var(--ui-surface));color:color-mix(in srgb,var(--h) 65%,#000);font-size:10px;font-weight:600;box-shadow:0 0 0 2px var(--ui-surface)}
.tile:first-child{margin-left:0}
.tile img{width:100%;height:100%;object-fit:contain;display:block}
.tile.more{background:var(--ui-control);color:var(--ui-text-secondary)}
.tile.big{width:28px;height:28px;margin-left:0;font-size:12px;box-shadow:none}
.cluster-pop{position:absolute;right:0;bottom:calc(100% + 6px);z-index:30;width:300px;box-sizing:border-box;border-radius:14px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),0 14px 36px rgba(0,0,0,.18);overflow:hidden}
.cluster-pop.down{bottom:auto;top:calc(100% + 6px)}
html[data-theme=dark] .cluster-pop{background:var(--ui-surface-alt)}
.pop-head{display:flex;align-items:baseline;gap:8px;padding:12px 14px 6px}
.pop-head b{font-size:13.5px;font-weight:600;color:var(--ui-text-primary)}
.pop-head span{font-size:12px;color:var(--ui-text-secondary)}
.pop-list{max-height:280px;overflow-y:auto;padding:0 5px;scrollbar-width:thin}
.pop-empty{padding:16px;text-align:center;font-size:12.5px;color:var(--ui-text-secondary)}
.pop-row{display:flex;align-items:center;gap:10px;padding:7px 10px;border-radius:8px}
.pop-row:hover{background:var(--ui-surface-hover)}
.row-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:1px}
.row-text strong{font-size:13.5px;font-weight:500;color:var(--ui-text-primary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.row-text small{font-size:11.5px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.switch{position:relative;flex:none;width:34px;height:20px;padding:0;border:0;border-radius:10px;background:var(--ui-text-tertiary);cursor:pointer;transition:background .15s}
.switch i{position:absolute;top:2px;left:2px;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.25);transition:left .15s}
.switch.on{background:var(--ui-btn)}
.switch.on i{left:16px}
.row-link{flex:none;height:24px;padding:0 10px;border:0;border-radius:7px;background:transparent;box-shadow:inset 0 0 0 1px var(--ui-border-strong);color:var(--ui-ink);font:inherit;font-size:12px;font-weight:500;cursor:pointer}
.pop-foot{display:flex;align-items:center;gap:4px;margin-top:5px;padding:6px 10px;border-top:1px solid var(--ui-line)}
.pop-foot span{margin-left:auto;font-size:11px;color:var(--ui-text-secondary)}
.foot-link{padding:5px 6px;border:0;border-radius:6px;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:12.5px;font-weight:500;cursor:pointer}
.foot-link:hover{background:var(--ui-control)}
@media(prefers-reduced-motion:reduce){.switch,.switch i{transition:none}}
</style>
