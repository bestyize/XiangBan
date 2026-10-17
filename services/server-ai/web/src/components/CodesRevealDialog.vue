<script setup lang="ts">
import { computed, ref } from 'vue';
import { kindLabels, money } from '../format';
import type { Batch } from '../types';

const props = defineProps<{ batch: Batch; codes: string[] }>();
const emit = defineEmits<{ close: [] }>();

const copied = ref(false);
const downloaded = ref(false);
const text = computed(() => props.codes.join('\n'));

async function copy() {
  try {
    await navigator.clipboard.writeText(text.value);
    copied.value = true;
  } catch {
    // 没有剪贴板权限时，退回到选中文本让用户手动复制。
    (document.getElementById('codes-area') as HTMLTextAreaElement | null)?.select();
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
}

const safe = computed(() => copied.value || downloaded.value);
</script>

<template>
  <div class="scrim"></div>
  <div class="dialog wide" role="alertdialog" aria-modal="true" aria-labelledby="reveal-title">
    <div class="dialog-head">
      <h2 id="reveal-title">已生成 {{ codes.length }} 个兑换码</h2>
      <p class="hint">{{ kindLabels[batch.kind] }}，每个 {{ money(batch.quota) }}，兑换后 {{ batch.valid_days }} 天有效。</p>
    </div>
    <div class="dialog-body">
      <div class="notice warn" role="status">
        <strong>这些码只会显示这一次。</strong>系统里只保存它们的校验值，关闭后无法再找回。请先下载或复制，再关闭这个窗口。
      </div>
      <label class="sr" for="codes-area">全部兑换码</label>
      <textarea id="codes-area" class="textarea mono codes" readonly :value="text" rows="8" @focus="($event.target as HTMLTextAreaElement).select()"></textarea>
      <p class="hint">下载的是文本文件，一行一个码，可以直接导入淘宝的自动发货（卡密）库存。</p>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="copy">{{ copied ? '已复制' : '复制全部' }}</button>
      <button type="button" class="btn" :class="{ primary: !safe }" @click="download">{{ downloaded ? '已下载，再下载一次' : '下载 txt' }}</button>
      <button type="button" class="btn" :class="{ primary: safe }" @click="emit('close')">我已保存，关闭</button>
    </div>
  </div>
</template>

<style scoped>
.wide { width: min(560px, calc(100vw - 32px)); }
.codes { min-height: 190px; line-height: 1.7; letter-spacing: 0.04em; resize: none; }
.sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
</style>
