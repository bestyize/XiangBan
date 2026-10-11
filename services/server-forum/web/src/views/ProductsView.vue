<script setup lang="ts">
import { computed, ref } from 'vue'
import ForumShell from '@/components/ForumShell.vue'
import { useMeta } from '@/stores/meta'
import { useHead } from '@/lib/head'

useHead(() => ({ title: '产品' }))
const meta = useMeta()
const q = ref('')

const list = computed(() => {
  const k = q.value.trim().toLowerCase()
  return meta.products.filter(p => !k || `${p.name} ${p.vendor} ${p.slug}`.toLowerCase().includes(k))
})
</script>

<template>
  <ForumShell active="products" :rail="false">
    <header class="head">
      <div>
        <h1>产品</h1>
        <p class="muted">每个产品一页，汇总它历史上所有羊毛和相关讨论。</p>
      </div>
      <input v-model="q" class="input filter" type="search" placeholder="筛选产品" aria-label="筛选产品" />
    </header>
    <ul class="grid">
      <li v-for="p in list" :key="p.id">
        <RouterLink :to="`/p/${p.slug}`" class="card">
          <span class="initial" aria-hidden="true">{{ Array.from(p.name)[0] }}</span>
          <span class="text">
            <strong>{{ p.name }}</strong>
            <span class="muted">{{ p.vendor && p.vendor !== p.name ? p.vendor : '' }}</span>
          </span>
          <span class="n" :class="{ zero: !p.active_deal_count }">
            <b>{{ p.active_deal_count }}</b>
            <small>条在领</small>
          </span>
        </RouterLink>
      </li>
    </ul>
    <p v-if="!list.length" class="muted empty">没有找到这个产品。</p>
  </ForumShell>
</template>

<style scoped>
.head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 16px;
  margin-bottom: 16px;
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.head p {
  margin: 2px 0 0;
  font-size: var(--fs-sm);
}
.filter {
  max-width: 220px;
}
.grid {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}
.card {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  padding: 12px 14px;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-md);
}
.card:hover {
  border-color: var(--line-strong);
}
.initial {
  width: 36px;
  height: 36px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: var(--surface-2);
  border: 1px solid var(--line);
  font-family: var(--font-display);
  font-weight: 700;
  font-size: 17px;
}
.text {
  display: grid;
  min-width: 0;
}
.text strong {
  font-size: var(--fs-sm);
}
.text .muted {
  font-size: var(--fs-xs);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.n {
  display: grid;
  justify-items: end;
  line-height: 1.1;
}
.n b {
  font-family: var(--font-display);
  font-size: 18px;
  color: var(--valid);
}
.n small {
  font-size: 10.5px;
  color: var(--ink-3);
}
.n.zero b {
  color: var(--ink-3);
}
.empty {
  text-align: center;
  padding: 40px;
}
@media (max-width: 767px) {
  .head {
    flex-direction: column;
    align-items: stretch;
  }
  h1 {
    font-size: var(--fs-xl);
  }
  .filter {
    max-width: none;
  }
  .grid {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
