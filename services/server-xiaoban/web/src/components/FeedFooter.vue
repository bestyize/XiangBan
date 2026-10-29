<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { useIntersectionObserver } from '@vueuse/core'

const props = defineProps<{ loadingMore: boolean; hasMore: boolean; error: string | null; count: number }>()
const emit = defineEmits<{ more: []; retry: [] }>()

const sentinel = ref<HTMLElement>()

// The observer only reports changes. When a page is too short to push the sentinel out of range
// (a filter can thin a page out), it stays in range, never reports again, and scrolling would stall.
watch(
  () => props.count,
  async () => {
    await nextTick()
    const top = sentinel.value?.getBoundingClientRect().top
    if (top !== undefined && top < window.innerHeight + 900 && props.hasMore && !props.loadingMore && !props.error) {
      emit('more')
    }
  },
)

// Start fetching well before the user reaches the end so scrolling never stalls.
useIntersectionObserver(
  sentinel,
  ([entry]) => {
    if (entry.isIntersecting) emit('more')
  },
  { rootMargin: '0px 0px 900px 0px' },
)
</script>

<template>
  <div class="footer">
    <div ref="sentinel" class="sentinel" aria-hidden="true" />
    <p v-if="error && count > 0" class="note">
      {{ error }}
      <button class="link" type="button" @click="emit('retry')">重试</button>
    </p>
    <p v-else-if="loadingMore" class="note">加载中…</p>
    <p v-else-if="!hasMore && count > 0" class="note end">已经到底了</p>
  </div>
</template>

<style scoped>
.footer {
  position: relative;
  padding: 28px 0 8px;
  text-align: center;
}

.sentinel {
  position: absolute;
  inset: 0 0 auto;
  height: 1px;
}

.note {
  font-size: 13px;
  color: var(--text-3);
}

.link {
  margin-left: 8px;
  color: var(--text);
  text-decoration: underline;
  text-underline-offset: 3px;
}
</style>
