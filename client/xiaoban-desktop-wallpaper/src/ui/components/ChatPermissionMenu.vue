<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { Check, ChevronDown, Settings2, ShieldAlert, ShieldCheck } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';

/** 默认权限 / 完全信任 under the message box. The mode is the chat store's `permissionMode`:
 *  `ask` asks before writing files, running commands and changing settings (an approval can be remembered);
 *  `auto` runs them without asking. On systems with a sandbox (macOS, or Windows when turned on in settings)
 *  those commands then run inside it, limited to the working folder; dangerous operations are still asked. */
const props = defineProps<{ store: ControlStore; disabled?: boolean }>();

const root = ref<HTMLElement | null>(null);
const open = ref(false);
const mode = computed(() => props.store.chat.permissionMode);
const full = computed(() => mode.value === 'auto');
const options = [
  { id: 'ask', label: '默认权限', desc: '读写文件、运行命令前先问你，可以记住授权' },
  { id: 'auto', label: '完全信任', desc: '不再询问，直接执行。支持沙箱的系统上命令只在工作区内运行；高风险操作仍会先问你' },
] as const;

function close() { open.value = false; }
function onOutside(e: PointerEvent) { if (root.value && !root.value.contains(e.target as Node)) close(); }
function onKey(e: KeyboardEvent) { if (e.key === 'Escape') close(); }
watch(open, value => {
  if (value) { document.addEventListener('pointerdown', onOutside); document.addEventListener('keydown', onKey); }
  else { document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); }
});
watch(() => props.disabled, disabled => { if (disabled) close(); });
onBeforeUnmount(() => { document.removeEventListener('pointerdown', onOutside); document.removeEventListener('keydown', onKey); });

function pick(next: 'ask' | 'auto') {
  close();
  if (next !== mode.value) void props.store.chat.setPermissionMode(next);
}
function settings() { close(); props.store.openSettings('safety'); }
</script>

<template>
  <div ref="root" class="perm-anchor">
    <button type="button" class="perm-trigger" :class="{ 'is-full': full }" data-testid="chat-permission-toggle" aria-haspopup="menu" :aria-expanded="open" :title="full ? '完全信任：不再询问直接执行' : '默认权限：执行前先问你'" :disabled="disabled" @click="open = !open">
      <ShieldAlert v-if="full" :size="14" /><ShieldCheck v-else :size="14" /><span>{{ full ? '完全信任' : '默认权限' }}</span><ChevronDown :size="11" :stroke-width="2.4" />
    </button>
    <div v-show="open" class="xb-menu perm-menu" role="menu" aria-label="权限">
      <button v-for="option in options" :key="option.id" type="button" class="perm-option" role="menuitemradio" :class="{ active: mode === option.id, warn: option.id === 'auto' }" :aria-checked="mode === option.id" :data-testid="`perm-${option.id}`" @click="pick(option.id)">
        <span class="perm-tile"><ShieldAlert v-if="option.id === 'auto'" :size="15" /><ShieldCheck v-else :size="15" /></span>
        <span class="perm-text"><b>{{ option.label }}</b><small>{{ option.desc }}</small></span>
        <span class="perm-check"><Check v-if="mode === option.id" :size="13" :stroke-width="3" /></span>
      </button>
      <div class="xb-menu-sep" />
      <button type="button" class="xb-menu-item perm-settings" role="menuitem" data-testid="perm-settings" @click="settings"><Settings2 :size="14" />权限与沙箱设置…</button>
    </div>
  </div>
</template>

<style scoped>
.perm-anchor{position:relative;display:flex;align-items:center;flex:none}
.perm-trigger{display:inline-flex;align-items:center;gap:5px;height:28px;padding:0 9px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-secondary);font:inherit;font-size:12.5px;cursor:pointer;white-space:nowrap}
.perm-trigger:hover,.perm-trigger[aria-expanded="true"]{background:var(--ui-control);color:var(--ui-text-primary)}
.perm-trigger.is-full,.perm-trigger.is-full:hover{color:var(--ui-danger)}
.perm-trigger:disabled{opacity:.5;cursor:not-allowed}
.perm-menu{position:absolute;left:0;bottom:calc(100% + 8px);width:300px;max-width:calc(100vw - 32px);box-sizing:border-box}
.perm-option{width:100%;display:flex;align-items:flex-start;gap:10px;padding:9px 10px;border:0;border-radius:8px;background:transparent;color:var(--ui-text-primary);font:inherit;text-align:left;cursor:pointer}
.perm-option:hover{background:var(--ui-surface-hover)}
.perm-option.active{background:var(--ui-soft)}
.perm-option.warn.active{background:color-mix(in srgb,var(--ui-danger) 9%,transparent)}
.perm-tile{width:28px;height:28px;border-radius:8px;flex-shrink:0;display:grid;place-items:center;background:var(--ui-soft);color:var(--ui-ink)}
.perm-option.warn .perm-tile{background:color-mix(in srgb,var(--ui-danger) 12%,transparent);color:var(--ui-danger)}
.perm-text{flex:1;min-width:0;display:flex;flex-direction:column;gap:2px}
.perm-text b{font-size:13.5px;font-weight:500}
.perm-option.warn .perm-text b{color:var(--ui-danger)}
.perm-text small{font-size:11.5px;line-height:1.5;color:var(--ui-text-secondary)}
.perm-check{width:14px;padding-top:3px;color:var(--ui-ink)}
.perm-option.warn .perm-check{color:var(--ui-danger)}
.perm-settings{font-size:12.5px;color:var(--ui-text-secondary);min-height:32px}
</style>
