<script setup lang="ts">
import { computed, ref } from 'vue';
import { useModal } from '../composables/useModal';
import { kindLabels, money, xiaobanPoints } from '../format';
import { notify } from '../toast';
import type { Batch } from '../types';
import Icon from './Icon.vue';

const props = defineProps<{ batch: Batch; codes: string[] }>();
const emit = defineEmits<{ close: [] }>();

const root = ref<HTMLElement | null>(null);
useModal(root);

const copied = ref(false);
const downloaded = ref(false);
const list = ref<HTMLElement | null>(null);
const text = computed(() => props.codes.join('\n'));
const safe = computed(() => copied.value || downloaded.value);

async function copy() {
  try {
    await navigator.clipboard.writeText(text.value);
    copied.value = true;
    notify(`已复制 ${props.codes.length} 个兑换码`);
  } catch {
    if (list.value) window.getSelection()?.selectAllChildren(list.value);
  }
}

function download() {
  const blob = new Blob([text.value + '\n'], { type: 'text/plain;charset=utf-8' });
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `兑换码-批次${props.batch.id}-${props.codes.length}个.txt`;
  link.click();
  URL.revokeObjectURL(link.href);
  downloaded.value = true;
  notify('已下载 .txt 卡密文件');
}
</script>

<template>
  <div class="scrim"></div>
  <div ref="root" class="dialog wide" role="alertdialog" aria-modal="true" aria-labelledby="reveal-title">
    <div class="dialog-head">
      <h2 id="reveal-title">
        <span>已生成 {{ codes.length }} 个兑换码</span>
        <span class="tag num">批次 #{{ batch.id }}</span>
      </h2>
    </div>

    <div class="dialog-body">
      <div class="notice warn" role="status">
        <Icon name="alert" :size="15" />
        <span>明文仅显示这一次，关闭后无法找回，请先下载或复制。</span>
      </div>

      <div class="meta-row num">
        <span class="tag"><i class="dot" :class="batch.kind === 'monthly' ? 'warn' : 'ok'"></i>{{ kindLabels[batch.kind] }}</span>
        <span>单码 {{ money(batch.quota) }}（{{ xiaobanPoints(batch.quota) }}）· 兑换后 {{ batch.valid_days }} 天有效</span>
      </div>

      <ol ref="list" class="codes" aria-label="全部兑换码">
        <li v-for="(c, i) in codes" :key="c">
          <span class="no num">{{ i + 1 }}</span>
          <code>{{ c }}</code>
        </li>
      </ol>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn outline" @click="copy">
        <Icon :name="copied ? 'check' : 'copy'" :size="14" />{{ copied ? '已复制' : '复制全部' }}
      </button>
      <button type="button" class="btn" :class="{ primary: !safe, outline: safe }" @click="download">
        <Icon name="download" :size="14" />{{ downloaded ? '再次下载' : '下载 .txt' }}
      </button>
      <button type="button" class="btn" :class="{ primary: safe, ghost: !safe }" @click="emit('close')">完成</button>
    </div>
  </div>
</template>

<style scoped>
.meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 12.5px;
  color: var(--ink-2);
}
.codes {
  padding: 12px 14px;
  max-height: 240px;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px 16px;
  border-radius: var(--r-md);
  background: var(--subtle);
  border: 1px solid var(--line);
  color: var(--ink-1);
  user-select: text;
}
.codes li { display: flex; align-items: baseline; gap: 9px; padding: 2px 0; }
.no { width: 2.5ch; text-align: right; font-size: 11px; color: var(--ink-4); flex: none; }
code { font-family: var(--mono); font-size: 12.5px; letter-spacing: 0.05em; }
@media (max-width: 520px) {
  .codes { grid-template-columns: 1fr; }
}
</style>
