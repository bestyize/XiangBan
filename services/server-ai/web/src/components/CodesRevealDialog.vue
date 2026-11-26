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
      <div>
        <h2 id="reveal-title">已生成 {{ codes.length }} 个兑换码（批次 #{{ batch.id }}）</h2>
        <p class="sub num">
          <span class="tag" :class="batch.kind === 'monthly' ? 'warn' : 'ok'">{{ kindLabels[batch.kind] }}</span>
          单码 {{ money(batch.quota) }}（{{ xiaobanPoints(batch.quota) }}）· 兑换后 {{ batch.valid_days }} 天有效
        </p>
      </div>
    </div>

    <div class="dialog-body">
      <div class="notice warn" role="status">
        <Icon name="alert" :size="16" />
        <span><strong>明文仅显示这一次。</strong>系统只保存 HMAC 校验摘要，关闭窗口后无法找回。请先下载或复制后再关闭。</span>
      </div>

      <ol ref="list" class="codes" aria-label="全部兑换码">
        <li v-for="(c, i) in codes" :key="c">
          <span class="no num">{{ i + 1 }}</span>
          <code>{{ c }}</code>
        </li>
      </ol>

      <p class="hint">导出的 .txt 文本文件为一行一个码，可直接导入淘宝自动发货（卡密）库存。</p>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" @click="copy">
        <Icon :name="copied ? 'check' : 'copy'" :size="14" />{{ copied ? '已复制全部' : '复制全部' }}
      </button>
      <button type="button" class="btn" :class="{ primary: !safe }" @click="download">
        <Icon name="download" :size="14" />{{ downloaded ? '已下载，再下一次' : '下载 .txt 卡密' }}
      </button>
      <button type="button" class="btn" :class="{ primary: safe }" @click="emit('close')">我已保存，关闭</button>
    </div>
  </div>
</template>

<style scoped>
.sub { display: flex; align-items: center; gap: 7px; flex-wrap: wrap; margin-top: 5px; }
.codes {
  padding: 12px 14px;
  max-height: 256px;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 4px 16px;
  border-radius: 12px;
  background: #141317;
  color: #f3f1f5;
  user-select: text;
}
.codes li { display: flex; align-items: baseline; gap: 9px; padding: 3px 0; }
.no { width: 2.5ch; text-align: right; font-size: 11px; color: #7a7682; flex: none; }
code { font-family: var(--mono); font-size: 13px; letter-spacing: 0.06em; }
@media (max-width: 520px) {
  .codes { grid-template-columns: 1fr; }
}
</style>
