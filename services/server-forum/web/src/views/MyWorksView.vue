<script setup lang="ts">
import { onMounted, ref } from 'vue'
import ContentShell from '@/components/ContentShell.vue'
import { contentApi, type MyPack } from '@/api/content'
import { kindLabel, trustLabel } from '@/lib/content'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

useHead(() => ({ title: '我的作品' }))
const toast = useToast()
const items = ref<MyPack[]>([])
const loading = ref(true)

/** 一条作品当前对外的状态：已下架、被拒、审核中（未审核且在队列里）、已上架。 */
function state(p: MyPack): string {
  if (p.status === 'removed') return '已下架'
  if (p.status === 'rejected') return '未通过审核'
  if (p.in_review) return '审核中，期间展示为未审核'
  return p.trust_level < 2 ? '已审核' : '已上架'
}

async function load() {
  loading.value = true
  try {
    items.value = (await contentApi.myPacks()).items
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
}

async function remove(p: MyPack) {
  if (!window.confirm(`确定下架「${p.title}」？已经应用过它的人不受影响。`)) return
  try {
    await contentApi.removeOwn(p.id)
    toast.show('已下架')
    await load()
  } catch (e) {
    toast.error(e)
  }
}

onMounted(load)
</script>

<template>
  <ContentShell>
    <div class="head">
      <h1>我的作品</h1>
      <RouterLink class="btn btn-primary" to="/publish">发布内容包</RouterLink>
    </div>
    <p v-if="loading" class="state">加载中</p>
    <p v-else-if="!items.length" class="state">还没有发布过内容。</p>
    <ul v-else class="works">
      <li v-for="p in items" :key="p.id">
        <div class="main">
          <RouterLink :to="`/packs/${p.id}`"><strong>{{ p.title }}</strong></RouterLink>
          <p class="meta">
            <span>{{ kindLabel(p.kind) }}</span>
            <span>{{ trustLabel(p.trust_level) }}</span>
            <span>v{{ p.version }}</span>
            <span>{{ state(p) }}</span>
            <span v-if="p.apply_count">{{ p.apply_count }} 次使用</span>
            <span>{{ timeAgo(p.updated_at) }}更新</span>
          </p>
        </div>
        <div class="ops">
          <RouterLink v-if="p.status !== 'removed'" class="btn btn-sm" :to="`/publish?pack=${p.id}`">发布新版本</RouterLink>
          <button v-if="p.status !== 'removed'" class="btn btn-sm" type="button" @click="remove(p)">下架</button>
        </div>
      </li>
    </ul>
  </ContentShell>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 16px 8px;
}
h1 {
  margin: 0;
  font-size: var(--fs-xl);
  font-weight: 600;
}
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.works {
  list-style: none;
  margin: 0;
  padding: 0 16px;
}
.works li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid var(--line);
}
.meta {
  margin: 2px 0 0;
  display: flex;
  flex-wrap: wrap;
  gap: 2px 10px;
  color: var(--ink-3);
  font-size: var(--fs-xs);
}
.ops {
  display: flex;
  gap: 6px;
  flex: none;
}
</style>
