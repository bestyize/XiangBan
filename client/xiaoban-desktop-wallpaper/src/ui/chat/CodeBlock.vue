<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue';
import { Check, Copy } from 'lucide-vue-next';

const props = defineProps<{ lang: string; code: string }>();
const state = ref<'idle' | 'copied' | 'failed'>('idle');
let timer: ReturnType<typeof setTimeout> | undefined;

async function copy() {
  try { await navigator.clipboard.writeText(props.code); state.value = 'copied'; } catch { state.value = 'failed'; }
  clearTimeout(timer);
  timer = setTimeout(() => { state.value = 'idle'; }, 1600);
}
onBeforeUnmount(() => clearTimeout(timer));
</script>

<template>
  <div class="code-block">
    <div class="code-head">
      <span class="code-lang">{{ lang || '代码' }}</span>
      <button type="button" class="code-copy" data-testid="code-copy" :aria-label="state === 'copied' ? '已复制' : '复制代码'" @click="copy">
        <Check v-if="state === 'copied'" :size="13" /><Copy v-else :size="13" />{{ state === 'copied' ? '已复制' : state === 'failed' ? '复制失败' : '复制' }}
      </button>
    </div>
    <pre><code>{{ code }}</code></pre>
  </div>
</template>

<style scoped>
.code-block{margin:10px 0;border:1px solid var(--ui-line);border-radius:10px;background:var(--ui-surface-alt);overflow:hidden}
.code-head{display:flex;justify-content:space-between;align-items:center;padding:5px 10px;font-size:12px;color:var(--ui-text-tertiary);border-bottom:1px solid var(--ui-line)}
.code-copy{border:0;background:none;color:inherit;cursor:pointer;display:inline-flex;gap:5px;align-items:center;font:inherit;padding:2px 4px;border-radius:5px}
.code-copy:hover{color:var(--ui-text-primary)}
pre{margin:0;padding:12px 14px;overflow:auto;font-size:12.5px;line-height:1.6;max-height:520px}
code{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;white-space:pre}
</style>
