<script setup lang="ts">
import { useModal } from '../composables/useModal';
import { computed, ref } from 'vue';
import { kindLabels } from '../format';
import type { Batch } from '../types';
import Icon from './Icon.vue';
import Money from './Money.vue';

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
  } catch {
    // 没有剪贴板权限时，选中整个列表让用户手动复制。
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
}
</script>

<template>
  <div class="scrim"></div>
  <div ref="root" class="dialog wide" role="alertdialog" aria-modal="true" aria-labelledby="reveal-title">
    <div class="dialog-head">
      <h2 id="reveal-title">已生成 {{ codes.length }} 个兑换码</h2>
      <p class="hint row wrap">
        <span class="tag" :class="batch.kind === 'monthly' ? 'amber' : 'green'">{{ kindLabels[batch.kind] }}</span>
        每个 <Money :points="batch.quota" size="sm" />，兑换后 {{ batch.valid_days }} 天有效
      </p>
    </div>
    <div class="dialog-body">
      <div class="notice warn" role="status">
        <Icon name="alert" :size="17" />
        <span><strong>这些码只会显示这一次。</strong>系统里只保存它们的校验值，关闭后无法再找回。请先下载或复制，再关闭这个窗口。</span>
      </div>
      <ol ref="list" class="codes" aria-label="全部兑换码">
        <li v-for="(c, i) in codes" :key="c"><span class="no num faint">{{ i + 1 }}</span><code>{{ c }}</code></li>
      </ol>
      <p class="hint">下载的是文本文件，一行一个码，可以直接导入淘宝的自动发货（卡密）库存。</p>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="copy"><Icon :name="copied ? 'check' : 'copy'" :size="16" />{{ copied ? '已复制' : '复制全部' }}</button>
      <button type="button" class="btn" :class="{ primary: !safe }" @click="download"><Icon name="download" :size="16" />{{ downloaded ? '已下载，再下载一次' : '下载 txt' }}</button>
      <button type="button" class="btn" :class="{ primary: safe }" @click="emit('close')">我已保存，关闭</button>
    </div>
  </div>
</template>

<style scoped>
.hint { align-items: center; gap: 6px; }
.codes { list-style: none; margin: 0; padding: 12px 14px; max-height: 268px; overflow-y: auto; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 2px 18px; border-radius: 12px; background: #0f1b20; color: #dff5ef; user-select: text; }
.codes li { display: flex; align-items: baseline; gap: 10px; padding: 4px 0; }
.no { width: 2ch; text-align: right; font-size: 11.5px; color: #86a3a0; flex: none; }
code { font-family: var(--mono); font-size: 13.5px; letter-spacing: 0.06em; }
@media (max-width: 520px) { .codes { grid-template-columns: 1fr; } }
</style>
