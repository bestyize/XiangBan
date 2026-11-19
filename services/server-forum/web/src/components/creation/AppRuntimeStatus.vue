<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { fmtSize } from '@/lib/content'

const props = defineProps<{ message: string; error?: string; local?: boolean; receivedBytes?: number; totalBytes?: number }>()
defineEmits<{ retry: [] }>()

const offline = ref(!navigator.onLine)
const failure = computed(() => props.error || (!props.local && offline.value ? '网络连接已断开。' : ''))
const percent = computed(() => props.totalBytes
  ? Math.floor((props.receivedBytes ?? 0) / props.totalBytes * 100)
  : undefined)

function updateConnection() { offline.value = !navigator.onLine }

onMounted(() => {
  window.addEventListener('online', updateConnection)
  window.addEventListener('offline', updateConnection)
  updateConnection()
})
onBeforeUnmount(() => {
  window.removeEventListener('online', updateConnection)
  window.removeEventListener('offline', updateConnection)
})
</script>

<template>
  <div class="runtime-status">
    <div class="status-content">
      <template v-if="failure">
        <p role="alert">{{ failure }}</p>
        <button type="button" class="btn" @click="$emit('retry')">重试</button>
      </template>
      <template v-else>
        <p role="status">{{ message }}</p>
        <div class="progress" role="progressbar" aria-label="作品加载进度" :aria-valuenow="percent">
          <span :class="{ indeterminate: percent === undefined }"
            :style="percent === undefined ? undefined : { width: `${percent}%` }" />
        </div>
        <p v-if="receivedBytes !== undefined" class="download-size">
          <span v-if="totalBytes">{{ fmtSize(receivedBytes) }} / {{ fmtSize(totalBytes) }}</span>
          <span v-else>已接收 {{ fmtSize(receivedBytes) }}</span>
          <span v-if="percent !== undefined">{{ percent }}%</span>
        </p>
      </template>
    </div>
  </div>
</template>

<style scoped>
.runtime-status {
  position: absolute;
  inset: 0;
  z-index: 1;
  display: grid;
  place-items: center;
  padding: 20px;
  background: var(--surface);
  color: var(--ink-2);
  text-align: center;
}
.status-content { display: grid; justify-items: center; gap: 10px; width: min(100%, 280px); }
p { margin: 0; overflow-wrap: anywhere; }
.progress { width: 100%; height: 6px; overflow: hidden; border-radius: 3px; background: var(--line); }
.progress span { display: block; height: 100%; border-radius: inherit; background: var(--brand); }
.progress .indeterminate { width: 40%; animation: loading 1.6s ease-in-out infinite; }
.download-size { display: flex; justify-content: space-between; gap: 8px; width: 100%; font-size: 12px; font-variant-numeric: tabular-nums; }
.btn { min-height: 44px; }
@keyframes loading { from { transform: translateX(-100%); } to { transform: translateX(250%); } }
@media (prefers-reduced-motion: reduce) {
  .progress .indeterminate {
    width: 100%;
    animation: none;
    background: repeating-linear-gradient(135deg, var(--brand) 0 3px, transparent 3px 7px);
  }
}
</style>
