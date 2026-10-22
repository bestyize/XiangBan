<script setup lang="ts">
import { BookOpen, FileText, Globe, NotebookPen } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
defineProps<{ store: ControlStore }>();
</script>
<template>
  <section class="library-page xb-page" data-testid="library-page" aria-label="资料库">
    <header class="xb-page-head"><h1 class="xb-page-title">资料库</h1></header>
    <div class="xb-empty library-empty">
      <span class="xb-empty-icon library-icon"><BookOpen :size="26" /></span>
      <h2>还没有资料</h2>
      <p>把常用文件、网页和笔记放进来，小伴回答时会参考它们。</p>
      <span class="library-soon">即将上线</span>
      <div class="library-kinds" aria-hidden="true">
        <span><FileText :size="14" />文件</span><span><Globe :size="14" />网页</span><span><NotebookPen :size="14" />笔记</span>
      </div>
    </div>
  </section>
</template>
<style scoped>
.library-page{flex:1;min-height:0;display:flex;flex-direction:column;gap:18px;padding:22px 28px 0}
.library-empty{flex:1;padding:48px 24px 72px}
.library-icon{width:64px;height:64px;border-radius:18px}
.library-empty h2{font-size:17px}
.library-soon{display:inline-flex;align-items:center;height:22px;padding:0 9px;border-radius:999px;background:var(--ui-control);color:var(--ui-text-secondary);font-size:12px;font-weight:500}
.library-kinds{display:flex;gap:8px;margin-top:10px;flex-wrap:wrap;justify-content:center}
.library-kinds span{display:inline-flex;align-items:center;gap:6px;height:26px;padding:0 10px;border-radius:8px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border);color:var(--ui-text-tertiary);font-size:12px}
</style>
