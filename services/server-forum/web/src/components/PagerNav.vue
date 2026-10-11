<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ page: number; pageSize: number; total: number }>()
const emit = defineEmits<{ go: [page: number] }>()

const pages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)))
</script>

<template>
  <nav v-if="pages > 1" class="pager" aria-label="分页">
    <button class="btn btn-sm" type="button" :disabled="page <= 1" @click="emit('go', page - 1)">上一页</button>
    <span class="where">第 {{ page }} / {{ pages }} 页</span>
    <button class="btn btn-sm" type="button" :disabled="page >= pages" @click="emit('go', page + 1)">下一页</button>
  </nav>
</template>

<style scoped>
.pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  margin: 16px 0 4px;
}
.where {
  font-size: var(--fs-sm);
  color: var(--ink-3);
  font-variant-numeric: tabular-nums;
}
</style>
