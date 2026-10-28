<script setup lang="ts">
import { onMounted, ref } from 'vue'
import PagerNav from './PagerNav.vue'
import { contentApi, type WeeklyIssueItem } from '@/api/content'
import { useToast } from '@/stores/toast'

const toast = useToast()
const items = ref<WeeklyIssueItem[]>([])
const page = ref(1)
const total = ref(0)
const pageSize = ref(20)
const loading = ref(true)

async function load(p = 1) {
  loading.value = true
  try {
    const res = await contentApi.weeklyList(p)
    items.value = res.items
    ;({ page: page.value, total: total.value, page_size: pageSize.value } = res)
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
}
onMounted(() => load())
</script>

<template>
  <div>
    <p v-if="loading && !items.length" class="state">加载中</p>
    <p v-else-if="!items.length" class="state">周刊还没有发布，敬请期待。</p>
    <ul v-else class="issues">
      <li v-for="i in items" :key="i.week">
        <RouterLink :to="`/weekly/${i.week}`">
          <span class="week">{{ i.week }}</span>
          <h2>{{ i.title }}</h2>
          <p v-if="i.summary">{{ i.summary }}</p>
        </RouterLink>
      </li>
    </ul>
    <PagerNav :page="page" :page-size="pageSize" :total="total" @go="load" />
  </div>
</template>

<style scoped>
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.issues {
  list-style: none;
  margin: 0;
  padding: 0 16px;
}
.issues li {
  border-bottom: 1px solid var(--line);
}
.issues a {
  display: block;
  padding: 14px 0;
}
.week {
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
h2 {
  margin: 2px 0 0;
  font-size: var(--fs-lg);
  font-weight: 500;
}
p {
  margin: 4px 0 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
</style>
