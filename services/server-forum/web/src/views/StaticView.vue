<script setup lang="ts">
import { computed } from 'vue'
import rules from '@/content/rules'
import terms from '@/content/terms'
import privacy from '@/content/privacy'
import { useHead } from '@/lib/head'

const props = defineProps<{ page: 'rules' | 'terms' | 'privacy' }>()
const PAGES = { rules, terms, privacy }
const doc = computed(() => PAGES[props.page])
useHead(() => ({ title: doc.value.title }))
</script>

<template>
  <article class="doc">
    <nav class="switch" aria-label="站点文档">
      <RouterLink to="/rules" :class="{ on: page === 'rules' }">社区规则</RouterLink>
      <RouterLink to="/terms" :class="{ on: page === 'terms' }">用户协议</RouterLink>
      <RouterLink to="/privacy" :class="{ on: page === 'privacy' }">隐私政策</RouterLink>
    </nav>
    <h1>{{ doc.title }}</h1>
    <p class="muted updated">更新于 {{ doc.updated }}</p>
    <div class="panel prose body" v-html="doc.html" />
  </article>
</template>

<style scoped>
.doc {
  max-width: 760px;
  margin: 0 auto;
}
.switch {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
}
.switch a {
  padding: 4px 12px;
  border-radius: 14px;
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-2);
  border: 1px solid transparent;
}
.switch a.on {
  border-color: var(--line-strong);
  background: var(--surface);
  color: var(--ink);
}
h1 {
  margin: 0;
  font-size: var(--fs-2xl);
  font-weight: 800;
}
.updated {
  margin: 4px 0 16px;
  font-size: var(--fs-xs);
}
.body {
  padding: 22px 26px;
}
.body :deep(h2) {
  font-size: 1.15em;
}
@media (max-width: 767px) {
  .body {
    padding: 16px;
  }
}
</style>
