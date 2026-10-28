<script setup lang="ts">
// 内容层页面的外壳：顶部一行一级栏目，下面是单栏内容。
import { useRoute } from 'vue-router'

const route = useRoute()
const SECTIONS = [
  { to: '/', label: '讨论', match: ['home', 'category', 'topic', 'tag', 'search'] },
  { to: '/resources', label: '资源', match: ['resources', 'pack'] },
  { to: '/weekly', label: '周刊', match: ['weekly', 'weekly-issue'] },
  { to: '/projects', label: '项目', match: ['projects', 'project'] },
]
function on(match: string[]) {
  return match.includes(String(route.name))
}
</script>

<template>
  <div class="content-shell">
    <nav class="sections" aria-label="栏目">
      <RouterLink v-for="s in SECTIONS" :key="s.to" :to="s.to" :class="{ on: on(s.match) }">{{ s.label }}</RouterLink>
    </nav>
    <div class="body"><slot /></div>
  </div>
</template>

<style scoped>
.content-shell {
  max-width: 960px;
  margin: 0 auto;
  background: var(--surface);
  min-height: 70vh;
}
.sections {
  display: flex;
  justify-content: center;
  gap: 28px;
  padding: 12px 0 0;
  border-bottom: 1px solid var(--line);
}
.sections a {
  padding: 6px 2px 10px;
  font-size: var(--fs-lg);
  font-weight: 500;
  color: var(--ink-3);
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}
.sections a.on {
  color: var(--ink);
  border-bottom-color: var(--brand);
}
.body {
  padding: 0 0 24px;
}
/* 桌面端顶栏已有同样的栏目入口，这一行只在手机上显示。 */
@media (min-width: 768px) {
  .sections {
    display: none;
  }
}
</style>
