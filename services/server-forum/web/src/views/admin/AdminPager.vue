<script setup lang="ts">
// 列表底部的分页条：显示总数与页码，只有一页时只显示总数。
import { computed } from 'vue'

const props = defineProps<{ page: number; pageSize: number; total: number; loading?: boolean }>()
const emit = defineEmits<{ go: [page: number] }>()

const pages = computed(() => Math.max(1, Math.ceil(props.total / props.pageSize)))
</script>

<template>
  <div class="adm-pager">
    <span>共 {{ total }} 条</span>
    <div v-if="pages > 1">
      <button class="btn btn-sm" :disabled="loading || page <= 1" @click="emit('go', page - 1)">上一页</button>
      <span>第 {{ page }} 页，共 {{ pages }} 页</span>
      <button class="btn btn-sm" :disabled="loading || page >= pages" @click="emit('go', page + 1)">下一页</button>
    </div>
  </div>
</template>
