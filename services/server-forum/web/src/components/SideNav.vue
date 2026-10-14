<script setup lang="ts">
import { computed } from 'vue'
import { SHOW_CHILD_THRESHOLD, useMeta } from '@/stores/meta'

const props = defineProps<{ active?: string }>()
const meta = useMeta()

// 福利羊毛在侧栏中紧随随便唠唠，其他分类保留后台配置的顺序。
const navTree = computed(() => {
  const nodes = [...meta.tree]
  const dealsIndex = nodes.findIndex(node => node.slug === 'deals')
  if (dealsIndex < 0) return nodes
  const [deals] = nodes.splice(dealsIndex, 1)
  const casualIndex = nodes.findIndex(node => node.slug === 'casual')
  const siteIndex = nodes.findIndex(node => node.slug === 'site')
  const insertAt = casualIndex >= 0 ? casualIndex + 1 : siteIndex >= 0 ? siteIndex : dealsIndex
  nodes.splice(insertAt, 0, deals!)
  return nodes
})

/** 当前所在的一级分类：选中二级分类时，它的父级也算展开。 */
const activeRoot = computed(() => {
  const c = props.active ? meta.bySlug.get(props.active) : undefined
  return meta.parentOf(c)?.slug
})

function visibleChildren(node: (typeof meta.tree)[number]) {
  // 展开的一级分类显示全部子分类；其余只露出帖子够多的子分类。
  if (activeRoot.value === node.slug || (node.kind === 'deal' && props.active === 'deals')) return node.children
  return node.children.filter(c => c.topic_count >= SHOW_CHILD_THRESHOLD)
}
</script>

<template>
  <nav class="side" aria-label="分类">
    <RouterLink to="/" class="item top" :class="{ on: active === 'home' }">首页</RouterLink>
    <div v-for="node in navTree" :key="node.id" class="group">
      <!-- 福利羊毛一级分类直接进可筛选的福利羊毛页 -->
      <RouterLink
        :to="node.kind === 'deal' ? '/deals' : `/c/${node.slug}`"
        class="item root"
        :class="{ on: active === node.slug || (node.kind === 'deal' && active === 'deals') }"
      >
        <span>{{ node.name }}</span>
        <span class="count">{{ meta.totalCount(node) || '' }}</span>
      </RouterLink>
      <RouterLink
        v-for="child in visibleChildren(node)"
        :key="child.id"
        :to="`/c/${child.slug}`"
        class="item child"
        :class="{ on: active === child.slug }"
      >
        <span>{{ child.name }}</span>
        <span class="count">{{ child.topic_count || '' }}</span>
      </RouterLink>
    </div>
  </nav>
</template>

<style scoped>
.side {
  display: grid;
  gap: 1px;
  padding-bottom: 20px;
}
.group {
  display: grid;
  gap: 1px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--line);
}
.item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: var(--r-sm);
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
.item:hover {
  background: var(--surface);
  color: var(--ink);
}
.item.top,
.item.root {
  font-weight: 700;
  color: var(--ink);
  font-size: var(--fs-md);
}
.item.child {
  padding-left: 22px;
}
.item.on {
  background: var(--surface);
  color: var(--ink);
  box-shadow: inset 3px 0 0 var(--brand), 0 0 0 1px var(--line);
}
.count {
  font-family: var(--font-display);
  font-size: 12px;
  color: var(--ink-3);
  font-weight: 500;
  font-variant-numeric: tabular-nums;
}
</style>
