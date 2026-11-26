<script setup lang="ts">
import { computed, ref } from 'vue';
import { useModal } from '../composables/useModal';
import Icon, { type IconName } from './Icon.vue';

export interface PalettePage {
  key: string;
  label: string;
  desc: string;
  icon: IconName;
}

const props = defineProps<{ pages: readonly PalettePage[] }>();
const emit = defineEmits<{
  close: [];
  go: [key: string];
  openBinding: [id: number];
  searchUser: [query: string];
  lookupCode: [code: string];
}>();

const root = ref<HTMLElement | null>(null);
const query = ref('');
useModal(root, () => emit('close'));

const trimmed = computed(() => query.value.trim());
const bindingId = computed(() => {
  const m = trimmed.value.match(/^#?(\d+)$/);
  return m ? Number(m[1]) : null;
});
const looksLikeCode = computed(() => {
  const clean = trimmed.value.replace(/[\s-]/g, '');
  return clean.length >= 12 && /^[0-9a-z]+$/i.test(clean) && !/^[0-9a-f]{32}$/i.test(clean);
});
const looksLikeUid = computed(() => trimmed.value.length >= 4 && bindingId.value === null && !looksLikeCode.value && /^[0-9a-f-]+$/i.test(trimmed.value));

const matchedPages = computed(() => {
  const q = trimmed.value.toLowerCase();
  return props.pages.filter((p) => !q || p.label.toLowerCase().includes(q) || p.desc.toLowerCase().includes(q) || p.key.includes(q));
});

function submitFirst() {
  if (bindingId.value !== null) return emit('openBinding', bindingId.value);
  if (looksLikeCode.value) return emit('lookupCode', trimmed.value);
  if (looksLikeUid.value) return emit('searchUser', trimmed.value);
  if (matchedPages.value[0]) emit('go', matchedPages.value[0].key);
}
</script>

<template>
  <div class="scrim over" @click="emit('close')"></div>
  <div ref="root" class="cmd-dialog" role="dialog" aria-modal="true" aria-label="全局快速检索">
    <form class="cmd-input-wrap" @submit.prevent="submitFirst">
      <Icon name="search" :size="16" />
      <input
        v-model="query"
        type="text"
        placeholder="输入页面名、绑定编号（如 1042）、UID 前缀或粘贴兑换码…"
        autocomplete="off"
        spellcheck="false"
        data-autofocus
      />
      <button type="button" class="btn sm ghost" @click="emit('close')">ESC</button>
    </form>

    <div class="cmd-results">
      <template v-if="bindingId !== null">
        <button type="button" class="cmd-item" @click="emit('openBinding', bindingId)">
          <Icon name="users" :size="15" />
          <span>打开绑定 <strong>#{{ bindingId }}</strong> 账本</span>
          <span class="sub">回车直达</span>
        </button>
      </template>

      <template v-if="looksLikeCode">
        <button type="button" class="cmd-item" @click="emit('lookupCode', trimmed)">
          <Icon name="ticket" :size="15" />
          <span>查询兑换码 <code class="mono">{{ trimmed.toUpperCase() }}</code></span>
          <span class="sub">兑换码</span>
        </button>
      </template>

      <template v-if="looksLikeUid">
        <button type="button" class="cmd-item" @click="emit('searchUser', trimmed)">
          <Icon name="search" :size="15" />
          <span>搜索账号 <code class="mono">{{ trimmed }}</code></span>
          <span class="sub">用户额度</span>
        </button>
      </template>

      <button
        v-for="p in matchedPages"
        :key="p.key"
        type="button"
        class="cmd-item"
        :title="p.desc"
        @click="emit('go', p.key)"
      >
        <Icon :name="p.icon" :size="15" />
        <span>{{ p.label }}</span>
        <span class="sub">页面</span>
      </button>

      <div v-if="!matchedPages.length && bindingId === null && !looksLikeCode && !looksLikeUid" class="cmd-empty">
        无匹配结果
      </div>
    </div>
  </div>
</template>

<style scoped>
.cmd-dialog {
  position: fixed;
  top: 14%;
  left: 50%;
  transform: translateX(-50%);
  z-index: 62;
  width: min(500px, calc(100vw - 32px));
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-dialog);
  box-shadow: var(--shadow-dialog);
  overflow: hidden;
  animation: popCmd 180ms var(--ease);
}
.cmd-input-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
  border-bottom: 1px solid var(--line);
  color: var(--ink-3);
}
.cmd-input-wrap input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: none;
  outline: none;
  font-size: 14px;
  color: var(--ink-1);
}
.cmd-results {
  max-height: 320px;
  overflow-y: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.cmd-item {
  width: 100%;
  height: 36px;
  padding: 0 12px;
  border-radius: var(--r-inner);
  text-align: left;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--ink-1);
  transition: background 100ms;
}
.cmd-item:hover { background: var(--subtle); }
.cmd-item > svg { color: var(--ink-3); }
.cmd-item .sub { margin-left: auto; font-size: 11.5px; color: var(--ink-3); white-space: nowrap; }
.cmd-empty { padding: 24px 16px; text-align: center; font-size: 12.5px; color: var(--ink-3); }
@keyframes popCmd {
  from { opacity: 0; transform: translate(-50%, -8px) scale(0.97); }
}
</style>
